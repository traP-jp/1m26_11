mod common;

use std::{
    convert::Infallible,
    io::Cursor,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use axum::{
    Router,
    body::{Body, Bytes},
    http::{Request, StatusCode, header},
};
use common::{StubAuthRepository, body_json, request};
use futures_util::{StreamExt, stream};
use image::{DynamicImage, ImageFormat};
use openapi_generated::models::ErrorResponse;
use serde_json::json;
use server::{
    AppState, ImageStorage, ImageStorageError, ImageStorageUpload, app,
    config::AuthMode,
    problem::{Asset, PublicBaseAssetUrlResolver},
    repository::{
        AssetUploadClaimOutcome, AssetUploadClaimRequest, AssetUploadTargetRecord, AuthProvider,
        AuthRepository, AuthUserRecord, CompleteAssetUploadRequest, RepositoryError,
    },
};
use sha2::{Digest, Sha256};
use uuid::{Uuid, Version};

const ROOM_ID: &str = "11111111-1111-4111-8111-111111111111";
const PROBLEM_ID: &str = "22222222-2222-4222-8222-222222222221";
const ASSET_ID: &str = "33333333-3333-4333-8333-333333333333";
const CLAIM_TOKEN: &str = "44444444-4444-4444-8444-444444444444";
const IDEMPOTENCY_KEY: &str = "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa";

const ALT: &str = "ろうそくが立った誕生日ケーキ";
const PUBLIC_BASE_URL: &str =
    "https://api.s3-dev.trap.jp/01960f0e-52fc-7dd5-b328-825bd88920a3-1m-26-11";

const MAX_IMAGE_FILE_BYTES: usize = 5_242_880;
const MULTIPART_HEADROOM_BYTES: usize = 64 * 1024;
const OVERSIZED_ALT_BYTES: usize = MAX_IMAGE_FILE_BYTES + MULTIPART_HEADROOM_BYTES + 1;

struct NoopImageStorage;

#[async_trait]
impl ImageStorage for NoopImageStorage {
    async fn upload(&self, _upload: ImageStorageUpload) -> Result<(), ImageStorageError> {
        Ok(())
    }
}

struct RecordingImageStorage {
    result: Result<(), ImageStorageError>,
    uploads: Mutex<Vec<ImageStorageUpload>>,
}

impl RecordingImageStorage {
    fn with_result(result: Result<(), ImageStorageError>) -> Self {
        Self {
            result,
            uploads: Mutex::new(Vec::new()),
        }
    }

    fn uploads(&self) -> Vec<ImageStorageUpload> {
        self.uploads
            .lock()
            .expect("storage upload log should not be poisoned")
            .clone()
    }
}

#[async_trait]
impl ImageStorage for RecordingImageStorage {
    async fn upload(&self, upload: ImageStorageUpload) -> Result<(), ImageStorageError> {
        self.uploads
            .lock()
            .expect("storage upload log should not be poisoned")
            .push(upload);

        self.result
    }
}

#[derive(Clone, Copy)]
enum CompleteBehavior {
    Succeed,
    PublishedRoomImmutable,
    InternalError,
}

struct UploadRepositoryState {
    target: Option<AssetUploadTargetRecord>,
    claim_outcome: AssetUploadClaimOutcome,
    complete_behavior: CompleteBehavior,
    target_calls: Vec<(Uuid, Uuid)>,
    claim_requests: Vec<AssetUploadClaimRequest>,
    complete_requests: Vec<CompleteAssetUploadRequest>,
    release_calls: Vec<(String, String, Uuid, Uuid)>,
}

struct RecordingUploadRepository {
    state: Mutex<UploadRepositoryState>,
}

impl RecordingUploadRepository {
    fn new(claim_outcome: AssetUploadClaimOutcome) -> Self {
        Self {
            state: Mutex::new(UploadRepositoryState {
                target: Some(AssetUploadTargetRecord {
                    is_published: false,
                }),
                claim_outcome,
                complete_behavior: CompleteBehavior::Succeed,
                target_calls: Vec::new(),
                claim_requests: Vec::new(),
                complete_requests: Vec::new(),
                release_calls: Vec::new(),
            }),
        }
    }

    fn set_target(&self, target: Option<AssetUploadTargetRecord>) {
        self.state
            .lock()
            .expect("repository state should not be poisoned")
            .target = target;
    }

    fn set_complete_behavior(&self, behavior: CompleteBehavior) {
        self.state
            .lock()
            .expect("repository state should not be poisoned")
            .complete_behavior = behavior;
    }

    fn target_calls(&self) -> Vec<(Uuid, Uuid)> {
        self.state
            .lock()
            .expect("repository state should not be poisoned")
            .target_calls
            .clone()
    }

    fn claim_requests(&self) -> Vec<AssetUploadClaimRequest> {
        self.state
            .lock()
            .expect("repository state should not be poisoned")
            .claim_requests
            .clone()
    }

    fn complete_requests(&self) -> Vec<CompleteAssetUploadRequest> {
        self.state
            .lock()
            .expect("repository state should not be poisoned")
            .complete_requests
            .clone()
    }

    fn release_calls(&self) -> Vec<(String, String, Uuid, Uuid)> {
        self.state
            .lock()
            .expect("repository state should not be poisoned")
            .release_calls
            .clone()
    }
}

#[async_trait]
impl AuthRepository for RecordingUploadRepository {
    async fn find_user_by_demo_session(
        &self,
        _session_id: Uuid,
    ) -> Result<Option<AuthUserRecord>, RepositoryError> {
        Ok(None)
    }

    async fn find_user_by_provider_subject(
        &self,
        _auth_provider: AuthProvider,
        _provider_subject: &str,
    ) -> Result<Option<AuthUserRecord>, RepositoryError> {
        Ok(None)
    }

    async fn get_or_create_user(
        &self,
        auth_provider: AuthProvider,
        _provider_subject: &str,
        display_name: &str,
    ) -> Result<AuthUserRecord, RepositoryError> {
        Ok(AuthUserRecord {
            user_id: Uuid::nil(),
            display_name: display_name.to_owned(),
            auth_provider,
        })
    }

    async fn find_asset_upload_target(
        &self,
        room_id: Uuid,
        problem_id: Uuid,
    ) -> Result<Option<AssetUploadTargetRecord>, RepositoryError> {
        let mut state = self
            .state
            .lock()
            .expect("repository state should not be poisoned");

        state.target_calls.push((room_id, problem_id));

        Ok(state.target)
    }

    async fn claim_asset_upload(
        &self,
        request: &AssetUploadClaimRequest,
    ) -> Result<AssetUploadClaimOutcome, RepositoryError> {
        let mut state = self
            .state
            .lock()
            .expect("repository state should not be poisoned");

        state.claim_requests.push(request.clone());

        Ok(state.claim_outcome.clone())
    }

    async fn complete_asset_upload(
        &self,
        request: &CompleteAssetUploadRequest,
    ) -> Result<(), RepositoryError> {
        let mut state = self
            .state
            .lock()
            .expect("repository state should not be poisoned");

        state.complete_requests.push(request.clone());

        match state.complete_behavior {
            CompleteBehavior::Succeed => Ok(()),
            CompleteBehavior::PublishedRoomImmutable => {
                Err(RepositoryError::PublishedRoomImmutable)
            }
            CompleteBehavior::InternalError => Err(RepositoryError::AssetUploadTargetChanged),
        }
    }

    async fn release_asset_upload_claim(
        &self,
        request_method: &str,
        request_path: &str,
        idempotency_key: Uuid,
        claim_token: Uuid,
    ) -> Result<(), RepositoryError> {
        self.state
            .lock()
            .expect("repository state should not be poisoned")
            .release_calls
            .push((
                request_method.to_owned(),
                request_path.to_owned(),
                idempotency_key,
                claim_token,
            ));

        Ok(())
    }
}

fn parse_uuid(value: &str) -> Uuid {
    Uuid::parse_str(value).expect("test UUID should be valid")
}

fn upload_path() -> String {
    format!("/api/rooms/{ROOM_ID}/problems/{PROBLEM_ID}/assets")
}

fn completed_asset() -> Asset {
    Asset {
        asset_type: "image".to_owned(),
        object_key: format!("v1/problems/{ROOM_ID}/{PROBLEM_ID}/{ASSET_ID}.png"),
        alt: ALT.to_owned(),
    }
}

fn upload_test_app(auth_mode: AuthMode, storage_configured: bool) -> Router {
    let mut state = AppState::new(auth_mode, Arc::new(StubAuthRepository));

    if storage_configured {
        state = state.with_image_storage(Arc::new(NoopImageStorage));
    }

    app(state)
}

fn recording_upload_test_app(
    claim_outcome: AssetUploadClaimOutcome,
) -> (
    Router,
    Arc<RecordingUploadRepository>,
    Arc<RecordingImageStorage>,
) {
    recording_upload_test_app_with_storage_result(claim_outcome, Ok(()))
}

fn recording_upload_test_app_with_storage_result(
    claim_outcome: AssetUploadClaimOutcome,
    storage_result: Result<(), ImageStorageError>,
) -> (
    Router,
    Arc<RecordingUploadRepository>,
    Arc<RecordingImageStorage>,
) {
    let repository = Arc::new(RecordingUploadRepository::new(claim_outcome));
    let storage = Arc::new(RecordingImageStorage::with_result(storage_result));

    let state = AppState::new(AuthMode::Demo, repository.clone())
        .with_image_storage(storage.clone())
        .with_asset_url_resolver(Arc::new(PublicBaseAssetUrlResolver::new(PUBLIC_BASE_URL)));

    (app(state), repository, storage)
}

fn empty_upload_request() -> Request<Body> {
    Request::post(upload_path())
        .body(Body::empty())
        .expect("upload request should be valid")
}

fn encoded_image(width: u32, height: u32, format: ImageFormat) -> Vec<u8> {
    let image = DynamicImage::new_rgb8(width, height);
    let mut output = Cursor::new(Vec::new());

    image
        .write_to(&mut output, format)
        .expect("test image should be encoded");

    output.into_inner()
}

fn valid_png() -> Vec<u8> {
    encoded_image(1, 1, ImageFormat::Png)
}

fn multipart_body(fields: &[(&str, &[u8])]) -> Vec<u8> {
    const BOUNDARY: &str = "image-upload-valid-boundary";

    let mut body = Vec::new();

    for (name, value) in fields {
        body.extend_from_slice(
            format!(
                "--{BOUNDARY}\r\n\
                 Content-Disposition: form-data; name=\"{name}\""
            )
            .as_bytes(),
        );

        if *name == "file" {
            body.extend_from_slice(
                b"; filename=\"image.png\"\r\n\
                  Content-Type: image/png",
            );
        }

        body.extend_from_slice(b"\r\n\r\n");
        body.extend_from_slice(value);
        body.extend_from_slice(b"\r\n");
    }

    body.extend_from_slice(format!("--{BOUNDARY}--\r\n").as_bytes());

    body
}

fn multipart_request(
    path: &str,
    idempotency_keys: &[&str],
    fields: &[(&str, &[u8])],
) -> Request<Body> {
    const BOUNDARY: &str = "image-upload-valid-boundary";

    let mut builder = Request::post(path).header(
        header::CONTENT_TYPE,
        format!("multipart/form-data; boundary={BOUNDARY}"),
    );

    for idempotency_key in idempotency_keys {
        builder = builder.header("idempotency-key", *idempotency_key);
    }

    builder
        .body(Body::from(multipart_body(fields)))
        .expect("upload request should be valid")
}

fn multipart_upload_request(file: &[u8], alt: &str) -> Request<Body> {
    multipart_request(
        &upload_path(),
        &[IDEMPOTENCY_KEY],
        &[("file", file), ("alt", alt.as_bytes())],
    )
}

fn oversized_alt_request() -> Request<Body> {
    const BOUNDARY: &str = "image-upload-test-boundary";
    const CHUNK_BYTES: usize = 16 * 1024;

    let prefix = format!(
        "--{BOUNDARY}\r\n\
         Content-Disposition: form-data; name=\"alt\"\r\n\
         \r\n"
    );
    let suffix = format!("\r\n--{BOUNDARY}--\r\n");

    let mut chunks: Vec<Result<Bytes, Infallible>> = Vec::new();
    chunks.push(Ok(Bytes::from(prefix)));

    let mut remaining = OVERSIZED_ALT_BYTES;

    while remaining > 0 {
        let chunk_length = remaining.min(CHUNK_BYTES);
        chunks.push(Ok(Bytes::from(vec![b'a'; chunk_length])));
        remaining -= chunk_length;
    }

    chunks.push(Ok(Bytes::from(suffix)));

    let body_stream = stream::iter(chunks).then(|chunk| async move {
        tokio::task::yield_now().await;
        chunk
    });

    Request::post(upload_path())
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={BOUNDARY}"),
        )
        .header("idempotency-key", IDEMPOTENCY_KEY)
        .body(Body::from_stream(body_stream))
        .expect("upload request should be valid")
}

