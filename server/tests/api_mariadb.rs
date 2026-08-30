mod common;

use std::{path::PathBuf, sync::Arc};

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
};
use common::{body_bytes, body_json, connect_test_database, request};
use serde_json::{Value, json};
use server::{
    AppState, app,
    config::AuthMode,
    migrate,
    problem::{load_problem_data, seed_problem_data},
    repository::SqlxUserRepository,
};
use sqlx::MySqlPool;
use uuid::Uuid;

const AUTH_TEST_SUBJECT_PREFIX: &str = "issue79-auth-";
const NEO_AUTH_TEST_SUBJECT_PREFIX: &str = "issue79-neo-auth-";
const DEMO_HEADER_TEST_SUBJECT_PREFIX: &str = "issue79-demo-header-";
const RUN_TEST_SUBJECT_PREFIX: &str = "issue79-run-";
const GAME_ROOM_ID: &str = "1411824c-d357-4941-af76-c76cb827dda6";
const GAME_PROBLEM_IDS: [&str; 4] = [
    "52ed5a58-bc88-4e0f-97a4-0f64a112acd4",
    "9ebaa649-9c28-4bed-9dc1-fd7b9fedaa9b",
    "6853a228-0462-4413-91f4-6b8ef672cefc",
    "9ca65619-6ad2-4e74-bf4a-4f146b238067",
];

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL pointing to a disposable MariaDB database"]
async fn mariadb_demo_auth_http_flow() {
    let pool = connect_test_database().await;
    migrate(&pool).await.expect("migration should succeed");
    cleanup_users_with_subject_prefix(&pool, AUTH_TEST_SUBJECT_PREFIX).await;

    let suffix = Uuid::new_v4().simple().to_string();
    let display_name = format!("{AUTH_TEST_SUBJECT_PREFIX}{}", &suffix[..8]);

    let repository = Arc::new(SqlxUserRepository::new(pool.clone()));
    let app = app(AppState::new(AuthMode::Demo, repository).with_demo_cookie_secure(false));

    let (first_user_id, first_session_id, first_cookie) = login_guest(&app, &display_name).await;

    let stored_user_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT user_id
        FROM users
        WHERE auth_provider = 'demo'
          AND provider_subject = ?
        "#,
    )
    .bind(&display_name)
    .fetch_one(&pool)
    .await
    .expect("created demo user should be stored");

    assert_eq!(stored_user_id, first_user_id);

    let session_user_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT user_id
        FROM demo_sessions
        WHERE session_id = ?
        "#,
    )
    .bind(first_session_id)
    .fetch_one(&pool)
    .await
    .expect("created demo session should be stored");

    assert_eq!(session_user_id, first_user_id);

    let me_response = request(
        &app,
        Request::get("/api/me")
            .header(header::COOKIE, &first_cookie)
            .body(Body::empty())
            .expect("me request should be valid"),
    )
    .await;

    assert_eq!(me_response.status(), StatusCode::OK);

    let me_body: Value = body_json(me_response).await;
    assert_eq!(me_body["authenticated"], true);
    assert_eq!(me_body["auth_mode"], "demo");
    assert_eq!(me_body["user"]["id"], first_user_id.to_string());
    assert_eq!(me_body["user"]["display_name"], display_name);

    logout_guest(&app, &first_cookie).await;

    let session_count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM demo_sessions WHERE session_id = ?")
            .bind(first_session_id)
            .fetch_one(&pool)
            .await
            .expect("session count should be readable");

    assert_eq!(session_count, 0);

    let logged_out_me_response = request(
        &app,
        Request::get("/api/me")
            .header(header::COOKIE, &first_cookie)
            .body(Body::empty())
            .expect("logged-out me request should be valid"),
    )
    .await;

    assert_eq!(logged_out_me_response.status(), StatusCode::OK);

    let logged_out_me_body: Value = body_json(logged_out_me_response).await;
    assert_eq!(logged_out_me_body["authenticated"], false);
    assert_eq!(logged_out_me_body["user"], Value::Null);

    let (second_user_id, _second_session_id, second_cookie) =
        login_guest(&app, &display_name).await;

    assert_eq!(
        second_user_id, first_user_id,
        "logging in with the same display name should reuse the demo user",
    );

    let matching_user_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM users
        WHERE auth_provider = 'demo'
          AND provider_subject = ?
        "#,
    )
    .bind(&display_name)
    .fetch_one(&pool)
    .await
    .expect("matching user count should be readable");

    assert_eq!(matching_user_count, 1);

    logout_guest(&app, &second_cookie).await;

    cleanup_users_with_subject_prefix(&pool, AUTH_TEST_SUBJECT_PREFIX).await;
    pool.close().await;
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL pointing to a disposable MariaDB database"]
async fn mariadb_neoshowcase_auth_http_flow() {
    let pool = connect_test_database().await;
    migrate(&pool).await.expect("migration should succeed");
    cleanup_users_with_subject_prefix(&pool, NEO_AUTH_TEST_SUBJECT_PREFIX).await;

    let suffix = Uuid::new_v4().simple().to_string();
    let provider_subject = format!("{NEO_AUTH_TEST_SUBJECT_PREFIX}{}", &suffix[..8]);

    let repository = Arc::new(SqlxUserRepository::new(pool.clone()));
    let app = app(AppState::new(AuthMode::NeoShowcase, repository));

    let unauthenticated_response = request(
        &app,
        Request::get("/api/me")
            .body(Body::empty())
            .expect("unauthenticated me request should be valid"),
    )
    .await;

    assert_eq!(unauthenticated_response.status(), StatusCode::OK);

    let unauthenticated_body: Value = body_json(unauthenticated_response).await;

    assert_eq!(unauthenticated_body["authenticated"], false);
    assert_eq!(unauthenticated_body["auth_mode"], "neoshowcase");
    assert_eq!(unauthenticated_body["user"], Value::Null);
    assert_eq!(
        unauthenticated_body["login_url"],
        "/_oauth/login?redirect=/"
    );
    assert_eq!(unauthenticated_body["logout_url"], Value::Null);

    let authenticated_response = request(
        &app,
        Request::get("/api/me")
            .header("x-forwarded-user", &provider_subject)
            .body(Body::empty())
            .expect("authenticated me request should be valid"),
    )
    .await;

    assert_eq!(authenticated_response.status(), StatusCode::OK);

    let authenticated_body: Value = body_json(authenticated_response).await;

    assert_eq!(authenticated_body["authenticated"], true);
    assert_eq!(authenticated_body["auth_mode"], "neoshowcase");
    assert_eq!(authenticated_body["user"]["display_name"], provider_subject);
    assert_eq!(authenticated_body["login_url"], Value::Null);
    assert_eq!(
        authenticated_body["logout_url"],
        "/_oauth/logout?redirect=/"
    );

    let first_user_id = authenticated_body["user"]["id"]
        .as_str()
        .and_then(|value| Uuid::parse_str(value).ok())
        .expect("authenticated response should contain a user UUID");

    let stored_user_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT user_id
        FROM users
        WHERE auth_provider = 'neoshowcase'
          AND provider_subject = ?
        "#,
    )
    .bind(&provider_subject)
    .fetch_one(&pool)
    .await
    .expect("NeoShowcase user should be stored");

    assert_eq!(stored_user_id, first_user_id);

    let repeated_response = request(
        &app,
        Request::get("/api/me")
            .header("x-forwarded-user", &provider_subject)
            .body(Body::empty())
            .expect("repeated me request should be valid"),
    )
    .await;

    assert_eq!(repeated_response.status(), StatusCode::OK);

    let repeated_body: Value = body_json(repeated_response).await;
    assert_eq!(repeated_body["user"]["id"], first_user_id.to_string());

    let matching_user_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM users
        WHERE auth_provider = 'neoshowcase'
          AND provider_subject = ?
        "#,
    )
    .bind(&provider_subject)
    .fetch_one(&pool)
    .await
    .expect("matching NeoShowcase user count should be readable");

    assert_eq!(matching_user_count, 1);

    cleanup_users_with_subject_prefix(&pool, NEO_AUTH_TEST_SUBJECT_PREFIX).await;

    pool.close().await;
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL pointing to a disposable MariaDB database"]
async fn mariadb_demo_mode_ignores_forwarded_user_header() {
    let pool = connect_test_database().await;
    migrate(&pool).await.expect("migration should succeed");
    cleanup_users_with_subject_prefix(&pool, DEMO_HEADER_TEST_SUBJECT_PREFIX).await;

    let suffix = Uuid::new_v4().simple().to_string();
    let forged_subject = format!("{DEMO_HEADER_TEST_SUBJECT_PREFIX}{}", &suffix[..8]);

    let repository = Arc::new(SqlxUserRepository::new(pool.clone()));
    let app = app(AppState::new(AuthMode::Demo, repository));

    let response = request(
        &app,
        Request::get("/api/me")
            .header("x-forwarded-user", &forged_subject)
            .body(Body::empty())
            .expect("demo me request should be valid"),
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);

    let body: Value = body_json(response).await;
    assert_eq!(body["authenticated"], false);
    assert_eq!(body["auth_mode"], "demo");
    assert_eq!(body["user"], Value::Null);

    let matching_user_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM users
        WHERE provider_subject = ?
        "#,
    )
    .bind(&forged_subject)
    .fetch_one(&pool)
    .await
    .expect("matching user count should be readable");

    assert_eq!(
        matching_user_count, 0,
        "forwarded user header must not create a user in demo mode",
    );

    cleanup_users_with_subject_prefix(&pool, DEMO_HEADER_TEST_SUBJECT_PREFIX).await;

    pool.close().await;
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL pointing to a disposable MariaDB database"]
async fn mariadb_run_start_resume_and_current_http_flow() {
    let pool = connect_test_database().await;
    migrate(&pool).await.expect("migration should succeed");
    cleanup_game_flow_test_data(&pool).await;
    seed_game_catalog(&pool).await;

    let suffix = Uuid::new_v4().simple().to_string();
    let display_name = format!("{RUN_TEST_SUBJECT_PREFIX}{}", &suffix[..8]);
    let room_id = parse_uuid(GAME_ROOM_ID);

    let repository = Arc::new(SqlxUserRepository::new(pool.clone()));
    let app = app(AppState::new(AuthMode::Demo, repository).with_demo_cookie_secure(false));

    let (user_id, _session_id, cookie) = login_guest(&app, &display_name).await;

    let start_response = request(
        &app,
        Request::post(format!("/api/rooms/{room_id}/runs"))
            .header(header::COOKIE, &cookie)
            .body(Body::empty())
            .expect("start run request should be valid"),
    )
    .await;

    assert_eq!(start_response.status(), StatusCode::OK);
    assert_eq!(
        start_response.headers()[header::CONTENT_TYPE],
        "application/json"
    );

    let start_body: Value = body_json(start_response).await;
    assert_eq!(start_body["status"], "active");
    assert!(start_body["started_at"].as_str().is_some());
    assert!(start_body["elapsed_ms"].as_u64().is_some());
    assert_eq!(start_body["cleared_problem_ids"], json!([]));
    assert!(
        start_body.get("query_count").is_none(),
        "run response must not contain query_count",
    );

    let run_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT run_id
        FROM runs
        WHERE user_id = ?
          AND room_id = ?
          AND status = 'active'
        "#,
    )
    .bind(user_id)
    .bind(room_id)
    .fetch_one(&pool)
    .await
    .expect("active run should be stored");

    let progress_statuses = sqlx::query_scalar::<_, String>(
        r#"
        SELECT problem_progress.status
        FROM problem_progress
        INNER JOIN problems
            ON problems.problem_id = problem_progress.problem_id
        WHERE problem_progress.run_id = ?
        ORDER BY problems.number
        "#,
    )
    .bind(run_id)
    .fetch_all(&pool)
    .await
    .expect("problem progress should be readable");

    assert_eq!(
        progress_statuses,
        vec!["available", "locked", "locked", "available"],
    );

    let resume_response = request(
        &app,
        Request::post(format!("/api/rooms/{room_id}/runs"))
            .header(header::COOKIE, &cookie)
            .body(Body::empty())
            .expect("resume run request should be valid"),
    )
    .await;

    assert_eq!(resume_response.status(), StatusCode::OK);

    let resume_body: Value = body_json(resume_response).await;
    assert_eq!(resume_body["status"], "active");
    assert_eq!(resume_body["started_at"], start_body["started_at"]);
    assert_eq!(resume_body["cleared_problem_ids"], json!([]));
    assert!(resume_body.get("query_count").is_none());

    let active_run_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM runs
        WHERE user_id = ?
          AND room_id = ?
          AND status = 'active'
        "#,
    )
    .bind(user_id)
    .bind(room_id)
    .fetch_one(&pool)
    .await
    .expect("active run count should be readable");

    assert_eq!(
        active_run_count, 1,
        "resuming must not create another active run",
    );

    let current_response = request(
        &app,
        Request::get(format!("/api/rooms/{room_id}/runs/current"))
            .header(header::COOKIE, &cookie)
            .body(Body::empty())
            .expect("current run request should be valid"),
    )
    .await;

    assert_eq!(current_response.status(), StatusCode::OK);

    let current_body: Value = body_json(current_response).await;
    assert_eq!(current_body["status"], "active");
    assert_eq!(current_body["started_at"], start_body["started_at"]);
    assert_eq!(current_body["cleared_problem_ids"], json!([]));
    assert!(current_body["elapsed_ms"].as_u64().is_some());
    assert!(
        current_body.get("query_count").is_none(),
        "current run response must not contain query_count",
    );

    logout_guest(&app, &cookie).await;
    cleanup_game_flow_test_data(&pool).await;
    pool.close().await;
}

