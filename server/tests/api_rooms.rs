mod common;

use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use common::{
    MOCK_CLEARED_DETAIL_PROBLEM_ID, MOCK_CLEARED_PROBLEM_ID, MOCK_CLEARED_ROOM_ID,
    MOCK_DATABASE_ERROR_PROBLEM_ID, MOCK_LOCKED_PROBLEM_ID, MOCK_NEW_ROOM_ID, MOCK_RESUME_ROOM_ID,
    MOCK_SESSION_ID, StubAuthRepository, body_bytes, body_json, problem_test_app, request,
};
use serde_json::json;
use server::{AppState, app, config::AuthMode};
use uuid::Uuid;

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
    let cleared_ids: Vec<String> = body["cleared_problem_ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    assert_eq!(cleared_ids, vec![MOCK_CLEARED_PROBLEM_ID.to_string()]);
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

    let body: serde_json::Value = body_json(response).await;
    assert_eq!(body["error"]["code"], "BAD_REQUEST");
    assert_eq!(body["error"]["message"], "invalid room_id");
}

#[tokio::test]
async fn start_run_already_cleared_returns_409() {
    let app = app(AppState::new(AuthMode::Demo, Arc::new(StubAuthRepository)));

    let req = Request::post(format!("/api/rooms/{MOCK_CLEARED_ROOM_ID}/runs"))
        .header(header::COOKIE, format!("demo_session={MOCK_SESSION_ID}"))
        .body(Body::empty())
        .unwrap();

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::CONFLICT);

    let body: serde_json::Value = body_json(response).await;
    assert_eq!(body["error"]["code"], "CONFLICT");
    assert_eq!(body["error"]["message"], "room already cleared");
}

#[tokio::test]
async fn get_problems_matches_openapi_fixture() {
    let app = problem_test_app();

    let req = Request::get(format!("/api/rooms/{MOCK_RESUME_ROOM_ID}/problems"))
        .header(header::COOKIE, format!("demo_session={MOCK_SESSION_ID}"))
        .body(Body::empty())
        .unwrap();

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");

    let actual: serde_json::Value = body_json(response).await;
    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../openapi/examples/problems/list-response.json"
    ))
    .expect("OpenAPI fixture should be valid JSON");

    assert_eq!(actual, expected);
}

#[tokio::test]
async fn get_problems_unauthorized() {
    let app = problem_test_app();

    let req = Request::get(format!("/api/rooms/{MOCK_RESUME_ROOM_ID}/problems"))
        .body(Body::empty())
        .unwrap();

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let actual: serde_json::Value = body_json(response).await;
    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../openapi/examples/auth/error-unauthorized.json"
    ))
    .expect("OpenAPI fixture should be valid JSON");

    assert_eq!(actual, expected);
}

#[tokio::test]
async fn get_problems_without_active_run() {
    let app = problem_test_app();

    let req = Request::get(format!("/api/rooms/{MOCK_NEW_ROOM_ID}/problems"))
        .header(header::COOKIE, format!("demo_session={MOCK_SESSION_ID}"))
        .body(Body::empty())
        .unwrap();

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let actual: serde_json::Value = body_json(response).await;
    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../openapi/examples/runs/error-run-not-found.json"
    ))
    .expect("OpenAPI fixture should be valid JSON");

    assert_eq!(actual, expected);
}

#[tokio::test]
async fn get_problems_invalid_room_id_returns_400() {
    let app = problem_test_app();

    let req = Request::get("/api/rooms/not-a-uuid/problems")
        .header(header::COOKIE, format!("demo_session={MOCK_SESSION_ID}"))
        .body(Body::empty())
        .unwrap();

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = body_json(response).await;
    assert_eq!(body["error"]["code"], "BAD_REQUEST");
    assert_eq!(body["error"]["message"], "invalid room_id");
}