async fn assert_error_response(
    response: axum::response::Response,
    expected_status: StatusCode,
    expected_code: &str,
) {
    assert_eq!(response.status(), expected_status);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");

    let body: ErrorResponse = body_json(response).await;

    assert_eq!(body.error.code, expected_code);
    assert!(!body.error.message.is_empty());
    assert_eq!(body.error.details.0, json!({}));
}

async fn assert_rejected_before_repository(
    upload_request: Request<Body>,
    expected_status: StatusCode,
    expected_code: &str,
) {
    let claim_token = parse_uuid(CLAIM_TOKEN);
    let (app, repository, storage) =
        recording_upload_test_app(AssetUploadClaimOutcome::Acquired { claim_token });

    let response = request(&app, upload_request).await;

    assert_error_response(response, expected_status, expected_code).await;

    assert!(repository.target_calls().is_empty());
    assert!(repository.claim_requests().is_empty());
    assert!(repository.complete_requests().is_empty());
    assert!(repository.release_calls().is_empty());
    assert!(storage.uploads().is_empty());
}

#[tokio::test]
async fn upload_route_is_registered_in_demo_mode_with_storage() {
    let app = upload_test_app(AuthMode::Demo, true);

    let response = request(&app, empty_upload_request()).await;

    assert_error_response(
        response,
        StatusCode::BAD_REQUEST,
        "IDEMPOTENCY_KEY_REQUIRED",
    )
    .await;
}

