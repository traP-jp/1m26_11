use std::{io::Cursor, time::Duration};

use async_trait::async_trait;
use aws_sdk_s3::{
    config::{BehaviorVersion, Credentials, Region},
    primitives::ByteStream,
};
use image::{ImageFormat, ImageReader};
use sha2::{Digest, Sha256};
use thiserror::Error;
use tokio::time::timeout;
use uuid::Uuid;

use crate::config::StorageConfig;

pub(crate) const MAX_IMAGE_FILE_BYTES: usize = 5_242_880;
pub(crate) const MAX_IMAGE_EDGE: u32 = 4_096;
pub(crate) const MAX_IMAGE_PIXELS: u64 = 16_777_216;
pub(crate) const MAX_ALT_CHARACTERS: usize = 200;
pub(crate) const IMAGE_STORAGE_TIMEOUT: Duration = Duration::from_secs(10);
pub(crate) const IMAGE_CACHE_CONTROL: &str = "public, max-age=31536000, immutable";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ValidatedImage {
    pub bytes: Vec<u8>,
    pub alt: String,
    pub extension: &'static str,
    pub content_type: &'static str,
    pub sha256: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub(crate) enum ImageValidationError {
    #[error("image file is empty")]
    EmptyFile,

    #[error("image file is too large")]
    ImageTooLarge,

    #[error("image type is not supported")]
    UnsupportedImageType,

    #[error("image data is invalid")]
    InvalidImage,

    #[error("image dimensions exceed the limit")]
    ImageDimensionsExceeded,

    #[error("image alt text is invalid")]
    InvalidAlt,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ImageStorageUpload {
    pub object_key: String,
    pub bytes: Vec<u8>,
    pub content_type: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub(crate) enum ImageStorageError {
    #[error("storage provider rejected the upload")]
    ProviderError,

    #[error("storage provider is unavailable")]
    Unavailable,
}

#[async_trait]
pub(crate) trait ImageStorage: Send + Sync {
    async fn upload(&self, upload: ImageStorageUpload) -> Result<(), ImageStorageError>;
}

pub(crate) struct S3ImageStorage {
    client: aws_sdk_s3::Client,
    bucket: String,
}

impl S3ImageStorage {
    pub(crate) fn new(config: &StorageConfig) -> Self {
        let credentials = Credentials::new(
            config.access_key_id.clone(),
            config.secret_access_key.clone(),
            None,
            None,
            "image-upload-environment",
        );

        let sdk_config = aws_sdk_s3::Config::builder()
            .behavior_version(BehaviorVersion::latest())
            .credentials_provider(credentials)
            .region(Region::new(config.region.clone()))
            .endpoint_url(config.endpoint.clone())
            .force_path_style(config.force_path_style)
            .build();

        Self {
            client: aws_sdk_s3::Client::from_conf(sdk_config),
            bucket: config.bucket.clone(),
        }
    }
}

#[async_trait]
impl ImageStorage for S3ImageStorage {
    async fn upload(&self, upload: ImageStorageUpload) -> Result<(), ImageStorageError> {
        let request = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(upload.object_key)
            .body(ByteStream::from(upload.bytes))
            .content_type(upload.content_type)
            .cache_control(IMAGE_CACHE_CONTROL)
            .if_none_match("*")
            .send();

        match timeout(IMAGE_STORAGE_TIMEOUT, request).await {
            Ok(Ok(_)) => Ok(()),

            Ok(Err(error)) => {
                let status = error
                    .raw_response()
                    .map(|response| response.status().as_u16());

                Err(classify_storage_error(status))
            }

            Err(_) => Err(ImageStorageError::Unavailable),
        }
    }
}

pub(crate) fn build_image_object_key(
    room_id: Uuid,
    problem_id: Uuid,
    asset_id: Uuid,
    extension: &str,
) -> String {
    format!("v1/problems/{room_id}/{problem_id}/{asset_id}.{extension}")
}

fn classify_storage_error(status: Option<u16>) -> ImageStorageError {
    match status {
        Some(400..=499) => ImageStorageError::ProviderError,
        Some(500..=599) | None | Some(_) => ImageStorageError::Unavailable,
    }
}

pub(crate) fn validate_image(
    bytes: Vec<u8>,
    alt: &str,
) -> Result<ValidatedImage, ImageValidationError> {
    if bytes.is_empty() {
        return Err(ImageValidationError::EmptyFile);
    }

    if bytes.len() > MAX_IMAGE_FILE_BYTES {
        return Err(ImageValidationError::ImageTooLarge);
    }

    let format =
        image::guess_format(&bytes).map_err(|_| ImageValidationError::UnsupportedImageType)?;

    let (extension, content_type) = match format {
        ImageFormat::Png => ("png", "image/png"),
        ImageFormat::Jpeg => ("jpg", "image/jpeg"),
        ImageFormat::WebP => ("webp", "image/webp"),
        _ => return Err(ImageValidationError::UnsupportedImageType),
    };

    let (width, height) = ImageReader::with_format(Cursor::new(bytes.as_slice()), format)
        .into_dimensions()
        .map_err(|_| ImageValidationError::InvalidImage)?;

    if width == 0 || height == 0 {
        return Err(ImageValidationError::InvalidImage);
    }

    let pixel_count = u64::from(width) * u64::from(height);

    if width > MAX_IMAGE_EDGE || height > MAX_IMAGE_EDGE || pixel_count > MAX_IMAGE_PIXELS {
        return Err(ImageValidationError::ImageDimensionsExceeded);
    }

    ImageReader::with_format(Cursor::new(bytes.as_slice()), format)
        .decode()
        .map_err(|_| ImageValidationError::InvalidImage)?;

    let alt = alt.trim();

    if alt.is_empty() || alt.chars().count() > MAX_ALT_CHARACTERS {
        return Err(ImageValidationError::InvalidAlt);
    }

    let sha256 = Sha256::digest(&bytes).into();

    Ok(ValidatedImage {
        bytes,
        alt: alt.to_owned(),
        extension,
        content_type,
        sha256,
    })
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use image::{DynamicImage, ImageFormat};
    use sha2::{Digest, Sha256};
    use uuid::Uuid;

    use crate::config::StorageConfig;

    use super::{
        ImageStorage, ImageStorageError, ImageValidationError, MAX_ALT_CHARACTERS, MAX_IMAGE_EDGE,
        MAX_IMAGE_FILE_BYTES, S3ImageStorage, build_image_object_key, classify_storage_error,
        validate_image,
    };

    fn encoded_image(width: u32, height: u32, format: ImageFormat) -> Vec<u8> {
        let image = DynamicImage::new_rgb8(width, height);
        let mut output = Cursor::new(Vec::new());
        image
            .write_to(&mut output, format)
            .expect("test image should be encoded");

        output.into_inner()
    }

    #[test]
    fn accepts_png_jpeg_and_webp_from_actual_contents() {
        let cases = [
            (ImageFormat::Png, "png", "image/png"),
            (ImageFormat::Jpeg, "jpg", "image/jpeg"),
            (ImageFormat::WebP, "webp", "image/webp"),
        ];

        for (format, expected_extension, expected_content_type) in cases {
            let bytes = encoded_image(1, 1, format);
            let expected_sha256: [u8; 32] = Sha256::digest(&bytes).into();

            let image = validate_image(bytes.clone(), "  ろうそくが立った誕生日ケーキ  ")
                .expect("supported image should be valid");

            assert_eq!(image.bytes, bytes);
            assert_eq!(image.alt, "ろうそくが立った誕生日ケーキ");
            assert_eq!(image.extension, expected_extension);
            assert_eq!(image.content_type, expected_content_type);
            assert_eq!(image.sha256, expected_sha256);
        }
    }

    #[test]
    fn rejects_empty_and_oversized_files() {
        assert_eq!(
            validate_image(Vec::new(), "画像"),
            Err(ImageValidationError::EmptyFile)
        );

        assert_eq!(
            validate_image(vec![0; MAX_IMAGE_FILE_BYTES + 1], "画像",),
            Err(ImageValidationError::ImageTooLarge)
        );
    }

    #[test]
    fn rejects_unsupported_and_broken_image_contents() {
        assert_eq!(
            validate_image(b"GIF89a unsupported".to_vec(), "画像"),
            Err(ImageValidationError::UnsupportedImageType)
        );

        let mut broken_png = encoded_image(1, 1, ImageFormat::Png);
        broken_png.truncate(20);

        assert_eq!(
            validate_image(broken_png, "画像"),
            Err(ImageValidationError::InvalidImage)
        );
    }

    #[test]
    fn rejects_image_edge_over_limit() {
        let image = encoded_image(MAX_IMAGE_EDGE + 1, 1, ImageFormat::Png);

        assert_eq!(
            validate_image(image, "横幅が大きすぎる画像"),
            Err(ImageValidationError::ImageDimensionsExceeded)
        );
    }

    #[test]
    fn trims_alt_and_counts_unicode_characters() {
        let image = encoded_image(1, 1, ImageFormat::Png);
        let maximum_alt = "あ".repeat(MAX_ALT_CHARACTERS);

        let validated = validate_image(image.clone(), &format!("\u{3000}{maximum_alt}\n"))
            .expect("200 Unicode characters should be accepted");

        assert_eq!(validated.alt, maximum_alt);

        assert_eq!(
            validate_image(image.clone(), &"あ".repeat(MAX_ALT_CHARACTERS + 1),),
            Err(ImageValidationError::InvalidAlt)
        );

        assert_eq!(
            validate_image(image, " \n\t\u{3000} "),
            Err(ImageValidationError::InvalidAlt)
        );
    }

    #[test]
    fn builds_versioned_problem_asset_object_key() {
        let room_id = Uuid::parse_str("11111111-1111-4111-8111-111111111111")
            .expect("room UUID should be valid");
        let problem_id = Uuid::parse_str("22222222-2222-4222-8222-222222222221")
            .expect("problem UUID should be valid");
        let asset_id = Uuid::parse_str("33333333-3333-4333-8333-333333333333")
            .expect("asset UUID should be valid");

        assert_eq!(
            build_image_object_key(room_id, problem_id, asset_id, "png"),
            concat!(
                "v1/problems/",
                "11111111-1111-4111-8111-111111111111/",
                "22222222-2222-4222-8222-222222222221/",
                "33333333-3333-4333-8333-333333333333.png"
            )
        );
    }

    #[test]
    fn classifies_storage_provider_statuses() {
        assert_eq!(
            classify_storage_error(Some(400)),
            ImageStorageError::ProviderError
        );
        assert_eq!(
            classify_storage_error(Some(412)),
            ImageStorageError::ProviderError
        );
        assert_eq!(
            classify_storage_error(Some(499)),
            ImageStorageError::ProviderError
        );

        assert_eq!(
            classify_storage_error(Some(500)),
            ImageStorageError::Unavailable
        );
        assert_eq!(
            classify_storage_error(Some(503)),
            ImageStorageError::Unavailable
        );
        assert_eq!(classify_storage_error(None), ImageStorageError::Unavailable);
    }

    #[test]
    fn constructs_s3_storage_without_sending_a_request() {
        fn assert_image_storage<T: ImageStorage>() {}

        let config = StorageConfig {
            endpoint: "http://127.0.0.1:9000".to_owned(),
            bucket: "test-bucket".to_owned(),
            access_key_id: "test-access-key".to_owned(),
            secret_access_key: "test-secret-key".to_owned(),
            region: "test-region".to_owned(),
            force_path_style: true,
            public_base_url: "http://127.0.0.1:9000/test-bucket".to_owned(),
        };

        let _storage = S3ImageStorage::new(&config);
        assert_image_storage::<S3ImageStorage>();
    }
}
