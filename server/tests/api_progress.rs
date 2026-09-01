mod common;

use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
};
use common::{body_bytes, body_json, request};
use serde_json::json;
use server::{
    AppState, app,
    config::AuthMode,
    repository::{
        AuthProvider, AuthRepository, AuthUserRecord, GenreProgressRecord, RepositoryError,
        UserProgressRecord,
    },
};
use uuid::Uuid;

const SESSION_ID: &str = "77777777-7777-4777-8777-777777777777";
const USER_ID: &str = "55555555-5555-4555-8555-555555555555";

const SUMMARY_RESPONSE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../openapi/examples/progress/response-summary.json"
));

const EMPTY_RESPONSE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../openapi/examples/progress/response-empty.json"
));

const UNAUTHORIZED_RESPONSE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../openapi/examples/auth/error-unauthorized.json"
));

struct StubAuthRepository {
    user: Option<AuthUserRecord>,
    progress: UserProgressRecord,
    fail_progress: bool,
}

#[async_trait]
impl AuthRepository for StubAuthRepository {
    async fn find_user_by_demo_session(
        &self,
        session_id: Uuid,
    ) -> Result<Option<AuthUserRecord>, RepositoryError> {
        assert_eq!(session_id, parse_uuid(SESSION_ID));
        Ok(self.user.clone())
    }

    async fn find_user_by_provider_subject(
        &self,
        _auth_provider: AuthProvider,
        _provider_subject: &str,
    ) -> Result<Option<AuthUserRecord>, RepositoryError> {
        panic!("NeoShowcase authentication was not expected");
    }

    async fn get_or_create_user(
        &self,
        _auth_provider: AuthProvider,
        _provider_subject: &str,
        _display_name: &str,
    ) -> Result<AuthUserRecord, RepositoryError> {
        panic!("user creation was not expected");
    }

    async fn find_user_progress(
        &self,
        user_id: Uuid,
    ) -> Result<UserProgressRecord, RepositoryError> {
        let expected_user = self
            .user
            .as_ref()
            .expect("progress lookup requires an authenticated user");

        assert_eq!(user_id, expected_user.user_id);

        if self.fail_progress {
            return Err(RepositoryError::Database(sqlx::Error::Protocol(
                "simulated private progress database failure".to_owned(),
            )));
        }

        Ok(self.progress.clone())
    }
}

fn parse_uuid(value: &str) -> Uuid {
    Uuid::parse_str(value).expect("test UUID should be valid")
}

fn authenticated_user() -> AuthUserRecord {
    AuthUserRecord {
        user_id: parse_uuid(USER_ID),
        display_name: "progress-user".to_owned(),
        auth_provider: AuthProvider::Demo,
    }
}

fn summary_progress() -> UserProgressRecord {
    UserProgressRecord {
        cleared_room_count: 5,
        total_room_count: 20,
        by_genre: vec![
            GenreProgressRecord {
                genre: "OSINT".to_owned(),
                cleared_room_count: 3,
                total_room_count: 8,
            },
            GenreProgressRecord {
                genre: "Web".to_owned(),
                cleared_room_count: 2,
                total_room_count: 12,
            },
        ],
    }
}

fn empty_progress() -> UserProgressRecord {
    UserProgressRecord {
        cleared_room_count: 0,
        total_room_count: 0,
        by_genre: Vec::new(),
    }
}

fn test_app(repository: StubAuthRepository) -> Router {
    app(AppState::new(AuthMode::Demo, Arc::new(repository)))
}

fn authenticated_request() -> Request<Body> {
    Request::get("/api/me/progress")
        .header(header::COOKIE, format!("demo_session={SESSION_ID}"))
        .body(Body::empty())
        .expect("request should be valid")
}

async fn assert_response_matches_fixture(
    app: &Router,
    request_value: Request<Body>,
    fixture: &str,
) {
    let response = request(app, request_value).await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");

    let actual: serde_json::Value = body_json(response).await;
    let expected: serde_json::Value =
        serde_json::from_str(fixture).expect("fixture should be valid JSON");

    assert_eq!(actual, expected);
}

#[tokio::test]
async fn authenticated_progress_matches_summary_fixture() {
    let app = test_app(StubAuthRepository {
        user: Some(authenticated_user()),
        progress: summary_progress(),
        fail_progress: false,
    });

    assert_response_matches_fixture(&app, authenticated_request(), SUMMARY_RESPONSE).await;
}

#[tokio::test]
async fn empty_progress_matches_fixture() {
    let app = test_app(StubAuthRepository {
        user: Some(authenticated_user()),
        progress: empty_progress(),
        fail_progress: false,
    });

    assert_response_matches_fixture(&app, authenticated_request(), EMPTY_RESPONSE).await;
}

#[tokio::test]
async fn unauthenticated_progress_returns_401_fixture() {
    let app = test_app(StubAuthRepository {
        user: None,
        progress: empty_progress(),
        fail_progress: false,
    });

    let request_value = Request::get("/api/me/progress")
        .body(Body::empty())
        .expect("request should be valid");

    let response = request(&app, request_value).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");

    let actual: serde_json::Value = body_json(response).await;
    let expected: serde_json::Value =
        serde_json::from_str(UNAUTHORIZED_RESPONSE).expect("fixture should be valid JSON");

    assert_eq!(actual, expected);
}

#[tokio::test]
async fn repository_error_returns_500_without_details() {
    let app = test_app(StubAuthRepository {
        user: Some(authenticated_user()),
        progress: empty_progress(),
        fail_progress: true,
    });

    let response = request(&app, authenticated_request()).await;

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");

    let body = body_bytes(response).await;
    let body_text = std::str::from_utf8(&body).expect("response body should be UTF-8");

    assert!(
        !body_text.contains("simulated private progress database failure"),
        "database error details must not be exposed"
    );

    let body: serde_json::Value =
        serde_json::from_slice(&body).expect("response body should be valid JSON");

    assert_eq!(body["error"]["code"], "INTERNAL_SERVER_ERROR");
    assert_eq!(body["error"]["message"], "internal server error");
    assert_eq!(body["error"]["details"], json!({}));
}