#[tokio::test]
async fn upload_route_is_not_registered_without_storage() {
    let app = upload_test_app(AuthMode::Demo, false);

    let response = request(&app, empty_upload_request()).await;

    assert_error_response(response, StatusCode::NOT_FOUND, "NOT_FOUND").await;
}

#[tokio::test]
async fn upload_route_is_not_registered_in_neoshowcase_mode() {
    let app = upload_test_app(AuthMode::NeoShowcase, true);

    let response = request(&app, empty_upload_request()).await;

    assert_error_response(response, StatusCode::NOT_FOUND, "NOT_FOUND").await;
}

#[tokio::test]
async fn oversized_alt_returns_invalid_alt_instead_of_image_too_large() {
    let app = upload_test_app(AuthMode::Demo, true);

    let response = request(&app, oversized_alt_request()).await;

    assert_error_response(response, StatusCode::UNPROCESSABLE_ENTITY, "INVALID_ALT").await;
}

#[tokio::test]
async fn upload_succeeds_and_records_storage_and_repository_calls() {
    let claim_token = parse_uuid(CLAIM_TOKEN);
    let (app, repository, storage) =
        recording_upload_test_app(AssetUploadClaimOutcome::Acquired { claim_token });
    let file = valid_png();

    let response = request(&app, multipart_upload_request(&file, &format!("  {ALT}  "))).await;

    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");

    let uploads = storage.uploads();
    assert_eq!(uploads.len(), 1);

    let upload = &uploads[0];
    assert_eq!(upload.bytes.as_slice(), file.as_slice());
    assert_eq!(upload.content_type, "image/png");

    let object_key_prefix = format!("v1/problems/{ROOM_ID}/{PROBLEM_ID}/");
    let asset_id = upload
        .object_key
        .strip_prefix(&object_key_prefix)
        .and_then(|value| value.strip_suffix(".png"))
        .expect("object key should use the expected namespace");

    let asset_id = Uuid::parse_str(asset_id).expect("object key should contain a UUID");
    assert_eq!(asset_id.get_version(), Some(Version::Random));

    let actual: serde_json::Value = body_json(response).await;
    assert_eq!(
        actual,
        json!({
            "type": "image",
            "url": format!("{PUBLIC_BASE_URL}/{}", upload.object_key),
            "alt": ALT,
        })
    );
    assert!(
        actual.get("object_key").is_none(),
        "public response must not expose object_key"
    );

    assert_eq!(
        repository.target_calls(),
        vec![(parse_uuid(ROOM_ID), parse_uuid(PROBLEM_ID))]
    );

    let claim_requests = repository.claim_requests();
    assert_eq!(claim_requests.len(), 1);

    let claim = &claim_requests[0];
    assert_eq!(claim.request_method, "POST");
    assert_eq!(claim.request_path, upload_path());
    assert_eq!(claim.idempotency_key, parse_uuid(IDEMPOTENCY_KEY));
    assert_eq!(claim.alt, ALT);

    let expected_sha256: [u8; 32] = Sha256::digest(&file).into();
    assert_eq!(claim.file_sha256, expected_sha256);

    let complete_requests = repository.complete_requests();
    assert_eq!(complete_requests.len(), 1);

    let completion = &complete_requests[0];
    assert_eq!(completion.request_method, "POST");
    assert_eq!(completion.request_path, upload_path());
    assert_eq!(completion.idempotency_key, parse_uuid(IDEMPOTENCY_KEY));
    assert_eq!(completion.claim_token, claim_token);
    assert_eq!(completion.room_id, parse_uuid(ROOM_ID));
    assert_eq!(completion.problem_id, parse_uuid(PROBLEM_ID));
    assert_eq!(completion.asset.asset_type, "image");
    assert_eq!(completion.asset.object_key, upload.object_key);
    assert_eq!(completion.asset.alt, ALT);

    assert!(repository.release_calls().is_empty());
}

