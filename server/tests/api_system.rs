mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use common::{body_bytes, body_json, request, test_app};
use openapi_generated::models::ErrorResponse;
use serde_json::json;

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
