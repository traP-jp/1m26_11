mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use common::{
    MOCK_CLEARED_DETAIL_PROBLEM_ID, MOCK_CLEARED_PROBLEM_ID, MOCK_DATABASE_ERROR_PROBLEM_ID,
    MOCK_LOCKED_PROBLEM_ID, MOCK_NEW_ROOM_ID, MOCK_RESUME_ROOM_ID, MOCK_SESSION_ID, body_bytes,
    body_json, problem_test_app, request,
};
use serde_json::json;
use uuid::Uuid;

fn authenticated_query_request(
    room_id: &str,
    problem_id: &str,
    payload: serde_json::Value,
) -> Request<Body> {
    Request::post(format!(
        "/api/rooms/{room_id}/problems/{problem_id}/queries"
    ))
    .header(header::CONTENT_TYPE, "application/json")
    .header(header::COOKIE, format!("demo_session={MOCK_SESSION_ID}"))
    .body(Body::from(serde_json::to_vec(&payload).unwrap()))
    .unwrap()
}

fn serial_query_payload() -> serde_json::Value {
    serde_json::from_str(include_str!(
        "../../openapi/examples/queries/request-serial.json"
    ))
    .expect("OpenAPI request fixture should be valid JSON")
}

#[tokio::test]
async fn submit_query_correct_matches_openapi_fixture() {
    let app = problem_test_app();

    let payload: serde_json::Value = serde_json::from_str(include_str!(
        "../../openapi/examples/queries/request-serial.json"
    ))
    .expect("OpenAPI request fixture should be valid JSON");

    let response = request(
        &app,
        authenticated_query_request(MOCK_RESUME_ROOM_ID, MOCK_CLEARED_PROBLEM_ID, payload),
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");

    let actual: serde_json::Value = body_json(response).await;

    let query_id = actual["query_id"]
        .as_str()
        .expect("query_id should be a string");

    Uuid::parse_str(query_id).expect("query_id should be a UUID");

    let mut expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../openapi/examples/queries/response-correct.json"
    ))
    .expect("OpenAPI response fixture should be valid JSON");

    expected["query_id"] = actual["query_id"].clone();

    assert_eq!(actual, expected);
}

#[tokio::test]
async fn submit_query_incorrect_succeeds() {
    let app = problem_test_app();

    let payload = json!({
        "source": "mouse",
        "operations": [
            {
                "control": "down",
                "count": 16
            },
            {
                "control": "right",
                "count": 1
            }
        ]
    });

    let response = request(
        &app,
        authenticated_query_request(MOCK_RESUME_ROOM_ID, MOCK_CLEARED_PROBLEM_ID, payload),
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);

    let actual: serde_json::Value = body_json(response).await;

    Uuid::parse_str(
        actual["query_id"]
            .as_str()
            .expect("query_id should be a string"),
    )
    .expect("query_id should be a UUID");

    assert_eq!(actual["correct"], false);
    assert_eq!(
        actual["normalized_operations"],
        json!([
            {
                "control": "down",
                "count": 16
            },
            {
                "control": "right",
                "count": 1
            }
        ])
    );
    assert_eq!(actual["remaining_pattern_count"], 1);
    assert_eq!(actual["query_count"], 4);
    assert_eq!(actual["problem_status"], "available");
}

#[tokio::test]
async fn submit_query_invalid_source_matches_openapi_fixture() {
    let app = problem_test_app();

    let payload: serde_json::Value = serde_json::from_str(include_str!(
        "../../openapi/examples/queries/request-invalid-source.json"
    ))
    .expect("OpenAPI request fixture should be valid JSON");

    let response = request(
        &app,
        authenticated_query_request(MOCK_RESUME_ROOM_ID, MOCK_CLEARED_PROBLEM_ID, payload),
    )
    .await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let actual: serde_json::Value = body_json(response).await;

    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../openapi/examples/queries/error-validation.json"
    ))
    .expect("OpenAPI response fixture should be valid JSON");

    assert_eq!(actual, expected);
}

#[tokio::test]
async fn submit_query_locked_problem_matches_openapi_fixture() {
    let app = problem_test_app();

    let payload: serde_json::Value = serde_json::from_str(include_str!(
        "../../openapi/examples/queries/request-serial.json"
    ))
    .expect("OpenAPI request fixture should be valid JSON");

    let response = request(
        &app,
        authenticated_query_request(MOCK_RESUME_ROOM_ID, MOCK_LOCKED_PROBLEM_ID, payload),
    )
    .await;

    assert_eq!(response.status(), StatusCode::CONFLICT);

    let actual: serde_json::Value = body_json(response).await;

    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../openapi/examples/problems/error-problem-locked.json"
    ))
    .expect("OpenAPI response fixture should be valid JSON");

    assert_eq!(actual, expected);
}