async fn login_guest(app: &Router, display_name: &str) -> (Uuid, Uuid, String) {
    let response = request(
        app,
        Request::post("/api/auth/guest")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                serde_json::to_vec(&json!({
                    "display_name": display_name
                }))
                .expect("guest login payload should be serializable"),
            ))
            .expect("guest login request should be valid"),
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");

    let cookie = response
        .headers()
        .get(header::SET_COOKIE)
        .expect("Set-Cookie header should be present")
        .to_str()
        .expect("Set-Cookie header should be valid")
        .split(';')
        .next()
        .expect("Set-Cookie header should contain a cookie pair")
        .to_owned();

    let session_id = cookie
        .strip_prefix("demo_session=")
        .and_then(|value| Uuid::parse_str(value).ok())
        .expect("demo_session cookie should contain a UUID");

    let body: Value = body_json(response).await;

    assert_eq!(body["authenticated"], true);
    assert_eq!(body["user"]["display_name"], display_name);

    let user_id = body["user"]["id"]
        .as_str()
        .and_then(|value| Uuid::parse_str(value).ok())
        .expect("guest login response should contain a user UUID");

    (user_id, session_id, cookie)
}

async fn logout_guest(app: &Router, cookie: &str) {
    let response = request(
        app,
        Request::post("/api/auth/logout")
            .header(header::COOKIE, cookie)
            .body(Body::empty())
            .expect("logout request should be valid"),
    )
    .await;

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert!(body_bytes(response).await.is_empty());
}

