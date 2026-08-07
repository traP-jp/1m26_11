use std::{collections::HashMap, env, str::FromStr, sync::Arc};

use async_trait::async_trait;
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use server::{
    AppState,
    api::{CreateUserResponse, ErrorResponse, User},
    app, migrate,
    repository::{RepositoryError, SqlxUserRepository, UserRecord, UserRepository},
    service::ReqwestPhotoClient,
};
use sqlx::{
    MySqlPool,
    mysql::{MySqlConnectOptions, MySqlPoolOptions},
};
use tokio::sync::RwLock;
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

const ICON_URL: &str = "https://example.test/thumbnail.png";

#[derive(Default)]
struct MemoryRepository {
    users: RwLock<HashMap<Uuid, UserRecord>>,
}

#[async_trait]
impl UserRepository for MemoryRepository {
    async fn get_users(&self) -> Result<Vec<UserRecord>, RepositoryError> {
        let mut users = self
            .users
            .read()
            .await
            .values()
            .cloned()
            .collect::<Vec<_>>();
        users.sort_by_key(|user| user.id);
        Ok(users)
    }

    async fn create_user(&self, name: &str, email: &str) -> Result<Uuid, RepositoryError> {
        let id = Uuid::new_v4();
        self.users.write().await.insert(
            id,
            UserRecord {
                id,
                name: name.to_owned(),
                email: email.to_owned(),
            },
        );
        Ok(id)
    }

    async fn get_user(&self, user_id: Uuid) -> Result<UserRecord, RepositoryError> {
        self.users
            .read()
            .await
            .get(&user_id)
            .cloned()
            .ok_or(RepositoryError::NotFound)
    }
}

#[tokio::test]
async fn ping_returns_plain_text_pong() {
    let app = test_app(Arc::new(MemoryRepository::default()), "http://127.0.0.1");
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
    let app = test_app(Arc::new(MemoryRepository::default()), "http://127.0.0.1");
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
async fn create_user_validates_request() {
    let app = test_app(Arc::new(MemoryRepository::default()), "http://127.0.0.1");
    let cases = [
        (
            json!({"email": "alice@example.com"}),
            "invalid request body",
        ),
        (
            json!({"name": "   ", "email": "alice@example.com"}),
            "name must not be blank",
        ),
        (
            json!({"name": "Alice", "email": "not-an-email"}),
            "email must be a valid email address",
        ),
        (
            json!({"name": "a".repeat(256), "email": "alice@example.com"}),
            "name must be at most 255 characters",
        ),
        (
            json!({"name": "Alice", "email": format!("{}@example.com", "a".repeat(244))}),
            "email must be at most 255 characters",
        ),
    ];

    for (payload, expected_message) in cases {
        let response = post_json(&app, "/api/v1/users", payload).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let error: ErrorResponse = body_json(response).await;
        assert!(
            error.message.contains(expected_message),
            "unexpected message: {}",
            error.message
        );
    }
}

#[tokio::test]
async fn user_flow_uses_photo_thumbnail_as_icon() {
    let photo_api = MockServer::start().await;
    mount_photo_mock(&photo_api).await;
    let app = test_app(Arc::new(MemoryRepository::default()), &photo_api.uri());

    let response = post_json(
        &app,
        "/api/v1/users",
        json!({"name": "Alice", "email": "alice@example.com"}),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let created: CreateUserResponse = body_json(response).await;

    let response = request(
        &app,
        Request::get(format!("/api/v1/users/{}", created.id))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let user: User = body_json(response).await;
    assert_eq!(user.id, created.id);
    assert_eq!(user.name, "Alice");
    assert_eq!(user.email, "alice@example.com");
    assert_eq!(user.icon_url, ICON_URL);

    let response = request(
        &app,
        Request::get("/api/v1/users").body(Body::empty()).unwrap(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let users: Vec<User> = body_json(response).await;
    assert_eq!(users, vec![user]);
}

#[tokio::test]
async fn missing_user_returns_json_404_without_calling_photo_api() {
    let app = test_app(Arc::new(MemoryRepository::default()), "http://127.0.0.1:9");
    let response = request(
        &app,
        Request::get(format!("/api/v1/users/{}", Uuid::new_v4()))
            .body(Body::empty())
            .unwrap(),
    )
    .await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(
        body_json::<ErrorResponse>(response).await,
        ErrorResponse {
            message: "user not found".to_owned(),
        }
    );
}

#[tokio::test]
async fn invalid_path_and_unknown_route_return_json_errors() {
    let app = test_app(Arc::new(MemoryRepository::default()), "http://127.0.0.1");

    let response = request(
        &app,
        Request::get("/api/v1/users/not-a-uuid")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert!(
        body_json::<ErrorResponse>(response)
            .await
            .message
            .starts_with("invalid userID:")
    );

    let response = request(&app, Request::get("/missing").body(Body::empty()).unwrap()).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(
        body_json::<ErrorResponse>(response).await.message,
        "route not found"
    );
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL or a MariaDB service configured with DB_* variables"]
async fn mariadb_user_flow() {
    let pool = connect_test_database().await;
    migrate(&pool).await.expect("migration should succeed");
    sqlx::query("DELETE FROM users")
        .execute(&pool)
        .await
        .expect("users cleanup should succeed");

    let photo_api = MockServer::start().await;
    mount_photo_mock(&photo_api).await;
    let repository = Arc::new(SqlxUserRepository::new(pool.clone()));
    let app = test_app(repository.clone(), &photo_api.uri());

    let response = post_json(
        &app,
        "/api/v1/users",
        json!({"name": "Database User", "email": "db@example.com"}),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let created: CreateUserResponse = body_json(response).await;
    repository
        .get_user(created.id)
        .await
        .expect("created user should be readable from MariaDB");

    let response = request(
        &app,
        Request::get(format!("/api/v1/users/{}", created.id))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        body_json::<User>(response).await,
        User {
            id: created.id,
            name: "Database User".to_owned(),
            email: "db@example.com".to_owned(),
            icon_url: ICON_URL.to_owned(),
        }
    );

    sqlx::query("DELETE FROM users")
        .execute(&pool)
        .await
        .expect("users cleanup should succeed");
    pool.close().await;
}

fn test_app(repository: Arc<dyn UserRepository>, photo_api_url: &str) -> Router {
    let photos = ReqwestPhotoClient::new(reqwest::Client::new(), photo_api_url)
        .expect("test photo API URL should be valid");
    app(AppState::new(repository, Arc::new(photos)))
}

async fn mount_photo_mock(photo_api: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "albumId": 1,
            "id": 1,
            "title": "icon",
            "url": "https://example.test/photo.png",
            "thumbnailUrl": ICON_URL
        })))
        .mount(photo_api)
        .await;
}

async fn post_json(app: &Router, uri: &str, payload: Value) -> axum::response::Response {
    request(
        app,
        Request::post(uri)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap(),
    )
    .await
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
    let options = match env::var("TEST_DATABASE_URL") {
        Ok(url) => MySqlConnectOptions::from_str(&url).expect("TEST_DATABASE_URL should be valid"),
        Err(_) => MySqlConnectOptions::new()
            .host(&env_or("DB_HOST", "localhost"))
            .port(
                env_or("DB_PORT", "3306")
                    .parse()
                    .expect("DB_PORT should be a valid port"),
            )
            .username(&env_or("DB_USER", "root"))
            .password(&env_or("DB_PASS", "pass"))
            .database(&env_or("DB_NAME", "app")),
    };

    MySqlPoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .expect("test database should be reachable")
}

fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_owned())
}