#[tokio::test]
async fn get_available_problem_matches_openapi_fixture() {
    let app = problem_test_app();

    let req = Request::get(format!(
        "/api/rooms/{MOCK_RESUME_ROOM_ID}/problems/{MOCK_CLEARED_PROBLEM_ID}"
    ))
    .header(header::COOKIE, format!("demo_session={MOCK_SESSION_ID}"))
    .body(Body::empty())
    .unwrap();

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");

    let actual: serde_json::Value = body_json(response).await;

    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../openapi/examples/problems/available-response.json"
    ))
    .expect("OpenAPI fixture should be valid JSON");

    assert_eq!(actual, expected);
}

#[tokio::test]
async fn get_cleared_problem_succeeds() {
    let app = problem_test_app();

    let req = Request::get(format!(
        "/api/rooms/{MOCK_RESUME_ROOM_ID}/problems/{MOCK_CLEARED_DETAIL_PROBLEM_ID}"
    ))
    .header(header::COOKIE, format!("demo_session={MOCK_SESSION_ID}"))
    .body(Body::empty())
    .unwrap();

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");

    let body: serde_json::Value = body_json(response).await;

    assert_eq!(body["id"], MOCK_CLEARED_DETAIL_PROBLEM_ID);
    assert_eq!(body["status"], "cleared");
}

#[tokio::test]
async fn get_locked_problem_matches_openapi_fixture() {
    let app = problem_test_app();

    let req = Request::get(format!(
        "/api/rooms/{MOCK_RESUME_ROOM_ID}/problems/{MOCK_LOCKED_PROBLEM_ID}"
    ))
    .header(header::COOKIE, format!("demo_session={MOCK_SESSION_ID}"))
    .body(Body::empty())
    .unwrap();

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::CONFLICT);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");

    let actual: serde_json::Value = body_json(response).await;

    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../openapi/examples/problems/error-problem-locked.json"
    ))
    .expect("OpenAPI fixture should be valid JSON");

    assert_eq!(actual, expected);
}

#[tokio::test]
async fn get_missing_problem_returns_404() {
    let app = problem_test_app();
    let missing_problem_id = "99999999-9999-4999-8999-999999999999";

    let req = Request::get(format!(
        "/api/rooms/{MOCK_RESUME_ROOM_ID}/problems/{missing_problem_id}"
    ))
    .header(header::COOKIE, format!("demo_session={MOCK_SESSION_ID}"))
    .body(Body::empty())
    .unwrap();

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");

    let body: serde_json::Value = body_json(response).await;

    assert_eq!(body["error"]["code"], "NOT_FOUND");
    assert_eq!(body["error"]["message"], "problem not found");
    assert_eq!(body["error"]["details"], json!({}));
}

#[tokio::test]
async fn get_problem_unauthorized_matches_openapi_fixture() {
    let app = problem_test_app();

    let req = Request::get(format!(
        "/api/rooms/{MOCK_RESUME_ROOM_ID}/problems/{MOCK_CLEARED_PROBLEM_ID}"
    ))
    .body(Body::empty())
    .unwrap();

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");

    let actual: serde_json::Value = body_json(response).await;

    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../openapi/examples/auth/error-unauthorized.json"
    ))
    .expect("OpenAPI fixture should be valid JSON");

    assert_eq!(actual, expected);
}

#[tokio::test]
async fn get_problem_without_active_run_matches_openapi_fixture() {
    let app = problem_test_app();

    let req = Request::get(format!(
        "/api/rooms/{MOCK_NEW_ROOM_ID}/problems/{MOCK_CLEARED_PROBLEM_ID}"
    ))
    .header(header::COOKIE, format!("demo_session={MOCK_SESSION_ID}"))
    .body(Body::empty())
    .unwrap();

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");

    let actual: serde_json::Value = body_json(response).await;

    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../openapi/examples/runs/error-run-not-found.json"
    ))
    .expect("OpenAPI fixture should be valid JSON");

    assert_eq!(actual, expected);
}

