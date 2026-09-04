mod common;

use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
};
use chrono::{DateTime, Utc};
use common::{body_bytes, body_json};
use server::{
    AppState, app,
    config::AuthMode,
    repository::{
        AuthProvider, AuthRepository, AuthUserRecord, LeaderboardRecord, RepositoryError,
        RoomRecord, RunRecord,
    },
};
use uuid::Uuid;

const ROOM_ID: &str = "11111111-1111-4111-8111-111111111111";
const SESSION_ID: &str = "77777777-7777-4777-8777-777777777777";
const USER_ID: &str = "44444444-4444-4444-8444-444444444444";

const ACTIVE_RESPONSE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../openapi/examples/rooms/response-active.json"
));

const NOT_STARTED_RESPONSE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../openapi/examples/rooms/response-not-started.json"
));

const CLEARED_RESPONSE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../openapi/examples/rooms/response-cleared.json"
));

struct StubAuthRepository {
    user: Option<AuthUserRecord>,
    room: Option<RoomRecord>,
    problem_count: u32,
    active_run: Option<RunRecord>,
    cleared_run: Option<RunRecord>,
    leaderboard: Vec<LeaderboardRecord>,
    fail_room: bool,
    fail_problem_count: bool,
    fail_run: bool,
    fail_leaderboard: bool,
}

impl Default for StubAuthRepository {
    fn default() -> Self {
        Self {
            user: None,
            room: Some(room_record()),
            problem_count: 4,
            active_run: None,
            cleared_run: None,
            leaderboard: Vec::new(),
            fail_room: false,
            fail_problem_count: false,
            fail_run: false,
            fail_leaderboard: false,
        }
    }
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

        if self.fail_room {
            return Err(RepositoryError::Database(sqlx::Error::Protocol(
                "simulated private room database failure".to_owned(),
            )));
        }

        Ok(self.room.clone())
    }

    async fn count_problems_by_room_id(&self, room_id: Uuid) -> Result<u32, RepositoryError> {
        assert_eq!(room_id, parse_uuid(ROOM_ID));

        if self.fail_problem_count {
            return Err(RepositoryError::Database(sqlx::Error::Protocol(
                "simulated private problem count database failure".to_owned(),
            )));
        }

        Ok(self.problem_count)
    }

    async fn find_active_run(
        &self,
        _user_id: Uuid,
        room_id: Uuid,
    ) -> Result<Option<RunRecord>, RepositoryError> {
        assert_eq!(room_id, parse_uuid(ROOM_ID));

        if self.fail_run {
            return Err(RepositoryError::Database(sqlx::Error::Protocol(
                "simulated private run database failure".to_owned(),
            )));
        }

        Ok(self.active_run.clone())
    }

    async fn find_cleared_run(
        &self,
        _user_id: Uuid,
        room_id: Uuid,
    ) -> Result<Option<RunRecord>, RepositoryError> {
        assert_eq!(room_id, parse_uuid(ROOM_ID));

        if self.fail_run {
            return Err(RepositoryError::Database(sqlx::Error::Protocol(
                "simulated private run database failure".to_owned(),
            )));
        }

        Ok(self.cleared_run.clone())
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
        number: 12,
        name: "general".to_owned(),
        genre: "OSINT".to_owned(),
        description: "人物を特定して脱出せよ".to_owned(),
        is_published: true,
        created_at: parse_datetime("2026-08-06T10:00:00Z"),
    }
}

fn authenticated_user() -> AuthUserRecord {
    AuthUserRecord {
        user_id: parse_uuid(USER_ID),
        display_name: "Alice".to_owned(),
        auth_provider: AuthProvider::Demo,
    }
}

fn active_run_record() -> RunRecord {
    RunRecord {
        id: Uuid::new_v4(),
        user_id: parse_uuid(USER_ID),
        room_id: parse_uuid(ROOM_ID),
        status: "active".to_owned(),
        started_at: parse_datetime("2026-08-06T10:00:00Z"),
        cleared_at: None,
    }
}

fn cleared_run_record() -> RunRecord {
    RunRecord {
        id: Uuid::new_v4(),
        user_id: parse_uuid(USER_ID),
        room_id: parse_uuid(ROOM_ID),
        status: "cleared".to_owned(),
        started_at: parse_datetime("2026-08-06T10:00:00Z"),
        cleared_at: Some(parse_datetime("2026-08-06T10:05:00Z")),
    }
}

fn mock_leaderboard_records(
    user_id: Uuid,
    user_rank: u32,
    total_players: usize,
) -> Vec<LeaderboardRecord> {
    let mut records = Vec::with_capacity(total_players);
    for i in 1..=total_players {
        let rank = i as u32;
        let id = if rank == user_rank {
            user_id
        } else {
            Uuid::from_u128(1000 + i as u128)
        };
        records.push(LeaderboardRecord {
            rank,
            user_id: id,
            display_name: format!("Player {i}"),
            elapsed_ms: (10000 + i * 100) as u64,
            query_count: i as u64,
            cleared_at: parse_datetime("2026-08-06T10:05:00Z"),
        });
    }
    records
}