async fn cleanup_users_with_subject_prefix(pool: &MySqlPool, subject_prefix: &str) {
    let pattern = format!("{subject_prefix}%");

    sqlx::query(
        r#"
        DELETE FROM users
        WHERE provider_subject LIKE ?
        "#,
    )
    .bind(pattern)
    .execute(pool)
    .await
    .expect("authentication test data cleanup should succeed");
}

async fn seed_game_catalog(pool: &MySqlPool) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../mock-problem-data");
    let catalog = load_problem_data(root).expect("mock problem data should be valid");

    seed_problem_data(pool, &catalog)
        .await
        .expect("mock problem data should be seeded");
}

async fn cleanup_game_flow_test_data(pool: &MySqlPool) {
    let room_id = parse_uuid(GAME_ROOM_ID);

    sqlx::query("DELETE FROM runs WHERE room_id = ?")
        .bind(room_id)
        .execute(pool)
        .await
        .expect("game test runs should be removable");

    cleanup_users_with_subject_prefix(pool, RUN_TEST_SUBJECT_PREFIX).await;

    for problem_id in GAME_PROBLEM_IDS.iter().rev() {
        sqlx::query("DELETE FROM problems WHERE problem_id = ?")
            .bind(parse_uuid(problem_id))
            .execute(pool)
            .await
            .expect("game test problem should be removable");
    }

    sqlx::query("DELETE FROM rooms WHERE room_id = ?")
        .bind(room_id)
        .execute(pool)
        .await
        .expect("game test room should be removable");
}

fn parse_uuid(value: &str) -> Uuid {
    Uuid::parse_str(value).expect("test UUID should be valid")
}