#[tokio::test]
async fn completed_idempotent_retry_matches_success_fixture() {
    let (app, repository, storage) =
        recording_upload_test_app(AssetUploadClaimOutcome::Completed {
            asset: completed_asset(),
        });
    let file = valid_png();

    let response = request(&app, multipart_upload_request(&file, ALT)).await;

    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");

    let actual: serde_json::Value = body_json(response).await;
    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../openapi/examples/assets/response-created.json"
    ))
    .expect("OpenAPI success fixture should be valid JSON");

    assert_eq!(actual, expected);
    assert_eq!(repository.claim_requests().len(), 1);
    assert!(repository.complete_requests().is_empty());
    assert!(repository.release_calls().is_empty());
    assert!(storage.uploads().is_empty());
}

async fn assert_idempotency_error(outcome: AssetUploadClaimOutcome, expected_code: &str) {
    let (app, repository, storage) = recording_upload_test_app(outcome);
    let file = valid_png();

    let response = request(&app, multipart_upload_request(&file, ALT)).await;

    assert_error_response(response, StatusCode::CONFLICT, expected_code).await;

    assert_eq!(repository.claim_requests().len(), 1);
    assert!(repository.complete_requests().is_empty());
    assert!(repository.release_calls().is_empty());
    assert!(storage.uploads().is_empty());
}

