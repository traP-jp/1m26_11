use std::{env, str::FromStr, sync::Arc};

use async_trait::async_trait;
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
};
use chrono::{DateTime, Utc};
use http_body_util::BodyExt;
use openapi_generated::models::ErrorResponse;
use serde_json::json;
use server::{
    AppState, app,
    config::AuthMode,
    migrate,
    repository::{
        AuthRepository, AuthUserRecord, RepositoryError, RoomRecord, RunRecord, SqlxUserRepository,
    },
};
use sqlx::{
    MySqlPool,
    mysql::{MySqlConnectOptions, MySqlPoolOptions},
};
use tower::ServiceExt;
use uuid::Uuid;

const MOCK_SESSION_ID: &str = "55555555-5555-4555-8555-555555555555";
const MOCK_RESUME_ROOM_ID: &str = "11111111-1111-4111-8111-111111111111";
const MOCK_NEW_ROOM_ID: &str = "33333333-3333-4333-8333-333333333333";

struct StubAuthRepository;

#[async_trait]
impl AuthRepository for StubAuthRepository {
    async fn find_user_by_demo_session(
        &self,
        _session_id: Uuid,
    ) -> Result<Option<AuthUserRecord>, RepositoryError> {
        Ok(Some(AuthUserRecord {
            id: Uuid::from_str(MOCK_SESSION_ID).unwrap(),
            display_name: "test-user".to_owned(),
        }))
    }

    async fn find_user_by_provider_subject(
        &self,
        _auth_provider: &str,
        _provider_subject: &str,
    ) -> Result<Option<AuthUserRecord>, RepositoryError> {
        Ok(None)
    }

    async fn get_or_create_user(
        &self,
        _auth_provider: &str,
        _provider_subject: &str,
        display_name: &str,
    ) -> Result<AuthUserRecord, RepositoryError> {
        Ok(AuthUserRecord {
            id: Uuid::new_v4(),
            display_name: display_name.to_owned(),
        })
    }

    async fn create_demo_session(
        &self,
        _session_id: Uuid,
        _user_id: Uuid,
    ) -> Result<(), RepositoryError> {
        Ok(())
    }

    async fn delete_demo_session(&self, _session_id: Uuid) -> Result<(), RepositoryError> {
        Ok(())
    }

    async fn find_room_by_id(&self, room_id: Uuid) -> Result<Option<RoomRecord>, RepositoryError> {
        if room_id == Uuid::nil() {
            Ok(None)
        } else {
            Ok(Some(RoomRecord {
                id: room_id,
                number: 1,
                name: "Test Room".to_owned(),
                genre: "Test".to_owned(),
                description: "Test description".to_owned(),
                is_published: true,
                created_at: Utc::now(),
            }))
        }
    }

    async fn find_active_run(
        &self,
        _user_id: Uuid,
        _room_id: Uuid,
    ) -> Result<Option<RunRecord>, RepositoryError> {
        let resume_room_id = Uuid::from_str(MOCK_RESUME_ROOM_ID).unwrap();
        if _room_id == resume_room_id {
            Ok(Some(RunRecord {
                id: Uuid::new_v4(),
                user_id: _user_id,
                room_id: _room_id,
                status: "active".to_owned(),
                started_at: Utc::now() - chrono::Duration::seconds(65),
                cleared_at: None,
            }))
        } else {
            Ok(None)
        }
    }

    async fn create_run(
        &self,
        id: Uuid,
        user_id: Uuid,
        room_id: Uuid,
        started_at: DateTime<Utc>,
    ) -> Result<RunRecord, RepositoryError> {
        Ok(RunRecord {
            id,
            user_id,
            room_id,
            status: "active".to_owned(),
            started_at,
            cleared_at: None,
        })
    }
}

#[tokio::test]
async fn ping_returns_plain_text_pong() {
    let app = test_app();
    let response = request(
        &app,
        Request::get("/api/v1/ping").body(Body::empty()).unwrap(),
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "text/plain; charset=utf-8"
    );
    assert_eq!(body_bytes(response).await, b"pong");
}

