use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
};
use http_body_util::BodyExt;
use server::{
    AppState, app,
    config::AuthMode,
    repository::{AuthRepository, AuthUserRecord, RepositoryError},
};
use tower::ServiceExt;
use uuid::Uuid;

const DEMO_AUTHENTICATED: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../openapi/examples/auth/me-demo-authenticated.json"
));

const DEMO_UNAUTHENTICATED: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../openapi/examples/auth/me-demo-unauthenticated.json"
));

const NEOSHOWCASE_AUTHENTICATED: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../openapi/examples/auth/me-neoshowcase-authenticated.json"
));

const NEOSHOWCASE_UNAUTHENTICATED: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../openapi/examples/auth/me-neoshowcase-unauthenticated.json"
));

struct StubAuthRepository {
    expected_demo_session_id: Option<Uuid>,
    user: Option<AuthUserRecord>,
}

#[async_trait]
impl AuthRepository for StubAuthRepository {
    async fn find_user_by_demo_session(
        &self,
        session_id: Uuid,
    ) -> Result<Option<AuthUserRecord>, RepositoryError> {
        let expected_session_id = self
            .expected_demo_session_id
            .expect("demo session lookup was not expected");

        assert_eq!(session_id, expected_session_id);

        Ok(self.user.clone())
    }

    async fn find_user_by_provider_subject(
        &self,
        _auth_provider: &str,
        _provider_subject: &str,
    ) -> Result<Option<AuthUserRecord>, RepositoryError> {
        panic!("direct provider lookup was not expected");
    }

    async fn get_or_create_user(
        &self,
        auth_provider: &str,
        provider_subject: &str,
        display_name: &str,
    ) -> Result<AuthUserRecord, RepositoryError> {
        let user = self
            .user
            .clone()
            .expect("NeoShowcase user lookup was not expected");

        assert_eq!(auth_provider, "neoshowcase");
        assert_eq!(provider_subject, user.display_name);
        assert_eq!(display_name, user.display_name);

        Ok(user)
    }
}

#[tokio::test]
async fn demo_authenticated_response_matches_fixture() {
    let session_id = Uuid::new_v4();
    let user_id = Uuid::parse_str("55555555-5555-4555-8555-555555555555")
        .expect("fixture user ID should be valid");

    let app = test_app(
        AuthMode::Demo,
        StubAuthRepository {
            expected_demo_session_id: Some(session_id),
            user: Some(AuthUserRecord {
                id: user_id,
                display_name: "kaomojikun".to_owned(),
            }),
        },
    );

    let request = Request::get("/api/me")
        .header(header::COOKIE, format!("demo_session={session_id}"))
        .body(Body::empty())
        .expect("request should be valid");

    assert_response_matches_fixture(&app, request, DEMO_AUTHENTICATED).await;
}

#[tokio::test]
async fn demo_unauthenticated_response_matches_fixture() {
    let app = test_app(
        AuthMode::Demo,
        StubAuthRepository {
            expected_demo_session_id: None,
            user: None,
        },
    );

    let request = Request::get("/api/me")
        .body(Body::empty())
        .expect("request should be valid");

    assert_response_matches_fixture(&app, request, DEMO_UNAUTHENTICATED).await;
}

#[tokio::test]
async fn neoshowcase_authenticated_response_matches_fixture() {
    let user_id = Uuid::parse_str("44444444-4444-4444-8444-444444444444")
        .expect("fixture user ID should be valid");

    let app = test_app(
        AuthMode::NeoShowcase,
        StubAuthRepository {
            expected_demo_session_id: None,
            user: Some(AuthUserRecord {
                id: user_id,
                display_name: "kaomojikun".to_owned(),
            }),
        },
    );

    let request = Request::get("/api/me")
        .header("x-forwarded-user", "kaomojikun")
        .body(Body::empty())
        .expect("request should be valid");

    assert_response_matches_fixture(&app, request, NEOSHOWCASE_AUTHENTICATED).await;
}

#[tokio::test]
async fn neoshowcase_unauthenticated_response_matches_fixture() {
    let app = test_app(
        AuthMode::NeoShowcase,
        StubAuthRepository {
            expected_demo_session_id: None,
            user: None,
        },
    );

    let request = Request::get("/api/me")
        .body(Body::empty())
        .expect("request should be valid");

    assert_response_matches_fixture(&app, request, NEOSHOWCASE_UNAUTHENTICATED).await;
}

fn test_app(auth_mode: AuthMode, repository: StubAuthRepository) -> Router {
    app(AppState::new(auth_mode, Arc::new(repository)))
}

async fn assert_response_matches_fixture(app: &Router, request: Request<Body>, fixture: &str) {
    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("request should succeed");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");

    let body = response
        .into_body()
        .collect()
        .await
        .expect("response body should be readable")
        .to_bytes();

    let actual: serde_json::Value =
        serde_json::from_slice(&body).expect("response body should be valid JSON");

    let expected: serde_json::Value =
        serde_json::from_str(fixture).expect("fixture should be valid JSON");

    assert_eq!(actual, expected);
}