#[tokio::test]
async fn reused_idempotency_key_returns_conflict() {
    assert_idempotency_error(AssetUploadClaimOutcome::Reused, "IDEMPOTENCY_KEY_REUSED").await;
}

#[tokio::test]
async fn in_progress_idempotency_request_returns_conflict() {
    assert_idempotency_error(
        AssetUploadClaimOutcome::InProgress,
        "IDEMPOTENCY_REQUEST_IN_PROGRESS",
    )
    .await;
}

#[tokio::test]
async fn invalid_path_parameters_are_rejected() {
    let file = valid_png();

    let invalid_paths = [
        format!("/api/rooms/not-a-uuid/problems/{PROBLEM_ID}/assets"),
        format!("/api/rooms/{ROOM_ID}/problems/not-a-uuid/assets"),
    ];

    for path in invalid_paths {
        let upload_request = multipart_request(
            &path,
            &[IDEMPOTENCY_KEY],
            &[("file", file.as_slice()), ("alt", ALT.as_bytes())],
        );

        assert_rejected_before_repository(
            upload_request,
            StatusCode::BAD_REQUEST,
            "INVALID_PATH_PARAMETER",
        )
        .await;
    }
}

#[tokio::test]
async fn invalid_idempotency_headers_are_rejected() {
    let file = valid_png();
    let path = upload_path();

    let missing = multipart_request(
        &path,
        &[],
        &[("file", file.as_slice()), ("alt", ALT.as_bytes())],
    );
    assert_rejected_before_repository(missing, StatusCode::BAD_REQUEST, "IDEMPOTENCY_KEY_REQUIRED")
        .await;

    let malformed = multipart_request(
        &path,
        &["not-a-uuid"],
        &[("file", file.as_slice()), ("alt", ALT.as_bytes())],
    );
    assert_rejected_before_repository(
        malformed,
        StatusCode::BAD_REQUEST,
        "INVALID_IDEMPOTENCY_KEY",
    )
    .await;

    let non_v4 = multipart_request(
        &path,
        &["11111111-1111-1111-8111-111111111111"],
        &[("file", file.as_slice()), ("alt", ALT.as_bytes())],
    );
    assert_rejected_before_repository(non_v4, StatusCode::BAD_REQUEST, "INVALID_IDEMPOTENCY_KEY")
        .await;

    let duplicated = multipart_request(
        &path,
        &[IDEMPOTENCY_KEY, IDEMPOTENCY_KEY],
        &[("file", file.as_slice()), ("alt", ALT.as_bytes())],
    );
    assert_rejected_before_repository(
        duplicated,
        StatusCode::BAD_REQUEST,
        "INVALID_IDEMPOTENCY_KEY",
    )
    .await;
}