#[tokio::test]
async fn openapi_returns_embedded_shared_document() {
    let app = test_app();
    let response = request(
        &app,
        Request::get("/openapi.yaml").body(Body::empty()).unwrap(),
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/yaml");
    assert_eq!(
        body_bytes(response).await,
        server::OPENAPI_DOCUMENT.as_bytes()
    );
}

#[tokio::test]
async fn unknown_route_returns_json_404() {
    let app = test_app();

    let response = request(&app, Request::get("/missing").body(Body::empty()).unwrap()).await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let error = body_json::<ErrorResponse>(response).await;

    assert_eq!(error.error.code, "NOT_FOUND");
    assert_eq!(error.error.message, "route not found");
    assert_eq!(error.error.details.0, json!({}));
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL pointing to a disposable MariaDB database"]
async fn mariadb_auth_repository_flow() {
    let pool = connect_test_database().await;

    migrate(&pool).await.expect("migration should succeed");

    let repository = SqlxUserRepository::new(pool.clone());

    let demo_user_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let demo_subject = format!("integration-demo-{demo_user_id}");

    sqlx::query(
        r#"
        INSERT INTO users (
            id,
            auth_provider,
            provider_subject,
            display_name
        )
        VALUES (?, 'demo', ?, ?)
        "#,
    )
    .bind(demo_user_id)
    .bind(&demo_subject)
    .bind("demo-user")
    .execute(&pool)
    .await
    .expect("demo user insertion should succeed");

    sqlx::query(
        r#"
        INSERT INTO demo_sessions (id, user_id)
        VALUES (?, ?)
        "#,
    )
    .bind(session_id)
    .bind(demo_user_id)
    .execute(&pool)
    .await
    .expect("demo session insertion should succeed");

    let demo_user = repository
        .find_user_by_demo_session(session_id)
        .await
        .expect("demo session lookup should succeed")
        .expect("demo session should resolve a user");

    let neo_subject = format!("integration-neoshowcase-{}", Uuid::new_v4());

    let first_neo_user = repository
        .get_or_create_user("neoshowcase", &neo_subject, "neo-user")
        .await
        .expect("first NeoShowcase lookup should succeed");

    let second_neo_user = repository
        .get_or_create_user("neoshowcase", &neo_subject, "neo-user")
        .await
        .expect("second NeoShowcase lookup should succeed");

    sqlx::query("DELETE FROM demo_sessions WHERE id = ?")
        .bind(session_id)
        .execute(&pool)
        .await
        .expect("demo session cleanup should succeed");

    sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(demo_user_id)
        .execute(&pool)
        .await
        .expect("demo user cleanup should succeed");

    sqlx::query(
        r#"
        DELETE FROM users
        WHERE auth_provider = 'neoshowcase'
          AND provider_subject = ?
        "#,
    )
    .bind(&neo_subject)
    .execute(&pool)
    .await
    .expect("NeoShowcase user cleanup should succeed");

    pool.close().await;

    assert_eq!(
        demo_user,
        AuthUserRecord {
            id: demo_user_id,
            display_name: "demo-user".to_owned(),
        }
    );

    assert_eq!(first_neo_user, second_neo_user);
    assert_eq!(first_neo_user.display_name, "neo-user");
}

fn test_app() -> Router {
    app(AppState::new(AuthMode::Demo, Arc::new(StubAuthRepository)))
}

async fn request(app: &Router, request: Request<Body>) -> axum::response::Response {
    app.clone().oneshot(request).await.unwrap()
}

async fn body_json<T: serde::de::DeserializeOwned>(response: axum::response::Response) -> T {
    serde_json::from_slice(&body_bytes(response).await).unwrap()
}

async fn body_bytes(response: axum::response::Response) -> Vec<u8> {
    response
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes()
        .to_vec()
}

async fn connect_test_database() -> MySqlPool {
    let database_url = env::var("TEST_DATABASE_URL")
        .expect("TEST_DATABASE_URL must point to a disposable test database");

    let options =
        MySqlConnectOptions::from_str(&database_url).expect("TEST_DATABASE_URL should be valid");

    MySqlPoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .expect("test database should be reachable")
}

#[tokio::test]
async fn guest_login_succeeds_in_demo_mode() {
    let app = app(AppState::new(AuthMode::Demo, Arc::new(StubAuthRepository)));

    let request_payload = json!({
        "display_name": "hoge"
    });

    let req = Request::post("/api/auth/guest")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&request_payload).unwrap()))
        .unwrap();

    let response = request(&app, req).await;

    let headers = response.headers().clone();
    let status = response.status();
    assert_eq!(status, StatusCode::OK);

    // Verify response body matches:
    // { "authenticated": true, "user": { "id": "uuid", "display_name": "hoge" } }
    let body: serde_json::Value = body_json(response).await;
    assert_eq!(body["authenticated"], true);
    assert_eq!(body["user"]["display_name"], "hoge");
    assert!(Uuid::parse_str(body["user"]["id"].as_str().unwrap()).is_ok());

    // Verify cookie:
    // It should have a Set-Cookie header with demo_session=<uuid>
    let cookie_header = headers
        .get(header::SET_COOKIE)
        .expect("Set-Cookie header should be present");
    let cookie_str = cookie_header.to_str().unwrap();
    assert!(cookie_str.contains("demo_session="));
    assert!(cookie_str.contains("HttpOnly"));
    assert!(cookie_str.contains("Path=/"));
}

