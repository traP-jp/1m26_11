mod common;

use std::{path::PathBuf, sync::Arc};

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
};
use chrono::{DateTime, Utc};
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

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL pointing to a disposable MariaDB database"]
async fn mariadb_problem_query_and_answer_http_flow() {
    let pool = connect_test_database().await;
    migrate(&pool).await.expect("migration should succeed");
    cleanup_game_flow_test_data(&pool).await;
    seed_game_catalog(&pool).await;

    let suffix = Uuid::new_v4().simple().to_string();
    let display_name = format!("{RUN_TEST_SUBJECT_PREFIX}{}", &suffix[..8]);
    let room_id = parse_uuid(GAME_ROOM_ID);
    let first_problem_id = parse_uuid(GAME_PROBLEM_IDS[0]);
    let second_problem_id = parse_uuid(GAME_PROBLEM_IDS[1]);
    let third_problem_id = parse_uuid(GAME_PROBLEM_IDS[2]);

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
    let _start_body: Value = body_json(start_response).await;

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

    // 2問目は1問目へ依存しているため、開始直後はlockedです。
    let locked_response = request(
        &app,
        Request::get(format!("/api/rooms/{room_id}/problems/{second_problem_id}"))
            .header(header::COOKIE, &cookie)
            .body(Body::empty())
            .expect("locked problem request should be valid"),
    )
    .await;

    assert_eq!(locked_response.status(), StatusCode::CONFLICT);

    let locked_body: Value = body_json(locked_response).await;
    assert_eq!(locked_body["error"]["code"], "PROBLEM_LOCKED");

    // 1問目へ不正解のqueryを送ります。
    let incorrect_query_response = request(
        &app,
        Request::post(format!(
            "/api/rooms/{room_id}/problems/{first_problem_id}/queries"
        ))
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::COOKIE, &cookie)
        .body(Body::from(
            serde_json::to_vec(&json!({
                "source": "serial",
                "operations": [
                    {
                        "control": "down",
                        "count": 1
                    }
                ]
            }))
            .expect("incorrect query payload should be serializable"),
        ))
        .expect("incorrect query request should be valid"),
    )
    .await;

    assert_eq!(incorrect_query_response.status(), StatusCode::OK);

    let incorrect_query_body: Value = body_json(incorrect_query_response).await;
    assert_eq!(incorrect_query_body["correct"], false);
    assert_eq!(incorrect_query_body["query_count"], 1);
    assert_eq!(incorrect_query_body["remaining_pattern_count"], 2);
    assert_eq!(incorrect_query_body["problem_status"], "available");
    assert_eq!(stored_query_count(&pool, run_id, first_problem_id).await, 1);
    assert_eq!(
        stored_answer_attempt_count(&pool, run_id, first_problem_id).await,
        1
    );

    let incorrect_query_id = incorrect_query_body["query_id"]
        .as_str()
        .and_then(|value| Uuid::parse_str(value).ok())
        .expect("incorrect query response should contain a query UUID");

    // 続いて1問目の正解queryを送ります。
    let correct_query_response = request(
        &app,
        Request::post(format!(
            "/api/rooms/{room_id}/problems/{first_problem_id}/queries"
        ))
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::COOKIE, &cookie)
        .body(Body::from(
            serde_json::to_vec(&json!({
                "source": "serial",
                "operations": [
                    {
                        "control": "down",
                        "count": 1
                    },
                    {
                        "control": "right",
                        "count": 2
                    }
                ]
            }))
            .expect("correct query payload should be serializable"),
        ))
        .expect("correct query request should be valid"),
    )
    .await;

    assert_eq!(correct_query_response.status(), StatusCode::OK);

    let correct_query_body: Value = body_json(correct_query_response).await;
    assert_eq!(correct_query_body["correct"], true);
    assert_eq!(correct_query_body["query_count"], 2);
    assert_eq!(correct_query_body["remaining_pattern_count"], 1);
    assert_eq!(correct_query_body["problem_status"], "cleared");
    assert_eq!(stored_query_count(&pool, run_id, first_problem_id).await, 2);
    assert_eq!(
        stored_answer_attempt_count(&pool, run_id, first_problem_id).await,
        2
    );

    let correct_query_id = correct_query_body["query_id"]
        .as_str()
        .and_then(|value| Uuid::parse_str(value).ok())
        .expect("correct query response should contain a query UUID");

    assert_ne!(incorrect_query_id, correct_query_id);

    // queryが2件ともMariaDBへ保存されたことを確認します。
    let query_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM queries
        WHERE run_id = ?
          AND problem_id = ?
        "#,
    )
    .bind(run_id)
    .bind(first_problem_id)
    .fetch_one(&pool)
    .await
    .expect("query history count should be readable");

    assert_eq!(query_count, 2);

    let incorrect_query_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM queries
        WHERE run_id = ?
          AND problem_id = ?
          AND is_correct = 0
        "#,
    )
    .bind(run_id)
    .bind(first_problem_id)
    .fetch_one(&pool)
    .await
    .expect("incorrect query count should be readable");

    let correct_query_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM queries
        WHERE run_id = ?
          AND problem_id = ?
          AND is_correct = 1
        "#,
    )
    .bind(run_id)
    .bind(first_problem_id)
    .fetch_one(&pool)
    .await
    .expect("correct query count should be readable");

    assert_eq!(incorrect_query_count, 1);
    assert_eq!(correct_query_count, 1);

    // 正解queryによって1問目がcleared、2問目がavailableになります。
    let first_status = sqlx::query_scalar::<_, String>(
        r#"
        SELECT status
        FROM problem_progress
        WHERE run_id = ?
          AND problem_id = ?
        "#,
    )
    .bind(run_id)
    .bind(first_problem_id)
    .fetch_one(&pool)
    .await
    .expect("first problem status should be readable");

    let second_status = sqlx::query_scalar::<_, String>(
        r#"
        SELECT status
        FROM problem_progress
        WHERE run_id = ?
          AND problem_id = ?
        "#,
    )
    .bind(run_id)
    .bind(second_problem_id)
    .fetch_one(&pool)
    .await
    .expect("second problem status should be readable");

    assert_eq!(first_status, "cleared");
    assert_eq!(second_status, "available");

    // unlockされた2問目をAPIから取得します。
    let problem_response = request(
        &app,
        Request::get(format!("/api/rooms/{room_id}/problems/{second_problem_id}"))
            .header(header::COOKIE, &cookie)
            .body(Body::empty())
            .expect("available problem request should be valid"),
    )
    .await;

    assert_eq!(problem_response.status(), StatusCode::OK);

    let problem_body: Value = body_json(problem_response).await;
    assert_eq!(problem_body["id"], second_problem_id.to_string());
    assert_eq!(problem_body["number"], 2);
    assert_eq!(problem_body["type"], "small");
    assert_eq!(problem_body["title"], "合言葉");
    assert_eq!(problem_body["submission_type"], "string");
    assert_eq!(problem_body["status"], "available");
    assert_eq!(problem_body["hint_count"], 0);
    assert!(
        problem_body.get("judge_config").is_none(),
        "problem response must not expose judge_config",
    );
    assert!(
        problem_body.get("hints").is_none(),
        "problem response must not expose hint bodies",
    );

    // 2問目へ不正解answerを送り、試行回数を増やします。
    let incorrect_answer_response = request(
        &app,
        Request::post(format!(
            "/api/rooms/{room_id}/problems/{second_problem_id}/answers"
        ))
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::COOKIE, &cookie)
        .body(Body::from(
            serde_json::to_vec(&json!({
                "answer": "違います"
            }))
            .expect("incorrect answer payload should be serializable"),
        ))
        .expect("incorrect answer request should be valid"),
    )
    .await;

    assert_eq!(incorrect_answer_response.status(), StatusCode::OK);

    let incorrect_answer_body: Value = body_json(incorrect_answer_response).await;
    assert_eq!(incorrect_answer_body["correct"], false);
    assert_eq!(incorrect_answer_body["answer_attempt_count"], 1);
    assert_eq!(incorrect_answer_body["problem_status"], "available");
    assert_eq!(incorrect_answer_body["run_status"], "active");

    let answer_attempt_count = sqlx::query_scalar::<_, i32>(
        r#"
        SELECT answer_attempt_count
        FROM problem_progress
        WHERE run_id = ?
          AND problem_id = ?
        "#,
    )
    .bind(run_id)
    .bind(second_problem_id)
    .fetch_one(&pool)
    .await
    .expect("answer attempt count should be readable");

    assert_eq!(answer_attempt_count, 1);

    // 正解answerを送り、3問目がunlockされることを確認します。
    let correct_answer_response = request(
        &app,
        Request::post(format!(
            "/api/rooms/{room_id}/problems/{second_problem_id}/answers"
        ))
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::COOKIE, &cookie)
        .body(Body::from(
            serde_json::to_vec(&json!({
                "answer": " 顔文字くん "
            }))
            .expect("correct answer payload should be serializable"),
        ))
        .expect("correct answer request should be valid"),
    )
    .await;

    assert_eq!(correct_answer_response.status(), StatusCode::OK);

    let correct_answer_body: Value = body_json(correct_answer_response).await;
    assert_eq!(correct_answer_body["correct"], true);
    assert_eq!(correct_answer_body["problem_status"], "cleared");
    assert_eq!(
        correct_answer_body["unlocked_problem_ids"],
        json!([third_problem_id])
    );
    assert_eq!(correct_answer_body["run_status"], "active");
    assert_eq!(
        correct_answer_body["progress"],
        json!({
            "cleared_problem_count": 2,
            "total_problem_count": 4
        })
    );
    assert!(correct_answer_body["elapsed_ms"].as_u64().is_some());

    let final_statuses = sqlx::query_scalar::<_, String>(
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
    .expect("final problem statuses should be readable");

    assert_eq!(
        final_statuses,
        vec!["cleared", "cleared", "available", "available"],
    );

    logout_guest(&app, &cookie).await;
    cleanup_game_flow_test_data(&pool).await;
    pool.close().await;
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL pointing to a disposable MariaDB database"]
async fn mariadb_final_problem_first_then_room_clear_http_flow() {
    let (pool, app, room_id, run_id, cookie) = setup_demo_game().await;

    let first_problem_id = parse_uuid(GAME_PROBLEM_IDS[0]);
    let second_problem_id = parse_uuid(GAME_PROBLEM_IDS[1]);
    let third_problem_id = parse_uuid(GAME_PROBLEM_IDS[2]);
    let final_problem_id = parse_uuid(GAME_PROBLEM_IDS[3]);

    // 大なぞは最初からavailableなので、先に正解させます。
    let final_answer_path = format!("/api/rooms/{room_id}/problems/{final_problem_id}/answers");

    let final_answer_response = post_authenticated_json(
        &app,
        &final_answer_path,
        &cookie,
        json!({
            "answer": " ワンマンソン "
        }),
    )
    .await;

    assert_eq!(final_answer_response.status(), StatusCode::OK);

    let final_answer_body: Value = body_json(final_answer_response).await;
    assert_eq!(final_answer_body["correct"], true);
    assert_eq!(final_answer_body["problem_status"], "cleared");
    assert_eq!(final_answer_body["unlocked_problem_ids"], json!([]));
    assert_eq!(final_answer_body["run_status"], "active");
    assert_eq!(
        final_answer_body["progress"],
        json!({
            "cleared_problem_count": 1,
            "total_problem_count": 4
        })
    );

    // 1問目をqueryで正解します。
    let first_query_path = format!("/api/rooms/{room_id}/problems/{first_problem_id}/queries");

    let first_query_response = post_authenticated_json(
        &app,
        &first_query_path,
        &cookie,
        json!({
            "source": "serial",
            "operations": [
                {
                    "control": "down",
                    "count": 1
                },
                {
                    "control": "right",
                    "count": 2
                }
            ]
        }),
    )
    .await;

    assert_eq!(first_query_response.status(), StatusCode::OK);

    let first_query_body: Value = body_json(first_query_response).await;
    assert_eq!(first_query_body["correct"], true);
    assert_eq!(first_query_body["query_count"], 1);
    assert_eq!(first_query_body["problem_status"], "cleared");

    // unlockされた2問目をanswerで正解します。
    let second_answer_path = format!("/api/rooms/{room_id}/problems/{second_problem_id}/answers");

    let second_answer_response = post_authenticated_json(
        &app,
        &second_answer_path,
        &cookie,
        json!({
            "answer": "かおもじくん"
        }),
    )
    .await;

    assert_eq!(second_answer_response.status(), StatusCode::OK);

    let second_answer_body: Value = body_json(second_answer_response).await;
    assert_eq!(second_answer_body["correct"], true);
    assert_eq!(second_answer_body["problem_status"], "cleared");
    assert_eq!(
        second_answer_body["unlocked_problem_ids"],
        json!([third_problem_id])
    );
    assert_eq!(second_answer_body["run_status"], "active");
    assert_eq!(
        second_answer_body["progress"],
        json!({
            "cleared_problem_count": 3,
            "total_problem_count": 4
        })
    );

    // 最後に3問目をqueryで正解します。
    let third_query_path = format!("/api/rooms/{room_id}/problems/{third_problem_id}/queries");

    let third_query_response = post_authenticated_json(
        &app,
        &third_query_path,
        &cookie,
        json!({
            "source": "serial",
            "operations": [
                {
                    "control": "left",
                    "count": 1
                },
                {
                    "control": "up",
                    "count": 1
                }
            ]
        }),
    )
    .await;

    assert_eq!(third_query_response.status(), StatusCode::OK);

    let third_query_body: Value = body_json(third_query_response).await;
    assert_eq!(third_query_body["correct"], true);
    assert_eq!(third_query_body["query_count"], 1);
    assert_eq!(third_query_body["problem_status"], "cleared");

    let statuses = sqlx::query_scalar::<_, String>(
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
    .expect("cleared problem statuses should be readable");

    assert_eq!(statuses, vec!["cleared", "cleared", "cleared", "cleared"],);

    // query型の1問目と3問目だけがqueriesへ保存されます。
    let first_query_count = stored_query_count(&pool, run_id, first_problem_id).await;
    let second_query_count = stored_query_count(&pool, run_id, second_problem_id).await;
    let third_query_count = stored_query_count(&pool, run_id, third_problem_id).await;
    let final_query_count = stored_query_count(&pool, run_id, final_problem_id).await;

    assert_eq!(first_query_count, 1);
    assert_eq!(second_query_count, 0);
    assert_eq!(third_query_count, 1);
    assert_eq!(final_query_count, 0);

    // answer型問題を正解しただけでは誤答counterは増えません。
    let second_answer_attempt_count =
        stored_answer_attempt_count(&pool, run_id, second_problem_id).await;
    let final_answer_attempt_count =
        stored_answer_attempt_count(&pool, run_id, final_problem_id).await;

    assert_eq!(second_answer_attempt_count, 0);
    assert_eq!(final_answer_attempt_count, 0);

    let (run_status, cleared_at) = sqlx::query_as::<_, (String, Option<DateTime<Utc>>)>(
        r#"
            SELECT status, cleared_at
            FROM runs
            WHERE run_id = ?
            "#,
    )
    .bind(run_id)
    .fetch_one(&pool)
    .await
    .expect("cleared run should be readable");

    assert_eq!(run_status, "cleared");
    assert!(cleared_at.is_some());

    // clear後はactive runがないためcurrentはRUN_NOT_FOUNDになります。
    let current_response = request(
        &app,
        Request::get(format!("/api/rooms/{room_id}/runs/current"))
            .header(header::COOKIE, &cookie)
            .body(Body::empty())
            .expect("cleared current run request should be valid"),
    )
    .await;

    assert_eq!(current_response.status(), StatusCode::NOT_FOUND);

    let current_body: Value = body_json(current_response).await;
    assert_eq!(current_body["error"]["code"], "RUN_NOT_FOUND");

    // clear済みroomでは新しいrunを開始できません。
    let restart_response = request(
        &app,
        Request::post(format!("/api/rooms/{room_id}/runs"))
            .header(header::COOKIE, &cookie)
            .body(Body::empty())
            .expect("restart request should be valid"),
    )
    .await;

    assert_eq!(restart_response.status(), StatusCode::CONFLICT);

    let restart_body: Value = body_json(restart_response).await;
    assert_eq!(restart_body["error"]["code"], "CONFLICT");
    assert_eq!(restart_body["error"]["message"], "room already cleared");

    logout_guest(&app, &cookie).await;
    cleanup_game_flow_test_data(&pool).await;
    pool.close().await;
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL pointing to a disposable MariaDB database"]
async fn mariadb_error_responses_and_query_rollback_http_flow() {
    let (pool, app, room_id, run_id, cookie) = setup_demo_game().await;

    let first_problem_id = parse_uuid(GAME_PROBLEM_IDS[0]);
    let second_problem_id = parse_uuid(GAME_PROBLEM_IDS[1]);
    let missing_problem_id = Uuid::new_v4();

    let first_query_path = format!("/api/rooms/{room_id}/problems/{first_problem_id}/queries");

    // 400: JSONの構文が壊れているrequestです。
    let malformed_response = request(
        &app,
        Request::post(&first_query_path)
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::COOKIE, &cookie)
            .body(Body::from("{"))
            .expect("malformed JSON request should be valid HTTP"),
    )
    .await;

    assert_eq!(malformed_response.status(), StatusCode::BAD_REQUEST);

    let malformed_body: Value = body_json(malformed_response).await;
    assert_eq!(malformed_body["error"]["code"], "BAD_REQUEST");
    assert_eq!(stored_query_count(&pool, run_id, first_problem_id).await, 0);
    assert_eq!(
        stored_answer_attempt_count(&pool, run_id, first_problem_id).await,
        0
    );

    // 401: Cookieなしでqueryを送信します。
    let unauthorized_response = request(
        &app,
        Request::post(&first_query_path)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                serde_json::to_vec(&json!({
                    "source": "serial",
                    "operations": [
                        {
                            "control": "down",
                            "count": 1
                        }
                    ]
                }))
                .expect("unauthorized payload should be serializable"),
            ))
            .expect("unauthorized request should be valid"),
    )
    .await;

    assert_eq!(unauthorized_response.status(), StatusCode::UNAUTHORIZED);

    let unauthorized_body: Value = body_json(unauthorized_response).await;
    assert_eq!(unauthorized_body["error"]["code"], "UNAUTHORIZED");
    assert_eq!(stored_query_count(&pool, run_id, first_problem_id).await, 0);
    assert_eq!(
        stored_answer_attempt_count(&pool, run_id, first_problem_id).await,
        0
    );

    // 404: 存在しないproblemを取得します。
    let missing_response = request(
        &app,
        Request::get(format!(
            "/api/rooms/{room_id}/problems/{missing_problem_id}"
        ))
        .header(header::COOKIE, &cookie)
        .body(Body::empty())
        .expect("missing problem request should be valid"),
    )
    .await;

    assert_eq!(missing_response.status(), StatusCode::NOT_FOUND);

    let missing_body: Value = body_json(missing_response).await;
    assert_eq!(missing_body["error"]["code"], "NOT_FOUND");

    // 409: まだlockedである2問目へqueryを送信します。
    let locked_query_path = format!("/api/rooms/{room_id}/problems/{second_problem_id}/queries");

    let locked_response = post_authenticated_json(
        &app,
        &locked_query_path,
        &cookie,
        json!({
            "source": "serial",
            "operations": [
                {
                    "control": "down",
                    "count": 1
                }
            ]
        }),
    )
    .await;

    assert_eq!(locked_response.status(), StatusCode::CONFLICT);

    let locked_body: Value = body_json(locked_response).await;
    assert_eq!(locked_body["error"]["code"], "PROBLEM_LOCKED");
    assert_eq!(
        stored_query_count(&pool, run_id, second_problem_id).await,
        0
    );
    assert_eq!(
        stored_answer_attempt_count(&pool, run_id, second_problem_id).await,
        0
    );

    // 422: 許可されていないsourceを送信します。
    let validation_response = post_authenticated_json(
        &app,
        &first_query_path,
        &cookie,
        json!({
            "source": "invalid-source",
            "operations": [
                {
                    "control": "down",
                    "count": 1
                }
            ]
        }),
    )
    .await;

    assert_eq!(
        validation_response.status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );

    let validation_body: Value = body_json(validation_response).await;
    assert_eq!(validation_body["error"]["code"], "VALIDATION_ERROR");
    assert_eq!(stored_query_count(&pool, run_id, first_problem_id).await, 0);
    assert_eq!(
        stored_answer_attempt_count(&pool, run_id, first_problem_id).await,
        0
    );

    // problem_progressのUPDATEだけを失敗させるtriggerを一時的に作ります。
    // correct queryでは、queriesへのINSERT後にproblem_progressをUPDATEするため、
    // この失敗でtransaction全体がrollbackされる必要があります。
    sqlx::query(
        r#"
        CREATE TRIGGER issue79_fail_problem_progress_update
        BEFORE UPDATE ON problem_progress
        FOR EACH ROW
        SIGNAL SQLSTATE '45000'
        SET MESSAGE_TEXT = 'issue79 injected failure'
        "#,
    )
    .execute(&pool)
    .await
    .expect("failure injection trigger should be created");

    let database_error_response = post_authenticated_json(
        &app,
        &first_query_path,
        &cookie,
        json!({
            "source": "serial",
            "operations": [
                {
                    "control": "down",
                    "count": 1
                },
                {
                    "control": "right",
                    "count": 2
                }
            ]
        }),
    )
    .await;

    // assertionに失敗しても後続testへtriggerを残しにくくするため、
    // responseの確認より先にtriggerを削除します。
    sqlx::query("DROP TRIGGER IF EXISTS issue79_fail_problem_progress_update")
        .execute(&pool)
        .await
        .expect("failure injection trigger should be removed");

    assert_eq!(
        database_error_response.status(),
        StatusCode::INTERNAL_SERVER_ERROR
    );
    assert_eq!(
        database_error_response.headers()[header::CONTENT_TYPE],
        "application/json"
    );

    let database_error_bytes = body_bytes(database_error_response).await;
    let database_error_text =
        std::str::from_utf8(&database_error_bytes).expect("error response should be UTF-8");

    assert!(
        !database_error_text.contains("issue79 injected failure"),
        "internal database error details must not be exposed",
    );

    let database_error_body: Value = serde_json::from_slice(&database_error_bytes)
        .expect("database error response should be JSON");

    assert_eq!(
        database_error_body["error"]["code"],
        "INTERNAL_SERVER_ERROR"
    );
    assert_eq!(database_error_body["error"]["details"], json!({}));

    // queriesへのINSERTもrollbackされ、履歴が残っていないことを確認します。
    assert_eq!(
        stored_query_count(&pool, run_id, first_problem_id).await,
        0,
        "failed transaction must not leave query history",
    );
    assert_eq!(
        stored_answer_attempt_count(&pool, run_id, first_problem_id).await,
        0,
        "failed transaction must not increment the shared attempt counter",
    );

    let first_status = sqlx::query_scalar::<_, String>(
        r#"
        SELECT status
        FROM problem_progress
        WHERE run_id = ?
          AND problem_id = ?
        "#,
    )
    .bind(run_id)
    .bind(first_problem_id)
    .fetch_one(&pool)
    .await
    .expect("first problem status should be readable after rollback");

    let second_status = sqlx::query_scalar::<_, String>(
        r#"
        SELECT status
        FROM problem_progress
        WHERE run_id = ?
          AND problem_id = ?
        "#,
    )
    .bind(run_id)
    .bind(second_problem_id)
    .fetch_one(&pool)
    .await
    .expect("second problem status should be readable after rollback");

    assert_eq!(first_status, "available");
    assert_eq!(second_status, "locked");

    // trigger削除後は同じrequestが成功することも確認します。
    let retry_response = post_authenticated_json(
        &app,
        &first_query_path,
        &cookie,
        json!({
            "source": "serial",
            "operations": [
                {
                    "control": "down",
                    "count": 1
                },
                {
                    "control": "right",
                    "count": 2
                }
            ]
        }),
    )
    .await;

    assert_eq!(retry_response.status(), StatusCode::OK);

    let retry_body: Value = body_json(retry_response).await;
    assert_eq!(retry_body["correct"], true);
    assert_eq!(retry_body["query_count"], 1);
    assert_eq!(retry_body["problem_status"], "cleared");

    assert_eq!(
        stored_query_count(&pool, run_id, first_problem_id).await,
        1,
        "successful retry should store exactly one query",
    );
    assert_eq!(
        stored_answer_attempt_count(&pool, run_id, first_problem_id).await,
        1,
        "successful retry should increment the shared attempt counter",
    );

    logout_guest(&app, &cookie).await;
    cleanup_game_flow_test_data(&pool).await;
    pool.close().await;
}

