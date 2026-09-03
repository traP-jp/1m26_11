use axum::{
    Json,
    extract::{
        Multipart, Path, State,
        multipart::{MultipartError, MultipartRejection},
    },
    http::{HeaderMap, StatusCode},
};
use chrono::{Duration, Utc};
use openapi_generated::models::Asset as PublicAsset;
use uuid::{Uuid, Version};

use crate::{
    AppState,
    error::AppError,
    image_upload::{
        ImageStorageError, ImageStorageUpload, ImageValidationError, MAX_IMAGE_FILE_BYTES,
        build_image_object_key, validate_image,
    },
    problem::Asset,
    repository::{
        AssetUploadClaimOutcome, AssetUploadClaimRequest, CompleteAssetUploadRequest,
        RepositoryError,
    },
};

const REQUEST_METHOD: &str = "POST";
const IDEMPOTENCY_TTL_HOURS: i64 = 24;

pub(crate) async fn upload_problem_asset(
    State(state): State<AppState>,
    Path((room_id, problem_id)): Path<(String, String)>,
    headers: HeaderMap,
    multipart: Result<Multipart, MultipartRejection>,
) -> Result<(StatusCode, Json<PublicAsset>), AppError> {
    let room_id = Uuid::parse_str(&room_id).map_err(|_| {
        AppError::api(
            StatusCode::BAD_REQUEST,
            "INVALID_PATH_PARAMETER",
            "room_id is invalid",
        )
    })?;

    let problem_id = Uuid::parse_str(&problem_id).map_err(|_| {
        AppError::api(
            StatusCode::BAD_REQUEST,
            "INVALID_PATH_PARAMETER",
            "problem_id is invalid",
        )
    })?;

    let idempotency_key = parse_idempotency_key(&headers)?;

    let multipart = multipart.map_err(|_| invalid_multipart_error())?;
    let (file, alt) = read_multipart(multipart).await?;
    let validated = validate_image(file, &alt).map_err(image_validation_error)?;

    let target = state
        .auth_repository
        .find_asset_upload_target(room_id, problem_id)
        .await?
        .ok_or_else(|| {
            AppError::api(
                StatusCode::NOT_FOUND,
                "ROOM_OR_PROBLEM_NOT_FOUND",
                "room or problem was not found",
            )
        })?;

    if target.is_published {
        return Err(AppError::api(
            StatusCode::CONFLICT,
            "PUBLISHED_ROOM_IMMUTABLE",
            "published room cannot be modified",
        ));
    }

    let request_path = format!("/api/rooms/{room_id}/problems/{problem_id}/assets");

    let claim = state
        .auth_repository
        .claim_asset_upload(&AssetUploadClaimRequest {
            request_method: REQUEST_METHOD.to_owned(),
            request_path: request_path.clone(),
            idempotency_key,
            file_sha256: validated.sha256,
            alt: validated.alt.clone(),
            expires_at: Utc::now() + Duration::hours(IDEMPOTENCY_TTL_HOURS),
        })
        .await?;

    let claim_token = match claim {
        AssetUploadClaimOutcome::Completed { asset } => {
            return public_asset_response(&state, asset);
        }
        AssetUploadClaimOutcome::Reused => {
            return Err(AppError::api(
                StatusCode::CONFLICT,
                "IDEMPOTENCY_KEY_REUSED",
                "Idempotency-Key was reused with different request data",
            ));
        }
        AssetUploadClaimOutcome::InProgress => {
            return Err(AppError::api(
                StatusCode::CONFLICT,
                "IDEMPOTENCY_REQUEST_IN_PROGRESS",
                "request with this Idempotency-Key is still being processed",
            ));
        }
        AssetUploadClaimOutcome::Acquired { claim_token } => claim_token,
    };

    let storage = state.image_storage.as_ref().ok_or_else(|| {
        AppError::internal(std::io::Error::other(
            "image upload route has no configured storage",
        ))
    })?;

    let asset_id = Uuid::new_v4();
    let object_key = build_image_object_key(room_id, problem_id, asset_id, validated.extension);

    let asset = Asset {
        asset_type: "image".to_owned(),
        object_key: object_key.clone(),
        alt: validated.alt.clone(),
    };

    let storage_result = storage
        .upload(ImageStorageUpload {
            object_key: object_key.clone(),
            bytes: validated.bytes,
            content_type: validated.content_type,
        })
        .await;

    if let Err(storage_error) = storage_result {
        if state
            .auth_repository
            .release_asset_upload_claim(REQUEST_METHOD, &request_path, idempotency_key, claim_token)
            .await
            .is_err()
        {
            tracing::error!("failed to release image upload claim");
        }

        return Err(image_storage_error(storage_error));
    }

    let completion_result = state
        .auth_repository
        .complete_asset_upload(&CompleteAssetUploadRequest {
            request_method: REQUEST_METHOD.to_owned(),
            request_path,
            idempotency_key,
            claim_token,
            room_id,
            problem_id,
            asset: asset.clone(),
            completed_at: Utc::now(),
        })
        .await;

    if let Err(error) = completion_result {
        tracing::error!(
            object_key = %object_key,
            "uploaded object was not linked to problem"
        );

        return match error {
            RepositoryError::PublishedRoomImmutable => Err(AppError::api(
                StatusCode::CONFLICT,
                "PUBLISHED_ROOM_IMMUTABLE",
                "published room cannot be modified",
            )),
            _ => Err(AppError::api(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_SERVER_ERROR",
                "failed to complete image upload",
            )),
        };
    }

    public_asset_response(&state, asset)
}