#[tokio::test]
async fn get_problem_invalid_room_id_returns_400() {
    let app = problem_test_app();

    let req = Request::get(format!(
        "/api/rooms/not-a-uuid/problems/{MOCK_CLEARED_PROBLEM_ID}"
    ))
    .header(header::COOKIE, format!("demo_session={MOCK_SESSION_ID}"))
    .body(Body::empty())
    .unwrap();

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");

    let body: serde_json::Value = body_json(response).await;

    assert_eq!(body["error"]["code"], "BAD_REQUEST");
    assert_eq!(body["error"]["message"], "invalid room_id");
    assert_eq!(body["error"]["details"], json!({}));
}

#[tokio::test]
async fn get_problem_invalid_problem_id_returns_400() {
    let app = problem_test_app();

    let req = Request::get(format!(
        "/api/rooms/{MOCK_RESUME_ROOM_ID}/problems/not-a-uuid"
    ))
    .header(header::COOKIE, format!("demo_session={MOCK_SESSION_ID}"))
    .body(Body::empty())
    .unwrap();

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");

    let body: serde_json::Value = body_json(response).await;

    assert_eq!(body["error"]["code"], "BAD_REQUEST");
    assert_eq!(body["error"]["message"], "invalid problem_id");
    assert_eq!(body["error"]["details"], json!({}));
}

#[tokio::test]
async fn get_problem_repository_error_returns_500_without_details() {
    let app = problem_test_app();

    let req = Request::get(format!(
        "/api/rooms/{MOCK_RESUME_ROOM_ID}/problems/{MOCK_DATABASE_ERROR_PROBLEM_ID}"
    ))
    .header(header::COOKIE, format!("demo_session={MOCK_SESSION_ID}"))
    .body(Body::empty())
    .unwrap();

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");

    let body = body_bytes(response).await;
    let body_text = std::str::from_utf8(&body).expect("response body should be UTF-8");

    assert!(
        !body_text.contains("simulated private database failure"),
        "database error details must not be exposed"
    );

    let body: serde_json::Value =
        serde_json::from_slice(&body).expect("response body should be valid JSON");

    assert_eq!(body["error"]["details"], json!({}));
}

#[tokio::test]
async fn get_current_run_succeeds() {
    let app = app(AppState::new(AuthMode::Demo, Arc::new(StubAuthRepository)));

    let req = Request::get(format!("/api/rooms/{MOCK_RESUME_ROOM_ID}/runs/current"))
        .header(header::COOKIE, format!("demo_session={MOCK_SESSION_ID}"))
        .body(Body::empty())
        .unwrap();

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::OK);

    let body: serde_json::Value = body_json(response).await;
    assert_eq!(body["status"], "active");
    let elapsed = body["elapsed_ms"].as_i64().unwrap();
    assert!(elapsed >= 65000);
    let cleared_ids: Vec<String> = body["cleared_problem_ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    assert_eq!(cleared_ids, vec![MOCK_CLEARED_PROBLEM_ID.to_string()]);
}

#[tokio::test]
async fn get_current_run_not_found() {
    let app = app(AppState::new(AuthMode::Demo, Arc::new(StubAuthRepository)));

    let req = Request::get(format!("/api/rooms/{MOCK_NEW_ROOM_ID}/runs/current"))
        .header(header::COOKIE, format!("demo_session={MOCK_SESSION_ID}"))
        .body(Body::empty())
        .unwrap();

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body: serde_json::Value = body_json(response).await;
    assert_eq!(body["error"]["code"], "RUN_NOT_FOUND");
    assert_eq!(body["error"]["message"], "挑戦中のrunが見つかりません");
}