async fn setup_demo_game() -> (MySqlPool, Router, Uuid, Uuid, String) {
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
    let _start_body: Value = body_json(start_response).await;

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

    (pool, app, room_id, run_id, cookie)
}

async fn post_authenticated_json(
    app: &Router,
    path: &str,
    cookie: &str,
    payload: Value,
) -> axum::response::Response {
    request(
        app,
        Request::post(path)
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::COOKIE, cookie)
            .body(Body::from(
                serde_json::to_vec(&payload).expect("request payload should be serializable"),
            ))
            .expect("authenticated JSON request should be valid"),
    )
    .await
}

async fn stored_query_count(pool: &MySqlPool, run_id: Uuid, problem_id: Uuid) -> i64 {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM queries
        WHERE run_id = ?
          AND problem_id = ?
        "#,
    )
    .bind(run_id)
    .bind(problem_id)
    .fetch_one(pool)
    .await
    .expect("stored query count should be readable")
}

async fn stored_answer_attempt_count(pool: &MySqlPool, run_id: Uuid, problem_id: Uuid) -> i32 {
    sqlx::query_scalar::<_, i32>(
        r#"
        SELECT answer_attempt_count
        FROM problem_progress
        WHERE run_id = ?
          AND problem_id = ?
        "#,
    )
    .bind(run_id)
    .bind(problem_id)
    .fetch_one(pool)
    .await
    .expect("stored answer attempt count should be readable")
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
    sqlx::query("DROP TRIGGER IF EXISTS issue79_fail_problem_progress_update")
        .execute(pool)
        .await
        .expect("failure injection trigger cleanup should succeed");

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