#[tokio::test]
async fn guest_login_returns_404_in_neoshowcase_mode() {
    let app = app(AppState::new(
        AuthMode::NeoShowcase,
        Arc::new(StubAuthRepository),
    ));

    let request_payload = json!({
        "display_name": "hoge"
    });

    let req = Request::post("/api/auth/guest")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&request_payload).unwrap()))
        .unwrap();

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn guest_logout_succeeds_in_demo_mode() {
    let app = app(AppState::new(AuthMode::Demo, Arc::new(StubAuthRepository)));

    let req = Request::post("/api/auth/logout")
        .header(header::COOKIE, format!("demo_session={MOCK_SESSION_ID}"))
        .body(Body::empty())
        .unwrap();

    let response = request(&app, req).await;

    let headers = response.headers().clone();
    let status = response.status();
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Verify cookie removal:
    let cookie_header = headers
        .get(header::SET_COOKIE)
        .expect("Set-Cookie header should be present");
    let cookie_str = cookie_header.to_str().unwrap();
    assert!(cookie_str.contains("demo_session="));
    assert!(cookie_str.contains("Max-Age=0") || cookie_str.contains("expires="));
    assert!(cookie_str.contains("Path=/"));
}

#[tokio::test]
async fn guest_logout_returns_404_in_neoshowcase_mode() {
    let app = app(AppState::new(
        AuthMode::NeoShowcase,
        Arc::new(StubAuthRepository),
    ));

    let req = Request::post("/api/auth/logout")
        .header(header::COOKIE, format!("demo_session={MOCK_SESSION_ID}"))
        .body(Body::empty())
        .unwrap();

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn start_new_run_succeeds() {
    let app = app(AppState::new(AuthMode::Demo, Arc::new(StubAuthRepository)));

    let req = Request::post(format!("/api/rooms/{MOCK_NEW_ROOM_ID}/runs"))
        .header(header::COOKIE, format!("demo_session={MOCK_SESSION_ID}"))
        .body(Body::empty())
        .unwrap();

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::OK);

    let body: serde_json::Value = body_json(response).await;
    assert_eq!(body["status"], "active");
    assert_eq!(body["elapsed_ms"], 0);
    assert_eq!(body["query_count"], 0);
    assert!(body["cleared_problem_ids"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn resume_active_run_succeeds() {
    let app = app(AppState::new(AuthMode::Demo, Arc::new(StubAuthRepository)));

    let req = Request::post(format!("/api/rooms/{MOCK_RESUME_ROOM_ID}/runs"))
        .header(header::COOKIE, format!("demo_session={MOCK_SESSION_ID}"))
        .body(Body::empty())
        .unwrap();

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::OK);

    let body: serde_json::Value = body_json(response).await;
    assert_eq!(body["status"], "active");
    let elapsed = body["elapsed_ms"].as_i64().unwrap();
    assert!(elapsed >= 65000);
}

#[tokio::test]
async fn start_run_unauthorized() {
    let app = app(AppState::new(AuthMode::Demo, Arc::new(StubAuthRepository)));

    let req = Request::post(format!("/api/rooms/{MOCK_NEW_ROOM_ID}/runs"))
        .body(Body::empty())
        .unwrap();

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn start_run_room_not_found() {
    let app = app(AppState::new(AuthMode::Demo, Arc::new(StubAuthRepository)));

    let req = Request::post(format!("/api/rooms/{}/runs", Uuid::nil()))
        .header(header::COOKIE, format!("demo_session={MOCK_SESSION_ID}"))
        .body(Body::empty())
        .unwrap();

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn start_run_invalid_room_id_format() {
    let app = app(AppState::new(AuthMode::Demo, Arc::new(StubAuthRepository)));

    let req = Request::post("/api/rooms/not-a-uuid/runs")
        .header(header::COOKIE, format!("demo_session={MOCK_SESSION_ID}"))
        .body(Body::empty())
        .unwrap();

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
