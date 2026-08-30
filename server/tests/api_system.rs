mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use common::{body_bytes, body_json, request, test_app};
use openapi_generated::models::ErrorResponse;
use serde_json::json;
use std::{
    io::{self, Write},
    sync::{Arc, Mutex},
};
use tracing::instrument::WithSubscriber;
use uuid::Uuid;

#[derive(Clone, Default)]
struct CapturedLogs {
    bytes: Arc<Mutex<Vec<u8>>>,
}

impl CapturedLogs {
    fn contents(&self) -> String {
        let bytes = self
            .bytes
            .lock()
            .expect("captured log lock should not be poisoned");

        String::from_utf8(bytes.clone()).expect("captured logs should be valid UTF-8")
    }
}

impl Write for CapturedLogs {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let mut bytes = self
            .bytes
            .lock()
            .map_err(|_| io::Error::other("captured log lock was poisoned"))?;

        bytes.extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
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
async fn request_log_contains_metadata_without_sensitive_values() {
    let app = test_app();
    let captured_logs = CapturedLogs::default();
    let log_writer = captured_logs.clone();

    let subscriber = tracing_subscriber::fmt()
        .without_time()
        .with_ansi(false)
        .with_target(false)
        .with_writer(move || log_writer.clone())
        .finish();

    let query_secret = "secret-query-value";
    let cookie_secret = "secret-cookie-value";
    let authorization_secret = "secret-authorization-value";
    let forwarded_user_secret = "secret-forwarded-user";
    let display_name_secret = "secret-display-name";

    let req = Request::post(format!("/api/auth/guest?token={query_secret}"))
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::COOKIE, format!("demo_session={cookie_secret}"))
        .header(
            header::AUTHORIZATION,
            format!("Bearer {authorization_secret}"),
        )
        .header("x-forwarded-user", forwarded_user_secret)
        .body(Body::from(
            serde_json::to_vec(&json!({
                "display_name": display_name_secret
            }))
            .unwrap(),
        ))
        .unwrap();

    let response = request(&app, req).with_subscriber(subscriber).await;

    assert_eq!(response.status(), StatusCode::OK);

    let logs = captured_logs.contents();

    assert!(logs.contains("request_id="));
    assert!(logs.contains("method=POST"));
    assert!(logs.contains("matched_route=/api/auth/guest"));
    assert!(logs.contains("status=200"));
    assert!(logs.contains("duration_ms="));
    assert!(logs.contains("request completed"));

    let request_id = logs
        .split("request_id=")
        .nth(1)
        .and_then(|rest| rest.split_whitespace().next())
        .expect("request log should contain a request ID");

    Uuid::parse_str(request_id).expect("request ID should be a UUID");

    for secret in [
        query_secret,
        cookie_secret,
        authorization_secret,
        forwarded_user_secret,
        display_name_secret,
    ] {
        assert!(
            !logs.contains(secret),
            "request log must not contain sensitive value: {secret}"
        );
    }
}
