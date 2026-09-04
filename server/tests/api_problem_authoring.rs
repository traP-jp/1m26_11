mod common;

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
};
use common::{body_json, request};
use openapi_generated::models::{CreateProblemResponse, ErrorResponse};
use serde_json::{Value, json};
use server::{
    AppState, app,
    config::AuthMode,
    repository::{
        AuthProvider, AuthRepository, AuthUserRecord, CreateProblemRecordOutcome,
        CreateProblemRecordRequest, RepositoryError,
    },
};
use sha2::{Digest, Sha256};
use uuid::Uuid;

const ROOM_ID: &str = "11111111-1111-4111-8111-111111111111";
const PROBLEM_ID: &str = "33333333-3333-4333-8333-333333333333";
const IDEMPOTENCY_KEY: &str = "44444444-4444-4444-8444-444444444444";

const OPERATION_REQUEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../openapi/examples/problems/create-operation-sequence-request.json"
));

const CREATED_RESPONSE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../openapi/examples/problems/create-response.json"
));

#[derive(Clone, Copy)]
enum RepositoryBehavior {
    Created,
    Replayed,
    Reused,
    RoomNotFound,
    PublishedRoom,
    NumberConflict,
    InvalidDependency,
    DatabaseError,
}

struct RecordingProblemRepository {
    behavior: RepositoryBehavior,
    requests: Mutex<Vec<CreateProblemRecordRequest>>,
}

impl RecordingProblemRepository {
    fn new(behavior: RepositoryBehavior) -> Self {
        Self {
            behavior,
            requests: Mutex::new(Vec::new()),
        }
    }

    fn requests(&self) -> Vec<CreateProblemRecordRequest> {
        self.requests
            .lock()
            .expect("problem request log should not be poisoned")
            .clone()
    }
}

#[async_trait]
impl AuthRepository for RecordingProblemRepository {
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
            user_id: Uuid::new_v4(),
            display_name: display_name.to_owned(),
            auth_provider,
        })
    }

    async fn create_problem(
        &self,
        request: &CreateProblemRecordRequest,
    ) -> Result<CreateProblemRecordOutcome, RepositoryError> {
        self.requests
            .lock()
            .expect("problem request log should not be poisoned")
            .push(request.clone());

        let problem_id = parse_uuid(PROBLEM_ID);

        match self.behavior {
            RepositoryBehavior::Created => Ok(CreateProblemRecordOutcome::Created { problem_id }),
            RepositoryBehavior::Replayed => Ok(CreateProblemRecordOutcome::Replayed { problem_id }),
            RepositoryBehavior::Reused => Ok(CreateProblemRecordOutcome::Reused),
            RepositoryBehavior::RoomNotFound => Err(RepositoryError::RoomNotFound),
            RepositoryBehavior::PublishedRoom => Err(RepositoryError::PublishedRoomImmutable),
            RepositoryBehavior::NumberConflict => Err(RepositoryError::ProblemNumberConflict),
            RepositoryBehavior::InvalidDependency => Err(RepositoryError::InvalidProblemDependency),
            RepositoryBehavior::DatabaseError => {
                Err(RepositoryError::Database(sqlx::Error::RowNotFound))
            }
        }
    }
}

fn parse_uuid(value: &str) -> Uuid {
    Uuid::parse_str(value).expect("test UUID should be valid")
}

fn create_path() -> String {
    format!("/api/rooms/{ROOM_ID}/problems")
}

fn test_app(
    auth_mode: AuthMode,
    enabled: bool,
    behavior: RepositoryBehavior,
) -> (Router, Arc<RecordingProblemRepository>) {
    let repository = Arc::new(RecordingProblemRepository::new(behavior));
    let state =
        AppState::new(auth_mode, repository.clone()).with_problem_authoring_enabled(enabled);

    (app(state), repository)
}

