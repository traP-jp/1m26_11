mod common;

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use async_trait::async_trait;
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
};
use chrono::Utc;
use common::{MOCK_SESSION_ID, body_json, problem_detail_record, request};
use openapi_generated::models::ErrorResponse;
use serde_json::json;
use server::{
    AppState, ImageUrlSigner, ImageUrlSigningError, app,
    config::AuthMode,
    problem::Asset,
    repository::{
        AuthProvider, AuthRepository, AuthUserRecord, ProblemDetailRecord, RepositoryError,
        RunRecord,
    },
};
use uuid::Uuid;

const ROOM_ID: &str = "11111111-1111-4111-8111-111111111111";
const PROBLEM_ID: &str = "22222222-2222-4222-8222-222222222221";
const LOCKED_PROBLEM_ID: &str = "22222222-2222-4222-8222-222222222222";
const MISSING_PROBLEM_ID: &str = "99999999-9999-4999-8999-999999999999";
const MISSING_RUN_ROOM_ID: &str = "88888888-8888-4888-8888-888888888888";
const RUN_ID: &str = "77777777-7777-4777-8777-777777777777";

const FIRST_OBJECT_KEY: &str = concat!(
    "v1/problems/",
    "11111111-1111-4111-8111-111111111111/",
    "22222222-2222-4222-8222-222222222221/",
    "33333333-3333-4333-8333-333333333333.png"
);
const SECOND_OBJECT_KEY: &str = concat!(
    "v1/problems/",
    "11111111-1111-4111-8111-111111111111/",
    "22222222-2222-4222-8222-222222222221/",
    "55555555-5555-4555-8555-555555555555.webp"
);

const FIRST_PRESIGNED_URL: &str = concat!(
    "https://storage.example.invalid/example-bucket/",
    "v1/problems/",
    "11111111-1111-4111-8111-111111111111/",
    "22222222-2222-4222-8222-222222222221/",
    "33333333-3333-4333-8333-333333333333.png",
    "?X-Amz-Expires=300&X-Amz-Signature=example-signature-1"
);
const SECOND_PRESIGNED_URL: &str = concat!(
    "https://storage.example.invalid/example-bucket/",
    "v1/problems/",
    "11111111-1111-4111-8111-111111111111/",
    "22222222-2222-4222-8222-222222222221/",
    "55555555-5555-4555-8555-555555555555.webp",
    "?X-Amz-Expires=300&X-Amz-Signature=example-signature-2"
);

#[derive(Clone, Copy)]
enum DownloadBehavior {
    WithAssets,
    EmptyAssets,
    MissingProblem,
    LockedProblem,
    DatabaseError,
}

struct DownloadRepository {
    behavior: DownloadBehavior,
}

#[async_trait]
impl AuthRepository for DownloadRepository {
    async fn find_user_by_demo_session(
        &self,
        session_id: Uuid,
    ) -> Result<Option<AuthUserRecord>, RepositoryError> {
        assert_eq!(session_id, parse_uuid(MOCK_SESSION_ID));

        Ok(Some(AuthUserRecord {
            user_id: parse_uuid(MOCK_SESSION_ID),
            display_name: "test-user".to_owned(),
            auth_provider: AuthProvider::Demo,
        }))
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
            user_id: parse_uuid(MOCK_SESSION_ID),
            display_name: display_name.to_owned(),
            auth_provider,
        })
    }

    async fn find_active_run(
        &self,
        user_id: Uuid,
        room_id: Uuid,
    ) -> Result<Option<RunRecord>, RepositoryError> {
        if room_id != parse_uuid(ROOM_ID) {
            return Ok(None);
        }

        Ok(Some(RunRecord {
            id: parse_uuid(RUN_ID),
            user_id,
            room_id,
            status: "active".to_owned(),
            started_at: Utc::now(),
            cleared_at: None,
        }))
    }

    async fn find_problem_for_run(
        &self,
        run_id: Uuid,
        room_id: Uuid,
        problem_id: Uuid,
    ) -> Result<Option<ProblemDetailRecord>, RepositoryError> {
        if run_id != parse_uuid(RUN_ID) || room_id != parse_uuid(ROOM_ID) {
            return Ok(None);
        }

        match self.behavior {
            DownloadBehavior::DatabaseError => {
                return Err(RepositoryError::Database(sqlx::Error::Protocol(
                    "simulated private database failure".to_owned(),
                )));
            }
            DownloadBehavior::MissingProblem => return Ok(None),
            DownloadBehavior::WithAssets
            | DownloadBehavior::EmptyAssets
            | DownloadBehavior::LockedProblem => {}
        }

        if problem_id == parse_uuid(MISSING_PROBLEM_ID) {
            return Ok(None);
        }

        let status = match self.behavior {
            DownloadBehavior::LockedProblem => "locked",
            _ => "available",
        };

        let mut problem = problem_detail_record(problem_id, status);

        problem.assets.0 = match self.behavior {
            DownloadBehavior::EmptyAssets => Vec::new(),
            _ => vec![
                Asset {
                    asset_type: "image".to_owned(),
                    object_key: FIRST_OBJECT_KEY.to_owned(),
                    alt: "ろうそくが立った誕生日ケーキ".to_owned(),
                },
                Asset {
                    asset_type: "image".to_owned(),
                    object_key: SECOND_OBJECT_KEY.to_owned(),
                    alt: "問題を解くための補足画像".to_owned(),
                },
            ],
        };

        Ok(Some(problem))
    }
}