fn test_app(repository: StubAuthRepository) -> Router {
    app(AppState::new(AuthMode::Demo, Arc::new(repository)).with_demo_cookie_secure(false))
}

fn room_detail_url() -> String {
    format!("/api/rooms/{ROOM_ID}")
}

async fn assert_response_matches_fixture(app: &Router, request: Request<Body>, fixture: &str) {
    let response = common::request(app, request).await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(header::CONTENT_TYPE).unwrap(),
        "application/json"
    );

    let body = body_bytes(response).await;
    let actual: serde_json::Value =
        serde_json::from_slice(&body).expect("response should be valid JSON");
    let expected: serde_json::Value =
        serde_json::from_str(fixture).expect("fixture should be valid JSON");

    assert_eq!(actual, expected);
}

#[tokio::test]
async fn authenticated_active_room_detail_matches_fixture() {
    let app = test_app(StubAuthRepository {
        user: Some(authenticated_user()),
        room: Some(room_record()),
        problem_count: 4,
        active_run: Some(active_run_record()),
        cleared_run: None,
        leaderboard: mock_leaderboard_records(parse_uuid(USER_ID), 14, 84),
        ..Default::default()
    });

    let req = Request::get(room_detail_url())
        .header(header::COOKIE, format!("demo_session={SESSION_ID}"))
        .body(Body::empty())
        .expect("request should be valid");

    assert_response_matches_fixture(&app, req, ACTIVE_RESPONSE).await;
}

#[tokio::test]
async fn unauthenticated_not_started_room_detail_matches_fixture() {
    let app = test_app(StubAuthRepository {
        user: None,
        room: Some(room_record()),
        problem_count: 4,
        active_run: None,
        cleared_run: None,
        leaderboard: mock_leaderboard_records(parse_uuid(USER_ID), 14, 84),
        ..Default::default()
    });

    let req = Request::get(room_detail_url())
        .body(Body::empty())
        .expect("request should be valid");

    assert_response_matches_fixture(&app, req, NOT_STARTED_RESPONSE).await;
}

#[tokio::test]
async fn authenticated_cleared_room_detail_matches_fixture() {
    let app = test_app(StubAuthRepository {
        user: Some(authenticated_user()),
        room: Some(room_record()),
        problem_count: 4,
        active_run: None,
        cleared_run: Some(cleared_run_record()),
        leaderboard: mock_leaderboard_records(parse_uuid(USER_ID), 14, 84),
        ..Default::default()
    });

    let req = Request::get(room_detail_url())
        .header(header::COOKIE, format!("demo_session={SESSION_ID}"))
        .body(Body::empty())
        .expect("request should be valid");

    assert_response_matches_fixture(&app, req, CLEARED_RESPONSE).await;
}

#[tokio::test]
async fn authenticated_without_run_returns_not_started_and_null_my_rank() {
    let app = test_app(StubAuthRepository {
        user: Some(authenticated_user()),
        room: Some(room_record()),
        problem_count: 4,
        active_run: None,
        cleared_run: None,
        leaderboard: mock_leaderboard_records(Uuid::new_v4(), 14, 84),
        ..Default::default()
    });

    let req = Request::get(room_detail_url())
        .header(header::COOKIE, format!("demo_session={SESSION_ID}"))
        .body(Body::empty())
        .expect("request should be valid");

    let response = common::request(&app, req).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body: serde_json::Value = body_json(response).await;
    assert_eq!(body["run_status"], "not_started");
    assert_eq!(body["ranking_summary"]["player_count"], 84);
    assert!(body["ranking_summary"]["my_rank"].is_null());
}

#[tokio::test]
async fn authenticated_cleared_user_not_in_leaderboard_returns_null_my_rank() {
    let other_user_id = Uuid::new_v4();
    let app = test_app(StubAuthRepository {
        user: Some(authenticated_user()),
        room: Some(room_record()),
        problem_count: 4,
        active_run: None,
        cleared_run: Some(cleared_run_record()),
        // User not in leaderboard
        leaderboard: mock_leaderboard_records(other_user_id, 1, 5),
        ..Default::default()
    });

    let req = Request::get(room_detail_url())
        .header(header::COOKIE, format!("demo_session={SESSION_ID}"))
        .body(Body::empty())
        .expect("request should be valid");

    let response = common::request(&app, req).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body: serde_json::Value = body_json(response).await;
    assert_eq!(body["run_status"], "cleared");
    assert_eq!(body["ranking_summary"]["player_count"], 5);
    assert!(body["ranking_summary"]["my_rank"].is_null());
}

