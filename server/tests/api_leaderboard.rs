mod common;

use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
};
use chrono::{DateTime, Utc};
use common::{body_bytes, body_json, request};
use serde_json::json;
use server::{
    AppState, app,
    config::AuthMode,
    repository::{
        AuthProvider, AuthRepository, AuthUserRecord, LeaderboardRecord, RepositoryError,
        RoomRecord,
    },
};
use uuid::Uuid;

const ROOM_ID: &str = "11111111-1111-4111-8111-111111111111";
const SESSION_ID: &str = "77777777-7777-4777-8777-777777777777";
const CAROL_ID: &str = "66666666-6666-4666-8666-666666666666";

const RANKED_RESPONSE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../openapi/examples/leaderboard/response-ranked.json"
));

const UNAUTHENTICATED_RESPONSE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../openapi/examples/leaderboard/response-unauthenticated.json"
));

const EMPTY_RESPONSE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../openapi/examples/leaderboard/response-empty.json"
));

struct StubAuthRepository {
    user: Option<AuthUserRecord>,
    room: Option<RoomRecord>,
    leaderboard: Vec<LeaderboardRecord>,
    fail_leaderboard: bool,
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

    async fn find_room_by_id(&self, room_id: Uuid) -> Result<Option<RoomRecord>, RepositoryError> {
        assert_eq!(room_id, parse_uuid(ROOM_ID));
        Ok(self.room.clone())
    }

    async fn find_leaderboard_by_room_id(
        &self,
        room_id: Uuid,
    ) -> Result<Vec<LeaderboardRecord>, RepositoryError> {
        assert_eq!(room_id, parse_uuid(ROOM_ID));

        if self.fail_leaderboard {
            return Err(RepositoryError::Database(sqlx::Error::Protocol(
                "simulated private leaderboard database failure".to_owned(),
            )));
        }

        Ok(self.leaderboard.clone())
    }
}

fn parse_uuid(value: &str) -> Uuid {
    Uuid::parse_str(value).expect("test UUID should be valid")
}

fn parse_datetime(value: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(value)
        .expect("test datetime should be valid")
        .with_timezone(&Utc)
}

fn room_record() -> RoomRecord {
    RoomRecord {
        id: parse_uuid(ROOM_ID),
        number: 1,
        name: "Test Room".to_owned(),
        genre: "test".to_owned(),
        description: "leaderboard test room".to_owned(),
        is_published: true,
        created_at: parse_datetime("2026-08-06T10:00:00Z"),
    }
}

fn ranked_records() -> Vec<LeaderboardRecord> {
    vec![
        LeaderboardRecord {
            rank: 1,
            user_id: parse_uuid("44444444-4444-4444-8444-444444444444"),
            display_name: "Alice".to_owned(),
            elapsed_ms: 72_340,
            query_count: 21,
            cleared_at: parse_datetime("2026-08-06T10:05:00Z"),
        },
        LeaderboardRecord {
            rank: 1,
            user_id: parse_uuid("55555555-5555-4555-8555-555555555555"),
            display_name: "Bob".to_owned(),
            elapsed_ms: 72_340,
            query_count: 21,
            cleared_at: parse_datetime("2026-08-06T10:05:00Z"),
        },
        LeaderboardRecord {
            rank: 3,
            user_id: parse_uuid(CAROL_ID),
            display_name: "Carol".to_owned(),
            elapsed_ms: 80_000,
            query_count: 18,
            cleared_at: parse_datetime("2026-08-06T10:07:00Z"),
        },
    ]
}

fn authenticated_user(user_id: &str, display_name: &str) -> AuthUserRecord {
    AuthUserRecord {
        user_id: parse_uuid(user_id),
        display_name: display_name.to_owned(),
        auth_provider: AuthProvider::Demo,
    }
}

fn test_app(repository: StubAuthRepository) -> Router {
    app(AppState::new(AuthMode::Demo, Arc::new(repository)))
}