struct RecordingImageUrlSigner {
    result: Result<(), ImageUrlSigningError>,
    calls: Mutex<Vec<(String, Duration)>>,
}

impl RecordingImageUrlSigner {
    fn new(result: Result<(), ImageUrlSigningError>) -> Self {
        Self {
            result,
            calls: Mutex::new(Vec::new()),
        }
    }

    fn calls(&self) -> Vec<(String, Duration)> {
        self.calls
            .lock()
            .expect("signer call log should not be poisoned")
            .clone()
    }
}

#[async_trait]
impl ImageUrlSigner for RecordingImageUrlSigner {
    async fn presign_get(
        &self,
        object_key: &str,
        expires_in: Duration,
    ) -> Result<String, ImageUrlSigningError> {
        self.calls
            .lock()
            .expect("signer call log should not be poisoned")
            .push((object_key.to_owned(), expires_in));

        self.result?;

        match object_key {
            FIRST_OBJECT_KEY => Ok(FIRST_PRESIGNED_URL.to_owned()),
            SECOND_OBJECT_KEY => Ok(SECOND_PRESIGNED_URL.to_owned()),
            _ => panic!("unexpected object key was passed to signer"),
        }
    }
}

fn parse_uuid(value: &str) -> Uuid {
    Uuid::parse_str(value).expect("test UUID should be valid")
}

fn download_path(room_id: &str, problem_id: &str) -> String {
    format!("/api/rooms/{room_id}/problems/{problem_id}/assets")
}

fn download_request(room_id: &str, problem_id: &str, authenticated: bool) -> Request<Body> {
    let mut builder = Request::get(download_path(room_id, problem_id));

    if authenticated {
        builder = builder.header(header::COOKIE, format!("demo_session={MOCK_SESSION_ID}"));
    }

    builder
        .body(Body::empty())
        .expect("download request should be valid")
}

fn download_test_app(
    auth_mode: AuthMode,
    behavior: DownloadBehavior,
    signing_result: Result<(), ImageUrlSigningError>,
) -> (Router, Arc<RecordingImageUrlSigner>) {
    let repository = Arc::new(DownloadRepository { behavior });
    let signer = Arc::new(RecordingImageUrlSigner::new(signing_result));

    let state = AppState::new(auth_mode, repository).with_image_url_signer(signer.clone());

    (app(state), signer)
}

async fn assert_fixture_response(
    response: axum::response::Response,
    expected_status: StatusCode,
    fixture: &str,
) {
    assert_eq!(response.status(), expected_status);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");

    let actual: serde_json::Value = body_json(response).await;
    let expected: serde_json::Value =
        serde_json::from_str(fixture).expect("OpenAPI fixture should be valid JSON");

    assert_eq!(actual, expected);
}

async fn assert_error_code(
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

#[tokio::test]
async fn download_route_is_not_registered_without_signer() {
    let state = AppState::new(
        AuthMode::Demo,
        Arc::new(DownloadRepository {
            behavior: DownloadBehavior::WithAssets,
        }),
    );
    let app = app(state);

    let response = request(&app, download_request(ROOM_ID, PROBLEM_ID, true)).await;

    assert_error_code(response, StatusCode::NOT_FOUND, "NOT_FOUND").await;
}

#[tokio::test]
async fn download_requires_authentication_in_both_auth_modes() {
    for auth_mode in [AuthMode::Demo, AuthMode::NeoShowcase] {
        let (app, signer) = download_test_app(auth_mode, DownloadBehavior::WithAssets, Ok(()));

        let response = request(
            &app,
            Request::get(download_path(ROOM_ID, PROBLEM_ID))
                .body(Body::empty())
                .expect("download request should be valid"),
        )
        .await;

        assert_fixture_response(
            response,
            StatusCode::UNAUTHORIZED,
            include_str!("../../openapi/examples/auth/error-unauthorized.json"),
        )
        .await;

        assert!(signer.calls().is_empty());
    }
}

#[tokio::test]
async fn download_returns_presigned_urls_from_registered_object_keys() {
    let (app, signer) = download_test_app(AuthMode::Demo, DownloadBehavior::WithAssets, Ok(()));

    let response = request(&app, download_request(ROOM_ID, PROBLEM_ID, true)).await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");

    let actual: serde_json::Value = body_json(response).await;
    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../openapi/examples/assets/response-list.json"
    ))
    .expect("OpenAPI success fixture should be valid JSON");

    assert_eq!(actual, expected);
    assert_eq!(
        signer.calls(),
        vec![
            (FIRST_OBJECT_KEY.to_owned(), Duration::from_secs(300),),
            (SECOND_OBJECT_KEY.to_owned(), Duration::from_secs(300),),
        ]
    );
}

