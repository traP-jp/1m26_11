mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use common::{
    MOCK_CLEARED_DETAIL_PROBLEM_ID, MOCK_CLEARED_PROBLEM_ID, MOCK_DATABASE_ERROR_PROBLEM_ID,
    MOCK_LOCKED_PROBLEM_ID, MOCK_NEW_ROOM_ID, MOCK_RESUME_ROOM_ID, MOCK_SESSION_ID,
    MOCK_STRING_PROBLEM_ID, body_bytes, body_json, request, test_app,
};

fn authenticated_answer_request(
    room_id: &str,
    problem_id: &str,
    payload: serde_json::Value,
) -> Request<Body> {
    Request::post(format!(
        "/api/rooms/{room_id}/problems/{problem_id}/answers"
    ))
    .header(header::CONTENT_TYPE, "application/json")
    .header(header::COOKIE, format!("demo_session={MOCK_SESSION_ID}"))
    .body(Body::from(
        serde_json::to_vec(&payload).expect("request payload should serialize"),
    ))
    .expect("request should be valid")
}

async fn assert_fixture_response(
    response: axum::response::Response,
    expected_status: StatusCode,
    fixture: &str,
) {
    assert_eq!(response.status(), expected_status);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");

    let actual: serde_json::Value = body_json(response).await;

    let expected: serde_json::Value =
        serde_json::from_str(fixture).expect("OpenAPI response fixture should be valid JSON");

    assert_eq!(actual, expected);
}

fn valid_answer_payload() -> serde_json::Value {
    serde_json::from_str(include_str!("../../openapi/examples/answers/request.json"))
        .expect("OpenAPI request fixture should be valid JSON")
}

#[tokio::test]
async fn submit_answer_correct_matches_openapi_fixture() {
    let app = test_app();

    let payload: serde_json::Value =
        serde_json::from_str(include_str!("../../openapi/examples/answers/request.json"))
            .expect("OpenAPI request fixture should be valid JSON");

    let response = request(
        &app,
        authenticated_answer_request(MOCK_RESUME_ROOM_ID, MOCK_STRING_PROBLEM_ID, payload),
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");

    let actual: serde_json::Value = body_json(response).await;

    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../openapi/examples/answers/response-correct-unlock.json"
    ))
    .expect("OpenAPI response fixture should be valid JSON");

    assert_eq!(actual, expected);
}

#[tokio::test]
async fn submit_answer_incorrect_matches_openapi_fixture() {
    let app = test_app();

    let payload = serde_json::json!({
        "answer": "wrong answer"
    });

    let response = request(
        &app,
        authenticated_answer_request(MOCK_RESUME_ROOM_ID, MOCK_STRING_PROBLEM_ID, payload),
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");

    let actual: serde_json::Value = body_json(response).await;

    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../openapi/examples/answers/response-incorrect.json"
    ))
    .expect("OpenAPI response fixture should be valid JSON");

    assert_eq!(actual, expected);
}

#[tokio::test]
async fn submit_answer_unauthorized_matches_openapi_fixture() {
    let app = test_app();

    let req = Request::post(format!(
        "/api/rooms/{MOCK_RESUME_ROOM_ID}/problems/{MOCK_STRING_PROBLEM_ID}/answers"
    ))
    .header(header::CONTENT_TYPE, "application/json")
    .body(Body::from(
        serde_json::to_vec(&valid_answer_payload()).unwrap(),
    ))
    .unwrap();

    let response = request(&app, req).await;

    assert_fixture_response(
        response,
        StatusCode::UNAUTHORIZED,
        include_str!("../../openapi/examples/auth/error-unauthorized.json"),
    )
    .await;
}

#[tokio::test]
async fn submit_answer_without_active_run_matches_openapi_fixture() {
    let app = test_app();

    let response = request(
        &app,
        authenticated_answer_request(
            MOCK_NEW_ROOM_ID,
            MOCK_STRING_PROBLEM_ID,
            valid_answer_payload(),
        ),
    )
    .await;

    assert_fixture_response(
        response,
        StatusCode::NOT_FOUND,
        include_str!("../../openapi/examples/runs/error-run-not-found.json"),
    )
    .await;
}

#[tokio::test]
async fn submit_answer_locked_problem_matches_openapi_fixture() {
    let app = test_app();

    let response = request(
        &app,
        authenticated_answer_request(
            MOCK_RESUME_ROOM_ID,
            MOCK_LOCKED_PROBLEM_ID,
            valid_answer_payload(),
        ),
    )
    .await;

    assert_fixture_response(
        response,
        StatusCode::CONFLICT,
        include_str!("../../openapi/examples/problems/error-problem-locked.json"),
    )
    .await;
}

#[tokio::test]
async fn submit_answer_cleared_problem_matches_openapi_fixture() {
    let app = test_app();

    let response = request(
        &app,
        authenticated_answer_request(
            MOCK_RESUME_ROOM_ID,
            MOCK_CLEARED_DETAIL_PROBLEM_ID,
            valid_answer_payload(),
        ),
    )
    .await;

    assert_fixture_response(
        response,
        StatusCode::CONFLICT,
        include_str!("../../openapi/examples/problems/error-problem-already-cleared.json"),
    )
    .await;
}