#[tokio::test]
async fn invalid_multipart_requests_are_rejected() {
    let file = valid_png();
    let path = upload_path();

    let missing_content_type = Request::post(&path)
        .header("idempotency-key", IDEMPOTENCY_KEY)
        .body(Body::empty())
        .expect("upload request should be valid");
    assert_rejected_before_repository(
        missing_content_type,
        StatusCode::BAD_REQUEST,
        "INVALID_MULTIPART",
    )
    .await;

    let invalid_content_type = Request::post(&path)
        .header(header::CONTENT_TYPE, "application/json")
        .header("idempotency-key", IDEMPOTENCY_KEY)
        .body(Body::from("{}"))
        .expect("upload request should be valid");
    assert_rejected_before_repository(
        invalid_content_type,
        StatusCode::BAD_REQUEST,
        "INVALID_MULTIPART",
    )
    .await;

    let missing_file = multipart_request(&path, &[IDEMPOTENCY_KEY], &[("alt", ALT.as_bytes())]);
    assert_rejected_before_repository(missing_file, StatusCode::BAD_REQUEST, "INVALID_MULTIPART")
        .await;

    let missing_alt = multipart_request(&path, &[IDEMPOTENCY_KEY], &[("file", file.as_slice())]);
    assert_rejected_before_repository(missing_alt, StatusCode::BAD_REQUEST, "INVALID_MULTIPART")
        .await;

    let duplicate_file = multipart_request(
        &path,
        &[IDEMPOTENCY_KEY],
        &[
            ("file", file.as_slice()),
            ("file", file.as_slice()),
            ("alt", ALT.as_bytes()),
        ],
    );
    assert_rejected_before_repository(duplicate_file, StatusCode::BAD_REQUEST, "INVALID_MULTIPART")
        .await;

    let duplicate_alt = multipart_request(
        &path,
        &[IDEMPOTENCY_KEY],
        &[
            ("file", file.as_slice()),
            ("alt", ALT.as_bytes()),
            ("alt", ALT.as_bytes()),
        ],
    );
    assert_rejected_before_repository(duplicate_alt, StatusCode::BAD_REQUEST, "INVALID_MULTIPART")
        .await;

    let unknown_field = multipart_request(
        &path,
        &[IDEMPOTENCY_KEY],
        &[
            ("file", file.as_slice()),
            ("alt", ALT.as_bytes()),
            ("unexpected", b"value"),
        ],
    );
    assert_rejected_before_repository(unknown_field, StatusCode::BAD_REQUEST, "INVALID_MULTIPART")
        .await;
}