#[tokio::test]
async fn submit_query_cleared_problem_matches_openapi_fixture() {
    let app = problem_test_app();

    let payload: serde_json::Value = serde_json::from_str(include_str!(
        "../../openapi/examples/queries/request-serial.json"
    ))
    .expect("OpenAPI request fixture should be valid JSON");

    let response = request(
        &app,
        authenticated_query_request(MOCK_RESUME_ROOM_ID, MOCK_CLEARED_DETAIL_PROBLEM_ID, payload),
    )
    .await;

    assert_eq!(response.status(), StatusCode::CONFLICT);

    let actual: serde_json::Value = body_json(response).await;

    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../openapi/examples/problems/error-problem-already-cleared.json"
    ))
    .expect("OpenAPI response fixture should be valid JSON");

    assert_eq!(actual, expected);
}

#[tokio::test]
async fn submit_query_unauthorized_matches_openapi_fixture() {
    let app = problem_test_app();
    let payload = serial_query_payload();

    let req = Request::post(format!(
        "/api/rooms/{MOCK_RESUME_ROOM_ID}/problems/{MOCK_CLEARED_PROBLEM_ID}/queries"
    ))
    .header(header::CONTENT_TYPE, "application/json")
    .body(Body::from(serde_json::to_vec(&payload).unwrap()))
    .unwrap();

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let actual: serde_json::Value = body_json(response).await;

    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../openapi/examples/auth/error-unauthorized.json"
    ))
    .expect("OpenAPI response fixture should be valid JSON");

    assert_eq!(actual, expected);
}

#[tokio::test]
async fn submit_query_without_active_run_matches_openapi_fixture() {
    let app = problem_test_app();

    let response = request(
        &app,
        authenticated_query_request(
            MOCK_NEW_ROOM_ID,
            MOCK_CLEARED_PROBLEM_ID,
            serial_query_payload(),
        ),
    )
    .await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let actual: serde_json::Value = body_json(response).await;

    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../openapi/examples/runs/error-run-not-found.json"
    ))
    .expect("OpenAPI response fixture should be valid JSON");

    assert_eq!(actual, expected);
}

#[tokio::test]
async fn submit_query_invalid_room_id_returns_400() {
    let app = problem_test_app();

    let response = request(
        &app,
        authenticated_query_request(
            "not-a-uuid",
            MOCK_CLEARED_PROBLEM_ID,
            serial_query_payload(),
        ),
    )
    .await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = body_json(response).await;

    assert_eq!(body["error"]["code"], "BAD_REQUEST");
    assert_eq!(body["error"]["message"], "invalid room_id");
    assert_eq!(body["error"]["details"], json!({}));
}

#[tokio::test]
async fn submit_query_invalid_problem_id_returns_400() {
    let app = problem_test_app();

    let response = request(
        &app,
        authenticated_query_request(MOCK_RESUME_ROOM_ID, "not-a-uuid", serial_query_payload()),
    )
    .await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = body_json(response).await;

    assert_eq!(body["error"]["code"], "BAD_REQUEST");
    assert_eq!(body["error"]["message"], "invalid problem_id");
    assert_eq!(body["error"]["details"], json!({}));
}

#[tokio::test]
async fn submit_query_missing_problem_returns_404() {
    let app = problem_test_app();
    let missing_problem_id = "99999999-9999-4999-8999-999999999999";

    let response = request(
        &app,
        authenticated_query_request(
            MOCK_RESUME_ROOM_ID,
            missing_problem_id,
            serial_query_payload(),
        ),
    )
    .await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body: serde_json::Value = body_json(response).await;

    assert_eq!(body["error"]["code"], "NOT_FOUND");
    assert_eq!(body["error"]["message"], "problem not found");
    assert_eq!(body["error"]["details"], json!({}));
}

#[tokio::test]
async fn submit_query_repository_error_returns_500_without_details() {
    let app = problem_test_app();

    let response = request(
        &app,
        authenticated_query_request(
            MOCK_RESUME_ROOM_ID,
            MOCK_DATABASE_ERROR_PROBLEM_ID,
            serial_query_payload(),
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
    assert_eq!(body["error"]["details"], json!({}));
}

#[tokio::test]
async fn submit_query_malformed_json_returns_json_400() {
    let app = problem_test_app();

    let req = Request::post(format!(
        "/api/rooms/{MOCK_RESUME_ROOM_ID}/problems/{MOCK_CLEARED_PROBLEM_ID}/queries"
    ))
    .header(header::CONTENT_TYPE, "application/json")
    .header(header::COOKIE, format!("demo_session={MOCK_SESSION_ID}"))
    .body(Body::from(r#"{"source":"serial","operations":["#))
    .unwrap();

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");

    let body: serde_json::Value = body_json(response).await;

    assert_eq!(body["error"]["code"], "BAD_REQUEST");
    assert_eq!(body["error"]["message"], "invalid request body");
    assert_eq!(body["error"]["details"], json!({}));
}

#[tokio::test]
async fn submit_query_missing_operations_returns_json_400() {
    let app = problem_test_app();

    let payload = json!({
        "source": "serial"
    });

    let response = request(
        &app,
        authenticated_query_request(MOCK_RESUME_ROOM_ID, MOCK_CLEARED_PROBLEM_ID, payload),
    )
    .await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");

    let body: serde_json::Value = body_json(response).await;

    assert_eq!(body["error"]["code"], "BAD_REQUEST");
    assert_eq!(body["error"]["message"], "invalid request body");
    assert_eq!(body["error"]["details"], json!({}));
}