#[tokio::test]
async fn submit_answer_operation_sequence_problem_returns_422() {
    let app = test_app();

    let response = request(
        &app,
        authenticated_answer_request(
            MOCK_RESUME_ROOM_ID,
            MOCK_CLEARED_PROBLEM_ID,
            valid_answer_payload(),
        ),
    )
    .await;

    assert_fixture_response(
        response,
        StatusCode::UNPROCESSABLE_ENTITY,
        include_str!("../../openapi/examples/queries/error-validation.json"),
    )
    .await;
}

#[tokio::test]
async fn submit_answer_too_long_returns_422() {
    let app = test_app();

    let payload: serde_json::Value = serde_json::from_str(include_str!(
        "../../openapi/examples/answers/request-too-long.json"
    ))
    .expect("OpenAPI request fixture should be valid JSON");

    let response = request(
        &app,
        authenticated_answer_request(MOCK_RESUME_ROOM_ID, MOCK_STRING_PROBLEM_ID, payload),
    )
    .await;

    assert_fixture_response(
        response,
        StatusCode::UNPROCESSABLE_ENTITY,
        include_str!("../../openapi/examples/queries/error-validation.json"),
    )
    .await;
}

#[tokio::test]
async fn submit_answer_normalized_empty_returns_422() {
    let app = test_app();

    let response = request(
        &app,
        authenticated_answer_request(
            MOCK_RESUME_ROOM_ID,
            MOCK_STRING_PROBLEM_ID,
            serde_json::json!({
                "answer": "  \t　"
            }),
        ),
    )
    .await;

    assert_fixture_response(
        response,
        StatusCode::UNPROCESSABLE_ENTITY,
        include_str!("../../openapi/examples/queries/error-validation.json"),
    )
    .await;
}

#[tokio::test]
async fn submit_answer_invalid_room_id_returns_400() {
    let app = test_app();

    let response = request(
        &app,
        authenticated_answer_request("not-a-uuid", MOCK_STRING_PROBLEM_ID, valid_answer_payload()),
    )
    .await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = body_json(response).await;

    assert_eq!(body["error"]["code"], "BAD_REQUEST");
    assert_eq!(body["error"]["message"], "invalid room_id");
    assert_eq!(body["error"]["details"], serde_json::json!({}));
}

#[tokio::test]
async fn submit_answer_invalid_problem_id_returns_400() {
    let app = test_app();

    let response = request(
        &app,
        authenticated_answer_request(MOCK_RESUME_ROOM_ID, "not-a-uuid", valid_answer_payload()),
    )
    .await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = body_json(response).await;

    assert_eq!(body["error"]["code"], "BAD_REQUEST");
    assert_eq!(body["error"]["message"], "invalid problem_id");
    assert_eq!(body["error"]["details"], serde_json::json!({}));
}

#[tokio::test]
async fn submit_answer_missing_problem_returns_404() {
    let app = test_app();
    let missing_problem_id = "99999999-9999-4999-8999-999999999999";

    let response = request(
        &app,
        authenticated_answer_request(
            MOCK_RESUME_ROOM_ID,
            missing_problem_id,
            valid_answer_payload(),
        ),
    )
    .await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body: serde_json::Value = body_json(response).await;

    assert_eq!(body["error"]["code"], "NOT_FOUND");
    assert_eq!(body["error"]["message"], "problem not found");
    assert_eq!(body["error"]["details"], serde_json::json!({}));
}

#[tokio::test]
async fn submit_answer_malformed_json_returns_400() {
    let app = test_app();

    let req = Request::post(format!(
        "/api/rooms/{MOCK_RESUME_ROOM_ID}/problems/{MOCK_STRING_PROBLEM_ID}/answers"
    ))
    .header(header::CONTENT_TYPE, "application/json")
    .header(header::COOKIE, format!("demo_session={MOCK_SESSION_ID}"))
    .body(Body::from(r#"{"answer":"#))
    .unwrap();

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");

    let body: serde_json::Value = body_json(response).await;

    assert_eq!(body["error"]["code"], "BAD_REQUEST");
    assert_eq!(body["error"]["message"], "invalid request body");
    assert_eq!(body["error"]["details"], serde_json::json!({}));
}

#[tokio::test]
async fn submit_answer_missing_answer_returns_400() {
    let app = test_app();

    let response = request(
        &app,
        authenticated_answer_request(
            MOCK_RESUME_ROOM_ID,
            MOCK_STRING_PROBLEM_ID,
            serde_json::json!({}),
        ),
    )
    .await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = body_json(response).await;

    assert_eq!(body["error"]["code"], "BAD_REQUEST");
    assert_eq!(body["error"]["message"], "invalid request body");
    assert_eq!(body["error"]["details"], serde_json::json!({}));
}

#[tokio::test]
async fn submit_answer_repository_error_returns_500_without_details() {
    let app = test_app();

    let response = request(
        &app,
        authenticated_answer_request(
            MOCK_RESUME_ROOM_ID,
            MOCK_DATABASE_ERROR_PROBLEM_ID,
            valid_answer_payload(),
        ),
    )
    .await;

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let body = body_bytes(response).await;
    let body_text = std::str::from_utf8(&body).expect("response body should be UTF-8");

    assert!(
        !body_text.contains("simulated private database failure"),
        "database error details must not be exposed"
    );

    let body: serde_json::Value =
        serde_json::from_slice(&body).expect("response body should be valid JSON");

    assert_eq!(body["error"]["code"], "INTERNAL_SERVER_ERROR");
    assert_eq!(body["error"]["message"], "internal server error");
    assert_eq!(body["error"]["details"], serde_json::json!({}));
}