fn leaderboard_url() -> String {
    format!("/api/rooms/{ROOM_ID}/leaderboard")
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
async fn authenticated_leaderboard_matches_ranked_fixture() {
    let app = test_app(StubAuthRepository {
        user: Some(authenticated_user(CAROL_ID, "Carol")),
        room: Some(room_record()),
        leaderboard: ranked_records(),
        fail_leaderboard: false,
    });

    let req = Request::get(leaderboard_url())
        .header(header::COOKIE, format!("demo_session={SESSION_ID}"))
        .body(Body::empty())
        .expect("request should be valid");

    assert_response_matches_fixture(&app, req, RANKED_RESPONSE).await;
}

#[tokio::test]
async fn unauthenticated_leaderboard_matches_fixture() {
    let app = test_app(StubAuthRepository {
        user: None,
        room: Some(room_record()),
        leaderboard: ranked_records().into_iter().take(1).collect(),
        fail_leaderboard: false,
    });

    let req = Request::get(leaderboard_url())
        .body(Body::empty())
        .expect("request should be valid");

    assert_response_matches_fixture(&app, req, UNAUTHENTICATED_RESPONSE).await;
}

#[tokio::test]
async fn empty_leaderboard_matches_fixture() {
    let app = test_app(StubAuthRepository {
        user: None,
        room: Some(room_record()),
        leaderboard: Vec::new(),
        fail_leaderboard: false,
    });

    let req = Request::get(leaderboard_url())
        .body(Body::empty())
        .expect("request should be valid");

    assert_response_matches_fixture(&app, req, EMPTY_RESPONSE).await;
}

#[tokio::test]
async fn authenticated_user_without_cleared_run_has_null_me() {
    let app = test_app(StubAuthRepository {
        user: Some(authenticated_user(
            "88888888-8888-4888-8888-888888888888",
            "Dave",
        )),
        room: Some(room_record()),
        leaderboard: ranked_records(),
        fail_leaderboard: false,
    });

    let req = Request::get(leaderboard_url())
        .header(header::COOKIE, format!("demo_session={SESSION_ID}"))
        .body(Body::empty())
        .expect("request should be valid");

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::OK);

    let body: serde_json::Value = body_json(response).await;
    assert_eq!(body["entries"].as_array().map(Vec::len), Some(3));
    assert!(body["me"].is_null());
}

#[tokio::test]
async fn invalid_room_id_returns_400() {
    let app = test_app(StubAuthRepository {
        user: None,
        room: Some(room_record()),
        leaderboard: Vec::new(),
        fail_leaderboard: false,
    });

    let req = Request::get("/api/rooms/not-a-uuid/leaderboard")
        .body(Body::empty())
        .expect("request should be valid");

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = body_json(response).await;
    assert_eq!(body["error"]["code"], "BAD_REQUEST");
    assert_eq!(body["error"]["message"], "invalid room_id");
    assert_eq!(body["error"]["details"], json!({}));
}

#[tokio::test]
async fn missing_room_returns_404() {
    let app = test_app(StubAuthRepository {
        user: None,
        room: None,
        leaderboard: Vec::new(),
        fail_leaderboard: false,
    });

    let req = Request::get(leaderboard_url())
        .body(Body::empty())
        .expect("request should be valid");

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body: serde_json::Value = body_json(response).await;
    assert_eq!(body["error"]["code"], "NOT_FOUND");
    assert_eq!(body["error"]["message"], "room not found");
    assert_eq!(body["error"]["details"], json!({}));
}

#[tokio::test]
async fn unpublished_room_returns_404() {
    let mut room = room_record();
    room.is_published = false;

    let app = test_app(StubAuthRepository {
        user: None,
        room: Some(room),
        leaderboard: ranked_records(),
        fail_leaderboard: false,
    });

    let req = Request::get(leaderboard_url())
        .body(Body::empty())
        .expect("request should be valid");

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body: serde_json::Value = body_json(response).await;
    assert_eq!(body["error"]["code"], "NOT_FOUND");
    assert_eq!(body["error"]["message"], "room not found");
    assert_eq!(body["error"]["details"], json!({}));
}

#[tokio::test]
async fn repository_error_returns_500_without_details() {
    let app = test_app(StubAuthRepository {
        user: None,
        room: Some(room_record()),
        leaderboard: Vec::new(),
        fail_leaderboard: true,
    });

    let req = Request::get(leaderboard_url())
        .body(Body::empty())
        .expect("request should be valid");

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");

    let body = body_bytes(response).await;
    let body_text = std::str::from_utf8(&body).expect("response body should be UTF-8");

    assert!(
        !body_text.contains("simulated private leaderboard database failure"),
        "database error details must not be exposed"
    );

    let body: serde_json::Value =
        serde_json::from_slice(&body).expect("response body should be valid JSON");

    assert_eq!(body["error"]["details"], json!({}));
}