#[tokio::test]
async fn invalid_image_and_alt_values_are_rejected() {
    let empty_file = Vec::new();
    assert_rejected_before_repository(
        multipart_upload_request(&empty_file, ALT),
        StatusCode::UNPROCESSABLE_ENTITY,
        "EMPTY_FILE",
    )
    .await;

    let oversized_file = vec![0_u8; MAX_IMAGE_FILE_BYTES + 1];
    assert_rejected_before_repository(
        multipart_upload_request(&oversized_file, ALT),
        StatusCode::PAYLOAD_TOO_LARGE,
        "IMAGE_TOO_LARGE",
    )
    .await;

    let unsupported_file = b"this is not an image";
    assert_rejected_before_repository(
        multipart_upload_request(unsupported_file, ALT),
        StatusCode::UNSUPPORTED_MEDIA_TYPE,
        "UNSUPPORTED_IMAGE_TYPE",
    )
    .await;

    let mut broken_png = valid_png();
    broken_png.truncate(20);
    assert_rejected_before_repository(
        multipart_upload_request(&broken_png, ALT),
        StatusCode::UNPROCESSABLE_ENTITY,
        "INVALID_IMAGE",
    )
    .await;

    let oversized_dimensions = encoded_image(4_097, 1, ImageFormat::Png);
    assert_rejected_before_repository(
        multipart_upload_request(&oversized_dimensions, ALT),
        StatusCode::UNPROCESSABLE_ENTITY,
        "IMAGE_DIMENSIONS_EXCEEDED",
    )
    .await;

    let valid_file = valid_png();

    assert_rejected_before_repository(
        multipart_upload_request(&valid_file, "   "),
        StatusCode::UNPROCESSABLE_ENTITY,
        "INVALID_ALT",
    )
    .await;

    let too_long_alt = "あ".repeat(201);
    assert_rejected_before_repository(
        multipart_upload_request(&valid_file, &too_long_alt),
        StatusCode::UNPROCESSABLE_ENTITY,
        "INVALID_ALT",
    )
    .await;
}

#[tokio::test]
async fn missing_upload_target_returns_not_found() {
    let claim_token = parse_uuid(CLAIM_TOKEN);
    let (app, repository, storage) =
        recording_upload_test_app(AssetUploadClaimOutcome::Acquired { claim_token });
    repository.set_target(None);

    let file = valid_png();
    let response = request(&app, multipart_upload_request(&file, ALT)).await;

    assert_error_response(response, StatusCode::NOT_FOUND, "ROOM_OR_PROBLEM_NOT_FOUND").await;

    assert_eq!(
        repository.target_calls(),
        vec![(parse_uuid(ROOM_ID), parse_uuid(PROBLEM_ID))]
    );
    assert!(repository.claim_requests().is_empty());
    assert!(repository.complete_requests().is_empty());
    assert!(repository.release_calls().is_empty());
    assert!(storage.uploads().is_empty());
}