#[tokio::test]
async fn get_current_run_unauthorized() {
    let app = app(AppState::new(AuthMode::Demo, Arc::new(StubAuthRepository)));

    let req = Request::get(format!("/api/rooms/{MOCK_RESUME_ROOM_ID}/runs/current"))
        .body(Body::empty())
        .unwrap();

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn get_current_run_room_not_found() {
    let app = app(AppState::new(AuthMode::Demo, Arc::new(StubAuthRepository)));

    let req = Request::get(format!("/api/rooms/{}/runs/current", Uuid::nil()))
        .header(header::COOKIE, format!("demo_session={MOCK_SESSION_ID}"))
        .body(Body::empty())
        .unwrap();

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body: serde_json::Value = body_json(response).await;
    assert_eq!(body["error"]["code"], "NOT_FOUND");
    assert_eq!(body["error"]["message"], "room not found");
}

#[tokio::test]
async fn get_current_run_invalid_room_id_format() {
    let app = app(AppState::new(AuthMode::Demo, Arc::new(StubAuthRepository)));

    let req = Request::get("/api/rooms/not-a-uuid/runs/current")
        .header(header::COOKIE, format!("demo_session={MOCK_SESSION_ID}"))
        .body(Body::empty())
        .unwrap();

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = body_json(response).await;
    assert_eq!(body["error"]["code"], "BAD_REQUEST");
    assert_eq!(body["error"]["message"], "invalid room_id");
}

#[tokio::test]
async fn get_problem_hint_level1_succeeds() {
    let app = app(AppState::new(AuthMode::Demo, Arc::new(StubAuthRepository)));

    let req = Request::get(format!(
        "/api/rooms/{MOCK_RESUME_ROOM_ID}/problems/{MOCK_CLEARED_PROBLEM_ID}/hints/1"
    ))
    .header(header::COOKIE, format!("demo_session={MOCK_SESSION_ID}"))
    .body(Body::empty())
    .unwrap();

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::OK);

    let actual: serde_json::Value = body_json(response).await;
    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../openapi/examples/problems/hint-level1-response.json"
    ))
    .unwrap();

    assert_eq!(actual, expected);
}

#[tokio::test]
async fn get_problem_hint_level2_succeeds() {
    let app = app(AppState::new(AuthMode::Demo, Arc::new(StubAuthRepository)));

    let req = Request::get(format!(
        "/api/rooms/{MOCK_RESUME_ROOM_ID}/problems/{MOCK_CLEARED_PROBLEM_ID}/hints/2"
    ))
    .header(header::COOKIE, format!("demo_session={MOCK_SESSION_ID}"))
    .body(Body::empty())
    .unwrap();

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::OK);

    let actual: serde_json::Value = body_json(response).await;
    assert_eq!(actual["level"], 2);
    assert_eq!(actual["body_markdown"], "2番目のヒントです");
}

#[tokio::test]
async fn get_problem_hint_unauthorized() {
    let app = app(AppState::new(AuthMode::Demo, Arc::new(StubAuthRepository)));

    let req = Request::get(format!(
        "/api/rooms/{MOCK_RESUME_ROOM_ID}/problems/{MOCK_CLEARED_PROBLEM_ID}/hints/1"
    ))
    .body(Body::empty())
    .unwrap();

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn get_problem_hint_problem_locked() {
    let app = app(AppState::new(AuthMode::Demo, Arc::new(StubAuthRepository)));

    let req = Request::get(format!(
        "/api/rooms/{MOCK_RESUME_ROOM_ID}/problems/{MOCK_LOCKED_PROBLEM_ID}/hints/1"
    ))
    .header(header::COOKIE, format!("demo_session={MOCK_SESSION_ID}"))
    .body(Body::empty())
    .unwrap();

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::CONFLICT);

    let actual: serde_json::Value = body_json(response).await;
    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../openapi/examples/problems/error-problem-locked.json"
    ))
    .unwrap();

    assert_eq!(actual, expected);
}

#[tokio::test]
async fn get_problem_hint_not_found_when_hint_missing() {
    let app = app(AppState::new(AuthMode::Demo, Arc::new(StubAuthRepository)));

    let req = Request::get(format!(
        "/api/rooms/{MOCK_RESUME_ROOM_ID}/problems/{}/hints/1",
        Uuid::nil()
    ))
    .header(header::COOKIE, format!("demo_session={MOCK_SESSION_ID}"))
    .body(Body::empty())
    .unwrap();

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body: serde_json::Value = body_json(response).await;
    assert_eq!(body["error"]["code"], "NOT_FOUND");
}