fn create_request(
    path: &str,
    idempotency_key: Option<&str>,
    body: impl Into<Body>,
) -> Request<Body> {
    let mut builder = Request::post(path).header(header::CONTENT_TYPE, "application/json");

    if let Some(idempotency_key) = idempotency_key {
        builder = builder.header("Idempotency-Key", idempotency_key);
    }

    builder
        .body(body.into())
        .expect("problem creation request should be valid")
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

#[tokio::test]
async fn authoring_route_is_registered_only_when_demo_and_enabled() {
    let configurations = [
        (AuthMode::Demo, false),
        (AuthMode::NeoShowcase, false),
        (AuthMode::NeoShowcase, true),
    ];

    for (auth_mode, enabled) in configurations {
        let (app, repository) = test_app(auth_mode, enabled, RepositoryBehavior::Created);

        let response = request(
            &app,
            create_request(
                &create_path(),
                Some(IDEMPOTENCY_KEY),
                Body::from(OPERATION_REQUEST),
            ),
        )
        .await;

        assert_error_response(response, StatusCode::NOT_FOUND, "NOT_FOUND").await;
        assert!(repository.requests().is_empty());
    }
}

#[tokio::test]
async fn created_and_replayed_requests_match_success_fixture() {
    for behavior in [RepositoryBehavior::Created, RepositoryBehavior::Replayed] {
        let (app, repository) = test_app(AuthMode::Demo, true, behavior);

        let response = request(
            &app,
            create_request(
                &create_path(),
                Some(IDEMPOTENCY_KEY),
                Body::from(OPERATION_REQUEST),
            ),
        )
        .await;

        assert_eq!(response.status(), StatusCode::CREATED);
        assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");

        let actual: CreateProblemResponse = body_json(response).await;
        let expected: CreateProblemResponse = serde_json::from_str(CREATED_RESPONSE)
            .expect("created response fixture should be valid");

        assert_eq!(actual, expected);

        let requests = repository.requests();
        assert_eq!(requests.len(), 1);

        let recorded = &requests[0];

        assert_eq!(recorded.request_method, "POST");
        assert_eq!(recorded.request_path, create_path());
        assert_eq!(recorded.idempotency_key, parse_uuid(IDEMPOTENCY_KEY));
        assert_eq!(recorded.draft.room_id, parse_uuid(ROOM_ID));
        assert_eq!(recorded.draft.number, 3);
        assert_eq!(recorded.draft.title, "操作列問題");
        assert!(recorded.draft.is_required);

        let serialized =
            serde_json::to_vec(&recorded.draft).expect("recorded draft should serialize");
        let expected_hash: [u8; 32] = Sha256::digest(serialized).into();

        assert_eq!(recorded.payload_sha256, expected_hash);
    }
}

#[tokio::test]
async fn reused_idempotency_key_returns_conflict() {
    let (app, repository) = test_app(AuthMode::Demo, true, RepositoryBehavior::Reused);

    let response = request(
        &app,
        create_request(
            &create_path(),
            Some(IDEMPOTENCY_KEY),
            Body::from(OPERATION_REQUEST),
        ),
    )
    .await;

    assert_error_response(response, StatusCode::CONFLICT, "IDEMPOTENCY_KEY_REUSED").await;

    assert_eq!(repository.requests().len(), 1);
}

#[tokio::test]
async fn repository_errors_are_mapped_to_contract_responses() {
    let cases = [
        (
            RepositoryBehavior::RoomNotFound,
            StatusCode::NOT_FOUND,
            "ROOM_NOT_FOUND",
        ),
        (
            RepositoryBehavior::PublishedRoom,
            StatusCode::CONFLICT,
            "PUBLISHED_ROOM_IMMUTABLE",
        ),
        (
            RepositoryBehavior::NumberConflict,
            StatusCode::CONFLICT,
            "PROBLEM_NUMBER_CONFLICT",
        ),
        (
            RepositoryBehavior::InvalidDependency,
            StatusCode::UNPROCESSABLE_ENTITY,
            "INVALID_PROBLEM",
        ),
        (
            RepositoryBehavior::DatabaseError,
            StatusCode::INTERNAL_SERVER_ERROR,
            "INTERNAL_SERVER_ERROR",
        ),
    ];

    for (behavior, expected_status, expected_code) in cases {
        let (app, repository) = test_app(AuthMode::Demo, true, behavior);

        let response = request(
            &app,
            create_request(
                &create_path(),
                Some(IDEMPOTENCY_KEY),
                Body::from(OPERATION_REQUEST),
            ),
        )
        .await;

        assert_error_response(response, expected_status, expected_code).await;
        assert_eq!(repository.requests().len(), 1);
    }
}

#[tokio::test]
async fn invalid_path_and_headers_are_rejected_before_repository() {
    let cases = [
        (
            "/api/rooms/not-a-uuid/problems",
            Some(IDEMPOTENCY_KEY),
            "INVALID_PATH_PARAMETER",
        ),
        (
            "/api/rooms/11111111-1111-4111-8111-111111111111/problems",
            None,
            "IDEMPOTENCY_KEY_REQUIRED",
        ),
        (
            "/api/rooms/11111111-1111-4111-8111-111111111111/problems",
            Some("not-a-uuid"),
            "INVALID_IDEMPOTENCY_KEY",
        ),
        (
            "/api/rooms/11111111-1111-4111-8111-111111111111/problems",
            Some("11111111-1111-1111-8111-111111111111"),
            "INVALID_IDEMPOTENCY_KEY",
        ),
    ];

    for (path, idempotency_key, expected_code) in cases {
        let (app, repository) = test_app(AuthMode::Demo, true, RepositoryBehavior::Created);

        let response = request(
            &app,
            create_request(path, idempotency_key, Body::from(OPERATION_REQUEST)),
        )
        .await;

        assert_error_response(response, StatusCode::BAD_REQUEST, expected_code).await;
        assert!(repository.requests().is_empty());
    }
}

#[tokio::test]
async fn malformed_json_is_rejected_before_repository() {
    let (app, repository) = test_app(AuthMode::Demo, true, RepositoryBehavior::Created);

    let response = request(
        &app,
        create_request(&create_path(), Some(IDEMPOTENCY_KEY), Body::from("{")),
    )
    .await;

    assert_error_response(response, StatusCode::BAD_REQUEST, "INVALID_JSON").await;
    assert!(repository.requests().is_empty());
}

#[tokio::test]
async fn invalid_problem_is_rejected_before_repository() {
    let (app, repository) = test_app(AuthMode::Demo, true, RepositoryBehavior::Created);

    let mut invalid_request: Value =
        serde_json::from_str(OPERATION_REQUEST).expect("operation request fixture should be valid");
    invalid_request["title"] = json!("   ");

    let response = request(
        &app,
        create_request(
            &create_path(),
            Some(IDEMPOTENCY_KEY),
            Body::from(invalid_request.to_string()),
        ),
    )
    .await;

    assert_error_response(
        response,
        StatusCode::UNPROCESSABLE_ENTITY,
        "INVALID_PROBLEM",
    )
    .await;

    assert!(repository.requests().is_empty());
}