#[tokio::test]
async fn published_room_returns_conflict_before_claim() {
    let claim_token = parse_uuid(CLAIM_TOKEN);
    let (app, repository, storage) =
        recording_upload_test_app(AssetUploadClaimOutcome::Acquired { claim_token });
    repository.set_target(Some(AssetUploadTargetRecord { is_published: true }));

    let file = valid_png();
    let response = request(&app, multipart_upload_request(&file, ALT)).await;

    assert_error_response(response, StatusCode::CONFLICT, "PUBLISHED_ROOM_IMMUTABLE").await;

    assert_eq!(
        repository.target_calls(),
        vec![(parse_uuid(ROOM_ID), parse_uuid(PROBLEM_ID))]
    );
    assert!(repository.claim_requests().is_empty());
    assert!(repository.complete_requests().is_empty());
    assert!(repository.release_calls().is_empty());
    assert!(storage.uploads().is_empty());
}

async fn assert_storage_failure(
    storage_error: ImageStorageError,
    expected_status: StatusCode,
    expected_code: &str,
) {
    let claim_token = parse_uuid(CLAIM_TOKEN);
    let (app, repository, storage) = recording_upload_test_app_with_storage_result(
        AssetUploadClaimOutcome::Acquired { claim_token },
        Err(storage_error),
    );

    let file = valid_png();
    let response = request(&app, multipart_upload_request(&file, ALT)).await;

    assert_error_response(response, expected_status, expected_code).await;

    assert_eq!(storage.uploads().len(), 1);
    assert_eq!(repository.claim_requests().len(), 1);
    assert!(repository.complete_requests().is_empty());
    assert_eq!(
        repository.release_calls(),
        vec![(
            "POST".to_owned(),
            upload_path(),
            parse_uuid(IDEMPOTENCY_KEY),
            claim_token,
        )]
    );
}

#[tokio::test]
async fn provider_error_returns_502_and_releases_claim() {
    assert_storage_failure(
        ImageStorageError::ProviderError,
        StatusCode::BAD_GATEWAY,
        "STORAGE_PROVIDER_ERROR",
    )
    .await;
}

#[tokio::test]
async fn unavailable_storage_returns_503_and_releases_claim() {
    assert_storage_failure(
        ImageStorageError::Unavailable,
        StatusCode::SERVICE_UNAVAILABLE,
        "STORAGE_UNAVAILABLE",
    )
    .await;
}

async fn assert_completion_failure(
    behavior: CompleteBehavior,
    expected_status: StatusCode,
    expected_code: &str,
) {
    let claim_token = parse_uuid(CLAIM_TOKEN);
    let (app, repository, storage) =
        recording_upload_test_app(AssetUploadClaimOutcome::Acquired { claim_token });
    repository.set_complete_behavior(behavior);

    let file = valid_png();
    let response = request(&app, multipart_upload_request(&file, ALT)).await;

    assert_error_response(response, expected_status, expected_code).await;

    assert_eq!(storage.uploads().len(), 1);
    assert_eq!(repository.claim_requests().len(), 1);
    assert_eq!(repository.complete_requests().len(), 1);

    assert!(
        repository.release_calls().is_empty(),
        "claim must remain after storage upload succeeds"
    );
}

#[tokio::test]
async fn completion_database_error_does_not_release_claim() {
    assert_completion_failure(
        CompleteBehavior::InternalError,
        StatusCode::INTERNAL_SERVER_ERROR,
        "INTERNAL_SERVER_ERROR",
    )
    .await;
}

#[tokio::test]
async fn completion_publish_race_does_not_release_claim() {
    assert_completion_failure(
        CompleteBehavior::PublishedRoomImmutable,
        StatusCode::CONFLICT,
        "PUBLISHED_ROOM_IMMUTABLE",
    )
    .await;
}