#[tokio::test]
async fn empty_leaderboard_returns_player_count_0_and_null_my_rank() {
    let app = test_app(StubAuthRepository {
        user: Some(authenticated_user()),
        room: Some(room_record()),
        problem_count: 3,
        active_run: None,
        cleared_run: None,
        leaderboard: Vec::new(),
        ..Default::default()
    });

    let req = Request::get(room_detail_url())
        .header(header::COOKIE, format!("demo_session={SESSION_ID}"))
        .body(Body::empty())
        .expect("request should be valid");

    let response = common::request(&app, req).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body: serde_json::Value = body_json(response).await;
    assert_eq!(body["problem_count"], 3);
    assert_eq!(body["run_status"], "not_started");
    assert_eq!(body["ranking_summary"]["player_count"], 0);
    assert!(body["ranking_summary"]["my_rank"].is_null());
}

#[tokio::test]
async fn unpublished_room_returns_404() {
    let mut room = room_record();
    room.is_published = false;

    let app = test_app(StubAuthRepository {
        room: Some(room),
        ..Default::default()
    });

    let req = Request::get(room_detail_url())
        .body(Body::empty())
        .expect("request should be valid");

    let response = common::request(&app, req).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body: serde_json::Value = body_json(response).await;
    assert_eq!(body["error"]["code"], "NOT_FOUND");
    assert_eq!(body["error"]["message"], "room not found");
}

#[tokio::test]
async fn room_not_found_returns_404() {
    let app = test_app(StubAuthRepository {
        room: None,
        ..Default::default()
    });

    let req = Request::get(room_detail_url())
        .body(Body::empty())
        .expect("request should be valid");

    let response = common::request(&app, req).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body: serde_json::Value = body_json(response).await;
    assert_eq!(body["error"]["code"], "NOT_FOUND");
    assert_eq!(body["error"]["message"], "room not found");
}

#[tokio::test]
async fn invalid_room_id_uuid_returns_400() {
    let app = test_app(StubAuthRepository::default());

    let req = Request::get("/api/rooms/not-a-valid-uuid")
        .body(Body::empty())
        .expect("request should be valid");

    let response = common::request(&app, req).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = body_json(response).await;
    assert_eq!(body["error"]["code"], "BAD_REQUEST");
    assert_eq!(body["error"]["message"], "invalid room_id");
}

#[tokio::test]
async fn database_error_on_room_lookup_returns_500() {
    let app = test_app(StubAuthRepository {
        fail_room: true,
        ..Default::default()
    });

    let req = Request::get(room_detail_url())
        .body(Body::empty())
        .expect("request should be valid");

    let response = common::request(&app, req).await;
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let body: serde_json::Value = body_json(response).await;
    assert_eq!(body["error"]["code"], "INTERNAL_SERVER_ERROR");
}

#[tokio::test]
async fn database_error_on_problem_count_returns_500() {
    let app = test_app(StubAuthRepository {
        fail_problem_count: true,
        ..Default::default()
    });

    let req = Request::get(room_detail_url())
        .body(Body::empty())
        .expect("request should be valid");

    let response = common::request(&app, req).await;
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let body: serde_json::Value = body_json(response).await;
    assert_eq!(body["error"]["code"], "INTERNAL_SERVER_ERROR");
}

#[tokio::test]
async fn database_error_on_run_lookup_returns_500() {
    let app = test_app(StubAuthRepository {
        user: Some(authenticated_user()),
        fail_run: true,
        ..Default::default()
    });

    let req = Request::get(room_detail_url())
        .header(header::COOKIE, format!("demo_session={SESSION_ID}"))
        .body(Body::empty())
        .expect("request should be valid");

    let response = common::request(&app, req).await;
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let body: serde_json::Value = body_json(response).await;
    assert_eq!(body["error"]["code"], "INTERNAL_SERVER_ERROR");
}

#[tokio::test]
async fn database_error_on_leaderboard_lookup_returns_500() {
    let app = test_app(StubAuthRepository {
        fail_leaderboard: true,
        ..Default::default()
    });

    let req = Request::get(room_detail_url())
        .body(Body::empty())
        .expect("request should be valid");

    let response = common::request(&app, req).await;
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let body: serde_json::Value = body_json(response).await;
    assert_eq!(body["error"]["code"], "INTERNAL_SERVER_ERROR");
}

#[tokio::test]
async fn method_not_allowed_for_unsupported_http_methods() {
    let app = test_app(StubAuthRepository::default());

    for method in ["POST", "PUT", "DELETE", "PATCH"] {
        let req = Request::builder()
            .method(method)
            .uri(room_detail_url())
            .body(Body::empty())
            .expect("request should be valid");

        let response = common::request(&app, req).await;
        assert_eq!(
            response.status(),
            StatusCode::METHOD_NOT_ALLOWED,
            "method {method} should return 405"
        );
    }
}