fn parse_idempotency_key(headers: &HeaderMap) -> Result<Uuid, AppError> {
    let mut values = headers.get_all("idempotency-key").iter();

    let value = values.next().ok_or_else(|| {
        AppError::api(
            StatusCode::BAD_REQUEST,
            "IDEMPOTENCY_KEY_REQUIRED",
            "Idempotency-Key header is required",
        )
    })?;

    if values.next().is_some() {
        return Err(AppError::api(
            StatusCode::BAD_REQUEST,
            "INVALID_IDEMPOTENCY_KEY",
            "multiple Idempotency-Key headers are not allowed",
        ));
    }

    let value = value.to_str().map_err(|_| {
        AppError::api(
            StatusCode::BAD_REQUEST,
            "INVALID_IDEMPOTENCY_KEY",
            "Idempotency-Key header is invalid",
        )
    })?;

    let idempotency_key = Uuid::parse_str(value).map_err(|_| {
        AppError::api(
            StatusCode::BAD_REQUEST,
            "INVALID_IDEMPOTENCY_KEY",
            "Idempotency-Key must be a UUID v4",
        )
    })?;

    if idempotency_key.get_version() != Some(Version::Random) {
        return Err(AppError::api(
            StatusCode::BAD_REQUEST,
            "INVALID_IDEMPOTENCY_KEY",
            "Idempotency-Key must be a UUID v4",
        ));
    }

    Ok(idempotency_key)
}

async fn read_multipart(mut multipart: Multipart) -> Result<(Vec<u8>, String), AppError> {
    let mut file = None;
    let mut alt = None;

    while let Some(mut field) = multipart.next_field().await.map_err(map_multipart_error)? {
        let field_name = field
            .name()
            .map(str::to_owned)
            .ok_or_else(invalid_multipart_error)?;

        match field_name.as_str() {
            "file" => {
                if file.is_some() {
                    return Err(invalid_multipart_error());
                }

                let mut bytes = Vec::new();

                while let Some(chunk) = field.chunk().await.map_err(map_multipart_error)? {
                    if bytes.len().saturating_add(chunk.len()) > MAX_IMAGE_FILE_BYTES {
                        return Err(AppError::api(
                            StatusCode::PAYLOAD_TOO_LARGE,
                            "IMAGE_TOO_LARGE",
                            "image file is too large",
                        ));
                    }

                    bytes.extend_from_slice(&chunk);
                }

                file = Some(bytes);
            }
            "alt" => {
                if alt.is_some() {
                    return Err(invalid_multipart_error());
                }

                alt = Some(field.text().await.map_err(map_alt_multipart_error)?);
            }
            _ => {
                return Err(invalid_multipart_error());
            }
        }
    }

    let file = file.ok_or_else(invalid_multipart_error)?;
    let alt = alt.ok_or_else(invalid_multipart_error)?;

    Ok((file, alt))
}

fn map_multipart_error(error: MultipartError) -> AppError {
    if error.status() == StatusCode::PAYLOAD_TOO_LARGE {
        AppError::api(
            StatusCode::PAYLOAD_TOO_LARGE,
            "IMAGE_TOO_LARGE",
            "image file is too large",
        )
    } else {
        invalid_multipart_error()
    }
}

fn map_alt_multipart_error(error: MultipartError) -> AppError {
    if error.status() == StatusCode::PAYLOAD_TOO_LARGE {
        AppError::api(
            StatusCode::UNPROCESSABLE_ENTITY,
            "INVALID_ALT",
            "image alt text is invalid",
        )
    } else {
        invalid_multipart_error()
    }
}

fn invalid_multipart_error() -> AppError {
    AppError::api(
        StatusCode::BAD_REQUEST,
        "INVALID_MULTIPART",
        "multipart body must contain one file field and one alt field",
    )
}

fn image_validation_error(error: ImageValidationError) -> AppError {
    match error {
        ImageValidationError::EmptyFile => AppError::api(
            StatusCode::UNPROCESSABLE_ENTITY,
            "EMPTY_FILE",
            "image file is empty",
        ),
        ImageValidationError::ImageTooLarge => AppError::api(
            StatusCode::PAYLOAD_TOO_LARGE,
            "IMAGE_TOO_LARGE",
            "image file is too large",
        ),
        ImageValidationError::UnsupportedImageType => AppError::api(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "UNSUPPORTED_IMAGE_TYPE",
            "image type is not supported",
        ),
        ImageValidationError::InvalidImage => AppError::api(
            StatusCode::UNPROCESSABLE_ENTITY,
            "INVALID_IMAGE",
            "image data is invalid",
        ),
        ImageValidationError::ImageDimensionsExceeded => AppError::api(
            StatusCode::UNPROCESSABLE_ENTITY,
            "IMAGE_DIMENSIONS_EXCEEDED",
            "image dimensions exceed the limit",
        ),
        ImageValidationError::InvalidAlt => AppError::api(
            StatusCode::UNPROCESSABLE_ENTITY,
            "INVALID_ALT",
            "image alt text is invalid",
        ),
    }
}

fn image_storage_error(error: ImageStorageError) -> AppError {
    match error {
        ImageStorageError::ProviderError => AppError::api(
            StatusCode::BAD_GATEWAY,
            "STORAGE_PROVIDER_ERROR",
            "storage provider rejected the upload",
        ),
        ImageStorageError::Unavailable => AppError::api(
            StatusCode::SERVICE_UNAVAILABLE,
            "STORAGE_UNAVAILABLE",
            "storage provider is unavailable",
        ),
    }
}

fn public_asset_response(
    state: &AppState,
    asset: Asset,
) -> Result<(StatusCode, Json<PublicAsset>), AppError> {
    let url = state
        .asset_url_resolver
        .resolve(&asset.object_key)
        .map_err(AppError::internal)?;

    Ok((
        StatusCode::CREATED,
        Json(PublicAsset::new(asset.asset_type, url, asset.alt)),
    ))
}
