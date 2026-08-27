mod common;

use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use common::{
    MOCK_SESSION_ID, RecordingAuthRepository, StubAuthRepository, body_bytes, body_json, request,
};
use serde_json::json;
use server::{AppState, app, config::AuthMode};
use uuid::Uuid;

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
async fn guest_login_records_created_demo_session() {
    let user_id = Uuid::new_v4();
    let repository = Arc::new(RecordingAuthRepository::new(user_id));
    let app = app(AppState::new(AuthMode::Demo, repository.clone()));
    let request_payload = json!({
        "display_name": "hoge"
    });
    let req = Request::post("/api/auth/guest")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&request_payload).unwrap()))
        .unwrap();

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::OK);
    let cookie = response
        .headers()
        .get(header::SET_COOKIE)
        .expect("Set-Cookie header should be present")
        .to_str()
        .expect("Set-Cookie header should contain valid text");
    let cookie_pair = cookie
        .split(';')
        .next()
        .expect("session cookie should contain a name-value pair");
    let (cookie_name, cookie_value) = cookie_pair
        .split_once('=')
        .expect("session cookie should contain an equals sign");
    assert_eq!(cookie_name, "demo_session");
    let session_id = Uuid::parse_str(cookie_value).expect("session cookie should contain a UUID");

    let calls = repository
        .demo_session_calls
        .lock()
        .expect("demo session call log should not be poisoned");
    assert_eq!(calls.created.as_slice(), &[(session_id, user_id)]);
    assert!(calls.deleted.is_empty());
}

#[tokio::test]
async fn guest_logout_records_deleted_demo_session_and_returns_empty_body() {
    let session_id = Uuid::parse_str("55555555-5555-4555-8555-555555555555").unwrap();
    let repository = Arc::new(RecordingAuthRepository::new(Uuid::new_v4()));
    let app = app(AppState::new(AuthMode::Demo, repository.clone()));
    let req = Request::post("/api/auth/logout")
        .header(header::COOKIE, format!("demo_session={session_id}"))
        .body(Body::empty())
        .unwrap();

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    {
        let calls = repository
            .demo_session_calls
            .lock()
            .expect("demo session call log should not be poisoned");
        assert!(calls.created.is_empty());
        assert_eq!(calls.deleted.as_slice(), &[session_id]);
    }
    assert!(body_bytes(response).await.is_empty());
}