#[tokio::test]
async fn get_problem_hint_invalid_room_id() {
    let app = app(AppState::new(AuthMode::Demo, Arc::new(StubAuthRepository)));

    let req = Request::get(format!(
        "/api/rooms/not-a-uuid/problems/{MOCK_CLEARED_PROBLEM_ID}/hints/1"
    ))
    .header(header::COOKIE, format!("demo_session={MOCK_SESSION_ID}"))
    .body(Body::empty())
    .unwrap();

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = body_json(response).await;
    assert_eq!(body["error"]["code"], "BAD_REQUEST");
    assert_eq!(body["error"]["message"], "invalid room_id");
}

#[tokio::test]
async fn get_problem_hint_invalid_problem_id() {
    let app = app(AppState::new(AuthMode::Demo, Arc::new(StubAuthRepository)));

    let req = Request::get(format!(
        "/api/rooms/{MOCK_RESUME_ROOM_ID}/problems/not-a-uuid/hints/1"
    ))
    .header(header::COOKIE, format!("demo_session={MOCK_SESSION_ID}"))
    .body(Body::empty())
    .unwrap();

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = body_json(response).await;
    assert_eq!(body["error"]["code"], "BAD_REQUEST");
    assert_eq!(body["error"]["message"], "invalid problem_id");
}

#[tokio::test]
async fn get_problem_hint_invalid_level() {
    let app = app(AppState::new(AuthMode::Demo, Arc::new(StubAuthRepository)));

    for invalid_level in ["0", "-1", "abc"] {
        let req = Request::get(format!(
            "/api/rooms/{MOCK_RESUME_ROOM_ID}/problems/{MOCK_CLEARED_PROBLEM_ID}/hints/{invalid_level}"
        ))
        .header(header::COOKIE, format!("demo_session={MOCK_SESSION_ID}"))
        .body(Body::empty())
        .unwrap();

        let response = request(&app, req).await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body: serde_json::Value = body_json(response).await;
        assert_eq!(body["error"]["code"], "BAD_REQUEST");
        assert_eq!(body["error"]["message"], "invalid hint level");
    }
}

#[tokio::test]
async fn get_problem_hint_not_found_when_level_exceeds_available_hints() {
    let app = app(AppState::new(AuthMode::Demo, Arc::new(StubAuthRepository)));

    let req = Request::get(format!(
        "/api/rooms/{MOCK_RESUME_ROOM_ID}/problems/{MOCK_CLEARED_PROBLEM_ID}/hints/3"
    ))
    .header(header::COOKIE, format!("demo_session={MOCK_SESSION_ID}"))
    .body(Body::empty())
    .unwrap();

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body: serde_json::Value = body_json(response).await;
    assert_eq!(body["error"]["code"], "NOT_FOUND");
    assert_eq!(body["error"]["message"], "hint not found");
}

#[tokio::test]
async fn get_problem_hint_run_not_found() {
    let app = app(AppState::new(AuthMode::Demo, Arc::new(StubAuthRepository)));

    let req = Request::get(format!(
        "/api/rooms/{MOCK_NEW_ROOM_ID}/problems/{MOCK_CLEARED_PROBLEM_ID}/hints/1"
    ))
    .header(header::COOKIE, format!("demo_session={MOCK_SESSION_ID}"))
    .body(Body::empty())
    .unwrap();

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let actual: serde_json::Value = body_json(response).await;
    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../openapi/examples/runs/error-run-not-found.json"
    ))
    .unwrap();

    assert_eq!(actual, expected);
}

#[tokio::test]
async fn get_problem_hint_database_error() {
    let app = app(AppState::new(AuthMode::Demo, Arc::new(StubAuthRepository)));

    let req = Request::get(format!(
        "/api/rooms/{MOCK_RESUME_ROOM_ID}/problems/{MOCK_DATABASE_ERROR_PROBLEM_ID}/hints/1"
    ))
    .header(header::COOKIE, format!("demo_session={MOCK_SESSION_ID}"))
    .body(Body::empty())
    .unwrap();

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let body: serde_json::Value = body_json(response).await;
    assert_eq!(body["error"]["code"], "INTERNAL_SERVER_ERROR");
}