#[tokio::test]
async fn download_rejects_invalid_path_parameters() {
    let invalid_paths = [
        download_path("not-a-uuid", PROBLEM_ID),
        download_path(ROOM_ID, "not-a-uuid"),
    ];

    for path in invalid_paths {
        let (app, signer) = download_test_app(AuthMode::Demo, DownloadBehavior::WithAssets, Ok(()));

        let response = request(
            &app,
            Request::get(path)
                .header(header::COOKIE, format!("demo_session={MOCK_SESSION_ID}"))
                .body(Body::empty())
                .expect("download request should be valid"),
        )
        .await;

        assert_error_code(response, StatusCode::BAD_REQUEST, "INVALID_PATH_PARAMETER").await;

        assert!(signer.calls().is_empty());
    }
}

#[tokio::test]
async fn download_without_active_run_matches_fixture() {
    let (app, signer) = download_test_app(AuthMode::Demo, DownloadBehavior::WithAssets, Ok(()));

    let response = request(
        &app,
        download_request(MISSING_RUN_ROOM_ID, PROBLEM_ID, true),
    )
    .await;

    assert_fixture_response(
        response,
        StatusCode::NOT_FOUND,
        include_str!("../../openapi/examples/runs/error-run-not-found.json"),
    )
    .await;

    assert!(signer.calls().is_empty());
}

#[tokio::test]
async fn missing_problem_and_empty_assets_match_image_not_found_fixture() {
    let cases = [
        (DownloadBehavior::MissingProblem, MISSING_PROBLEM_ID),
        (DownloadBehavior::EmptyAssets, PROBLEM_ID),
    ];

    for (behavior, problem_id) in cases {
        let (app, signer) = download_test_app(AuthMode::Demo, behavior, Ok(()));

        let response = request(&app, download_request(ROOM_ID, problem_id, true)).await;

        assert_fixture_response(
            response,
            StatusCode::NOT_FOUND,
            include_str!("../../openapi/examples/assets/error-image-not-found.json"),
        )
        .await;

        assert!(signer.calls().is_empty());
    }
}

#[tokio::test]
async fn locked_problem_matches_fixture_without_signing() {
    let (app, signer) = download_test_app(AuthMode::Demo, DownloadBehavior::LockedProblem, Ok(()));

    let response = request(&app, download_request(ROOM_ID, LOCKED_PROBLEM_ID, true)).await;

    assert_fixture_response(
        response,
        StatusCode::CONFLICT,
        include_str!("../../openapi/examples/problems/error-problem-locked.json"),
    )
    .await;

    assert!(signer.calls().is_empty());
}

#[tokio::test]
async fn repository_error_returns_500_without_details() {
    let (app, signer) = download_test_app(AuthMode::Demo, DownloadBehavior::DatabaseError, Ok(()));

    let response = request(&app, download_request(ROOM_ID, PROBLEM_ID, true)).await;

    assert_error_code(
        response,
        StatusCode::INTERNAL_SERVER_ERROR,
        "INTERNAL_SERVER_ERROR",
    )
    .await;

    assert!(signer.calls().is_empty());
}

#[tokio::test]
async fn signing_error_returns_500_without_details() {
    let (app, signer) = download_test_app(
        AuthMode::Demo,
        DownloadBehavior::WithAssets,
        Err(ImageUrlSigningError::Signing),
    );

    let response = request(&app, download_request(ROOM_ID, PROBLEM_ID, true)).await;

    assert_error_code(
        response,
        StatusCode::INTERNAL_SERVER_ERROR,
        "INTERNAL_SERVER_ERROR",
    )
    .await;

    assert_eq!(
        signer.calls(),
        vec![(FIRST_OBJECT_KEY.to_owned(), Duration::from_secs(300),)]
    );
}
