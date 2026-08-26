use std::{
    env,
    str::FromStr,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
};
use chrono::{DateTime, Utc};
use http_body_util::BodyExt;
use openapi_generated::models::ErrorResponse;
use serde_json::json;
use server::{
    AppState, app,
    config::AuthMode,
    migrate,
    problem::{Asset, AssetUrlResolveError, AssetUrlResolver, InputSchema},
    repository::{
        AuthRepository, AuthUserRecord, ProblemDetailRecord, RepositoryError, RoomRecord,
        RunRecord, SqlxUserRepository,
    },
};
use sqlx::{
    MySqlPool,
    mysql::{MySqlConnectOptions, MySqlPoolOptions},
    types::Json,
};
use tower::ServiceExt;
use uuid::Uuid;

const MOCK_SESSION_ID: &str = "55555555-5555-4555-8555-555555555555";
const MOCK_RESUME_ROOM_ID: &str = "11111111-1111-4111-8111-111111111111";
const MOCK_NEW_ROOM_ID: &str = "33333333-3333-4333-8333-333333333333";
const MOCK_CLEARED_ROOM_ID: &str = "44444444-4444-4444-8444-444444444444";
const MOCK_CLEARED_PROBLEM_ID: &str = "22222222-2222-4222-8222-222222222221";
const MOCK_LOCKED_PROBLEM_ID: &str = "22222222-2222-4222-8222-222222222222";
const MOCK_CLEARED_DETAIL_PROBLEM_ID: &str = "22222222-2222-4222-8222-222222222223";
const MOCK_DATABASE_ERROR_PROBLEM_ID: &str = "22222222-2222-4222-8222-222222222224";

fn problem_detail_record(id: Uuid, status: &str) -> ProblemDetailRecord {
    ProblemDetailRecord {
        id,
        number: 1,
        problem_type: "small".to_owned(),
        title: "生年月日".to_owned(),
        body_markdown: "問題文です".to_owned(),
        submission_type: "operation_sequence".to_owned(),
        assets: Json(vec![Asset {
            asset_type: "image".to_owned(),
            object_key: "private/problem-assets/birthday.png".to_owned(),
            alt: "問題資料".to_owned(),
        }]),
        input_schema: Json(
            serde_json::from_value::<InputSchema>(json!({
                "query": {
                    "type": "operation_sequence",
                    "allowed_controls": ["down", "right", "up"],
                    "max_operations": 100
                },
                "answer": {
                    "type": "string",
                    "max_length": 50
                }
            }))
            .expect("problem input schema should be valid"),
        ),
        status: status.to_owned(),
        hint_count: 2,
    }
}

struct StubAssetUrlResolver;

impl AssetUrlResolver for StubAssetUrlResolver {
    fn resolve(&self, object_key: &str) -> Result<String, AssetUrlResolveError> {
        assert_eq!(
            object_key, "private/problem-assets/birthday.png",
            "expected object key should be passed to the resolver",
        );

        Ok("/assets/problems/birthday.png".to_owned())
    }
}

struct StubAuthRepository;

#[async_trait]
impl AuthRepository for StubAuthRepository {
    async fn find_user_by_demo_session(
        &self,
        _session_id: Uuid,
    ) -> Result<Option<AuthUserRecord>, RepositoryError> {
        Ok(Some(AuthUserRecord {
            user_id: Uuid::from_str(MOCK_SESSION_ID).unwrap(),
            display_name: "test-user".to_owned(),
        }))
    }

    async fn find_user_by_provider_subject(
        &self,
        _auth_provider: &str,
        _provider_subject: &str,
    ) -> Result<Option<AuthUserRecord>, RepositoryError> {
        Ok(None)
    }

    async fn get_or_create_user(
        &self,
        _auth_provider: &str,
        _provider_subject: &str,
        display_name: &str,
    ) -> Result<AuthUserRecord, RepositoryError> {
        Ok(AuthUserRecord {
            user_id: Uuid::new_v4(),
            display_name: display_name.to_owned(),
        })
    }

    async fn create_demo_session(
        &self,
        _session_id: Uuid,
        _user_id: Uuid,
    ) -> Result<(), RepositoryError> {
        Ok(())
    }

    async fn delete_demo_session(&self, _session_id: Uuid) -> Result<(), RepositoryError> {
        Ok(())
    }

    async fn find_room_by_id(&self, room_id: Uuid) -> Result<Option<RoomRecord>, RepositoryError> {
        if room_id == Uuid::nil() {
            Ok(None)
        } else {
            Ok(Some(RoomRecord {
                id: room_id,
                number: 1,
                name: "Test Room".to_owned(),
                genre: "Test".to_owned(),
                description: "Test description".to_owned(),
                is_published: true,
                created_at: Utc::now(),
            }))
        }
    }

    async fn find_active_run(
        &self,
        _user_id: Uuid,
        _room_id: Uuid,
    ) -> Result<Option<RunRecord>, RepositoryError> {
        let resume_room_id = Uuid::from_str(MOCK_RESUME_ROOM_ID).unwrap();
        if _room_id == resume_room_id {
            Ok(Some(RunRecord {
                id: resume_room_id,
                user_id: _user_id,
                room_id: _room_id,
                status: "active".to_owned(),
                started_at: Utc::now() - chrono::Duration::seconds(65),
                cleared_at: None,
            }))
        } else {
            Ok(None)
        }
    }

    async fn create_run(
        &self,
        id: Uuid,
        user_id: Uuid,
        room_id: Uuid,
        started_at: DateTime<Utc>,
    ) -> Result<RunRecord, RepositoryError> {
        Ok(RunRecord {
            id,
            user_id,
            room_id,
            status: "active".to_owned(),
            started_at,
            cleared_at: None,
        })
    }

    async fn find_cleared_run(
        &self,
        user_id: Uuid,
        room_id: Uuid,
    ) -> Result<Option<RunRecord>, RepositoryError> {
        let cleared_room_id = Uuid::from_str(MOCK_CLEARED_ROOM_ID).unwrap();
        if room_id == cleared_room_id {
            Ok(Some(RunRecord {
                id: Uuid::new_v4(),
                user_id,
                room_id,
                status: "cleared".to_owned(),
                started_at: Utc::now() - chrono::Duration::seconds(100),
                cleared_at: Some(Utc::now()),
            }))
        } else {
            Ok(None)
        }
    }

    async fn find_cleared_problem_ids(&self, run_id: Uuid) -> Result<Vec<Uuid>, RepositoryError> {
        let resume_room_id = Uuid::from_str(MOCK_RESUME_ROOM_ID).unwrap();
        if run_id == resume_room_id {
            Ok(vec![Uuid::from_str(MOCK_CLEARED_PROBLEM_ID).unwrap()])
        } else {
            Ok(vec![])
        }
    }

    async fn find_problem_for_run(
        &self,
        run_id: Uuid,
        room_id: Uuid,
        problem_id: Uuid,
    ) -> Result<Option<ProblemDetailRecord>, RepositoryError> {
        let active_run_id = Uuid::from_str(MOCK_RESUME_ROOM_ID).unwrap();

        if run_id != active_run_id || room_id != active_run_id {
            return Ok(None);
        }

        let available_id = Uuid::from_str(MOCK_CLEARED_PROBLEM_ID).unwrap();
        let locked_id = Uuid::from_str(MOCK_LOCKED_PROBLEM_ID).unwrap();
        let cleared_id = Uuid::from_str(MOCK_CLEARED_DETAIL_PROBLEM_ID).unwrap();
        let database_error_id = Uuid::from_str(MOCK_DATABASE_ERROR_PROBLEM_ID).unwrap();

        if problem_id == database_error_id {
            return Err(RepositoryError::Database(sqlx::Error::Protocol(
                "simulated private database failure".to_owned(),
            )));
        }

        if problem_id == available_id {
            Ok(Some(problem_detail_record(problem_id, "available")))
        } else if problem_id == locked_id {
            Ok(Some(problem_detail_record(problem_id, "locked")))
        } else if problem_id == cleared_id {
            Ok(Some(problem_detail_record(problem_id, "cleared")))
        } else {
            Ok(None)
        }
    }
}

#[derive(Default)]
struct DemoSessionCalls {
    created: Vec<(Uuid, Uuid)>,
    deleted: Vec<Uuid>,
}

struct RecordingAuthRepository {
    user_id: Uuid,
    demo_session_calls: Mutex<DemoSessionCalls>,
}

impl RecordingAuthRepository {
    fn new(user_id: Uuid) -> Self {
        Self {
            user_id,
            demo_session_calls: Mutex::new(DemoSessionCalls::default()),
        }
    }
}

#[async_trait]
impl AuthRepository for RecordingAuthRepository {
    async fn find_user_by_demo_session(
        &self,
        _session_id: Uuid,
    ) -> Result<Option<AuthUserRecord>, RepositoryError> {
        Ok(None)
    }

    async fn find_user_by_provider_subject(
        &self,
        _auth_provider: &str,
        _provider_subject: &str,
    ) -> Result<Option<AuthUserRecord>, RepositoryError> {
        Ok(None)
    }

    async fn get_or_create_user(
        &self,
        _auth_provider: &str,
        _provider_subject: &str,
        display_name: &str,
    ) -> Result<AuthUserRecord, RepositoryError> {
        Ok(AuthUserRecord {
            user_id: self.user_id,
            display_name: display_name.to_owned(),
        })
    }

    async fn create_demo_session(
        &self,
        session_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), RepositoryError> {
        self.demo_session_calls
            .lock()
            .expect("demo session call log should not be poisoned")
            .created
            .push((session_id, user_id));

        Ok(())
    }

    async fn delete_demo_session(&self, session_id: Uuid) -> Result<(), RepositoryError> {
        self.demo_session_calls
            .lock()
            .expect("demo session call log should not be poisoned")
            .deleted
            .push(session_id);

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
#[ignore = "requires TEST_DATABASE_URL pointing to a disposable MariaDB database"]
async fn mariadb_auth_repository_flow() {
    let pool = connect_test_database().await;

    migrate(&pool).await.expect("migration should succeed");

    let repository = SqlxUserRepository::new(pool.clone());

    let demo_user_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let demo_subject = format!("integration-demo-{demo_user_id}");

    sqlx::query(
        r#"
        INSERT INTO users (
            user_id,
            auth_provider,
            provider_subject,
            display_name
        )
        VALUES (?, 'demo', ?, ?)
        "#,
    )
    .bind(demo_user_id)
    .bind(&demo_subject)
    .bind("demo-user")
    .execute(&pool)
    .await
    .expect("demo user insertion should succeed");

    sqlx::query(
        r#"
        INSERT INTO demo_sessions (session_id, user_id)
        VALUES (?, ?)
        "#,
    )
    .bind(session_id)
    .bind(demo_user_id)
    .execute(&pool)
    .await
    .expect("demo session insertion should succeed");

    let demo_user = repository
        .find_user_by_demo_session(session_id)
        .await
        .expect("demo session lookup should succeed")
        .expect("demo session should resolve a user");

    let neo_subject = format!("integration-neoshowcase-{}", Uuid::new_v4());

    let first_neo_user = repository
        .get_or_create_user("neoshowcase", &neo_subject, "neo-user")
        .await
        .expect("first NeoShowcase lookup should succeed");

    let second_neo_user = repository
        .get_or_create_user("neoshowcase", &neo_subject, "neo-user")
        .await
        .expect("second NeoShowcase lookup should succeed");

    sqlx::query("DELETE FROM demo_sessions WHERE session_id = ?")
        .bind(session_id)
        .execute(&pool)
        .await
        .expect("demo session cleanup should succeed");

    sqlx::query("DELETE FROM users WHERE user_id = ?")
        .bind(demo_user_id)
        .execute(&pool)
        .await
        .expect("demo user cleanup should succeed");

    sqlx::query(
        r#"
        DELETE FROM users
        WHERE auth_provider = 'neoshowcase'
          AND provider_subject = ?
        "#,
    )
    .bind(&neo_subject)
    .execute(&pool)
    .await
    .expect("NeoShowcase user cleanup should succeed");

    pool.close().await;

    assert_eq!(
        demo_user,
        AuthUserRecord {
            user_id: demo_user_id,
            display_name: "demo-user".to_owned(),
        }
    );

    assert_eq!(first_neo_user, second_neo_user);
    assert_eq!(first_neo_user.display_name, "neo-user");
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL pointing to an empty disposable MariaDB database"]
async fn mariadb_game_schema_matches_contract() {
    let pool = connect_test_database().await;

    migrate(&pool)
        .await
        .expect("first migration run should succeed");

    migrate(&pool)
        .await
        .expect("second migration run should not reapply migrations");

    let mut tables = sqlx::query_scalar::<_, String>(
        r#"
        SELECT table_name
        FROM information_schema.tables
        WHERE table_schema = DATABASE()
          AND table_type = 'BASE TABLE'
          AND table_name <> '_sqlx_migrations'
        ORDER BY table_name
        "#,
    )
    .fetch_all(&pool)
    .await
    .expect("table names should be readable");

    tables.sort();

    assert_eq!(
        tables,
        vec![
            "demo_sessions".to_owned(),
            "problem_progress".to_owned(),
            "problems".to_owned(),
            "queries".to_owned(),
            "rooms".to_owned(),
            "runs".to_owned(),
            "users".to_owned(),
        ]
    );

    let primary_keys: &[(&str, &[&str])] = &[
        ("users", &["user_id"]),
        ("demo_sessions", &["session_id"]),
        ("rooms", &["room_id"]),
        ("problems", &["problem_id"]),
        ("runs", &["run_id"]),
        ("problem_progress", &["run_id", "problem_id"]),
        ("queries", &["query_id"]),
    ];

    for (table_name, expected_columns) in primary_keys {
        let actual_columns = primary_key_columns(&pool, table_name).await;
        let expected_columns = expected_columns
            .iter()
            .map(|column| (*column).to_owned())
            .collect::<Vec<_>>();

        assert_eq!(
            actual_columns, expected_columns,
            "unexpected primary key for {table_name}"
        );
    }

    let indexes: &[(&str, &str, &[&str])] = &[
        ("demo_sessions", "idx_demo_sessions_user_id", &["user_id"]),
        ("rooms", "uq_rooms_number", &["number"]),
        (
            "problems",
            "uq_problems_room_number",
            &["room_id", "number"],
        ),
        (
            "runs",
            "uq_runs_user_room_active",
            &["user_id", "room_id", "active_marker"],
        ),
        (
            "runs",
            "idx_runs_ranking",
            &["room_id", "status", "user_id", "cleared_at", "started_at"],
        ),
        (
            "problem_progress",
            "idx_problem_progress_run_status",
            &["run_id", "status"],
        ),
        (
            "queries",
            "idx_queries_run_problem_created_at",
            &["run_id", "problem_id", "created_at"],
        ),
    ];

    for (table_name, index_name, expected_columns) in indexes {
        let actual_columns = index_columns(&pool, table_name, index_name).await;
        let expected_columns = expected_columns
            .iter()
            .map(|column| (*column).to_owned())
            .collect::<Vec<_>>();

        assert_eq!(
            actual_columns, expected_columns,
            "unexpected index {index_name} on {table_name}"
        );
    }

    let foreign_keys = [
        ("fk_demo_sessions_user_id", "CASCADE"),
        ("fk_problems_room_id", "RESTRICT"),
        ("fk_problems_depends_on", "RESTRICT"),
        ("fk_runs_user_id", "RESTRICT"),
        ("fk_runs_room_id", "RESTRICT"),
        ("fk_problem_progress_run_id", "CASCADE"),
        ("fk_problem_progress_problem_id", "CASCADE"),
        ("fk_queries_problem_progress", "CASCADE"),
    ];

    for (constraint_name, expected_delete_rule) in foreign_keys {
        let actual_delete_rule = foreign_key_delete_rule(&pool, constraint_name).await;

        assert_eq!(
            actual_delete_rule, expected_delete_rule,
            "unexpected delete rule for {constraint_name}"
        );
    }

    pool.close().await;
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL pointing to a disposable MariaDB database"]
async fn mariadb_game_schema_enforces_constraints() {
    let pool = connect_test_database().await;

    migrate(&pool).await.expect("migration should succeed");

    let user_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let room_id = Uuid::new_v4();
    let problem_id = Uuid::new_v4();
    let run_id = Uuid::new_v4();
    let query_id = Uuid::new_v4();
    let room_number = (user_id.as_u128() % 2_000_000_000) as i32 + 1;
    let provider_subject = format!("schema-test-{user_id}");

    sqlx::query(
        r#"
        INSERT INTO users (
            user_id,
            auth_provider,
            provider_subject,
            display_name
        )
        VALUES (?, 'demo', ?, 'schema-test-user')
        "#,
    )
    .bind(user_id)
    .bind(&provider_subject)
    .execute(&pool)
    .await
    .expect("user insertion should succeed");

    sqlx::query(
        r#"
        INSERT INTO demo_sessions (session_id, user_id)
        VALUES (?, ?)
        "#,
    )
    .bind(session_id)
    .bind(user_id)
    .execute(&pool)
    .await
    .expect("demo session insertion should succeed");

    sqlx::query(
        r#"
        INSERT INTO rooms (
            room_id,
            number,
            name,
            genre,
            description,
            is_published
        )
        VALUES (?, ?, 'schema-test-room', 'test', 'test room', 1)
        "#,
    )
    .bind(room_id)
    .bind(room_number)
    .execute(&pool)
    .await
    .expect("room insertion should succeed");

    sqlx::query(
        r#"
        INSERT INTO problems (
            problem_id,
            room_id,
            number,
            problem_type,
            title,
            body_markdown,
            submission_type,
            assets,
            input_schema,
            hints,
            judge_config,
            depends_on_problem_id,
            is_required
        )
        VALUES (
            ?, ?, 1, 'small', 'test problem', 'test body', 'string',
            JSON_ARRAY(), JSON_OBJECT(), JSON_ARRAY(), JSON_OBJECT(),
            NULL, 1
        )
        "#,
    )
    .bind(problem_id)
    .bind(room_id)
    .execute(&pool)
    .await
    .expect("problem insertion should succeed");

    sqlx::query(
        r#"
        INSERT INTO runs (
            run_id,
            user_id,
            room_id,
            status,
            started_at,
            cleared_at
        )
        VALUES (?, ?, ?, 'active', CURRENT_TIMESTAMP(3), NULL)
        "#,
    )
    .bind(run_id)
    .bind(user_id)
    .bind(room_id)
    .execute(&pool)
    .await
    .expect("run insertion should succeed");

    sqlx::query(
        r#"
        INSERT INTO problem_progress (
            run_id,
            problem_id,
            status,
            answer_attempt_count,
            cleared_at
        )
        VALUES (?, ?, 'available', 0, NULL)
        "#,
    )
    .bind(run_id)
    .bind(problem_id)
    .execute(&pool)
    .await
    .expect("problem progress insertion should succeed");

    sqlx::query(
        r#"
        INSERT INTO queries (
            query_id,
            run_id,
            problem_id,
            source,
            operations,
            normalized_operations,
            remaining_pattern_count,
            is_correct
        )
        VALUES (
            ?, ?, ?, 'keyboard',
            JSON_ARRAY(), JSON_ARRAY(), 1, 0
        )
        "#,
    )
    .bind(query_id)
    .bind(run_id)
    .bind(problem_id)
    .execute(&pool)
    .await
    .expect("query insertion should succeed");

    let invalid_boolean = sqlx::query(
        r#"
        INSERT INTO rooms (
            room_id,
            number,
            name,
            genre,
            description,
            is_published
        )
        VALUES (?, ?, 'invalid room', 'test', 'invalid boolean', 2)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(room_number + 1)
    .execute(&pool)
    .await;

    assert!(
        invalid_boolean.is_err(),
        "is_published CHECK should reject 2"
    );

    let duplicate_room_number = sqlx::query(
        r#"
        INSERT INTO rooms (
            room_id,
            number,
            name,
            genre,
            description,
            is_published
        )
        VALUES (?, ?, 'duplicate room', 'test', 'duplicate number', 1)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(room_number)
    .execute(&pool)
    .await;

    assert!(
        duplicate_room_number.is_err(),
        "room number UNIQUE constraint should reject duplicates"
    );

    let duplicate_active_run = sqlx::query(
        r#"
        INSERT INTO runs (
            run_id,
            user_id,
            room_id,
            status,
            started_at,
            cleared_at
        )
        VALUES (?, ?, ?, 'active', CURRENT_TIMESTAMP(3), NULL)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(room_id)
    .execute(&pool)
    .await;

    assert!(
        duplicate_active_run.is_err(),
        "only one active run should be allowed per user and room"
    );

    let invalid_query_source = sqlx::query(
        r#"
        INSERT INTO queries (
            query_id,
            run_id,
            problem_id,
            source,
            operations,
            normalized_operations,
            remaining_pattern_count,
            is_correct
        )
        VALUES (
            ?, ?, ?, 'invalid',
            JSON_ARRAY(), JSON_ARRAY(), 1, 0
        )
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(run_id)
    .bind(problem_id)
    .execute(&pool)
    .await;

    assert!(
        invalid_query_source.is_err(),
        "query source CHECK should reject unknown sources"
    );

    let restricted_user_delete = sqlx::query(
        r#"
        DELETE FROM users
        WHERE user_id = ?
        "#,
    )
    .bind(user_id)
    .execute(&pool)
    .await;

    assert!(
        restricted_user_delete.is_err(),
        "user deletion should be restricted while a run exists"
    );

    let restricted_room_delete = sqlx::query(
        r#"
        DELETE FROM rooms
        WHERE room_id = ?
        "#,
    )
    .bind(room_id)
    .execute(&pool)
    .await;

    assert!(
        restricted_room_delete.is_err(),
        "room deletion should be restricted while referenced"
    );

    sqlx::query(
        r#"
        DELETE FROM runs
        WHERE run_id = ?
        "#,
    )
    .bind(run_id)
    .execute(&pool)
    .await
    .expect("run deletion should succeed");

    let progress_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM problem_progress
        WHERE run_id = ?
        "#,
    )
    .bind(run_id)
    .fetch_one(&pool)
    .await
    .expect("problem progress count should be readable");

    let query_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM queries
        WHERE run_id = ?
        "#,
    )
    .bind(run_id)
    .fetch_one(&pool)
    .await
    .expect("query count should be readable");

    assert_eq!(progress_count, 0, "run deletion should cascade to progress");
    assert_eq!(
        query_count, 0,
        "progress deletion should cascade to queries"
    );

    sqlx::query("DELETE FROM problems WHERE problem_id = ?")
        .bind(problem_id)
        .execute(&pool)
        .await
        .expect("problem cleanup should succeed");

    sqlx::query("DELETE FROM rooms WHERE room_id = ?")
        .bind(room_id)
        .execute(&pool)
        .await
        .expect("room cleanup should succeed");

    sqlx::query("DELETE FROM users WHERE user_id = ?")
        .bind(user_id)
        .execute(&pool)
        .await
        .expect("user deletion should succeed after run deletion");

    let session_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM demo_sessions
        WHERE session_id = ?
        "#,
    )
    .bind(session_id)
    .fetch_one(&pool)
    .await
    .expect("demo session count should be readable");

    assert_eq!(
        session_count, 0,
        "user deletion should cascade to demo sessions"
    );

    pool.close().await;
}

fn test_app() -> Router {
    app(AppState::new(AuthMode::Demo, Arc::new(StubAuthRepository)))
}

fn problem_test_app() -> Router {
    app(AppState::new(AuthMode::Demo, Arc::new(StubAuthRepository))
        .with_asset_url_resolver(Arc::new(StubAssetUrlResolver)))
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
    let database_url = env::var("TEST_DATABASE_URL")
        .expect("TEST_DATABASE_URL must point to a disposable test database");

    let options =
        MySqlConnectOptions::from_str(&database_url).expect("TEST_DATABASE_URL should be valid");

    MySqlPoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .expect("test database should be reachable")
}

async fn primary_key_columns(pool: &MySqlPool, table_name: &str) -> Vec<String> {
    sqlx::query_scalar(
        r#"
        SELECT column_name
        FROM information_schema.key_column_usage
        WHERE table_schema = DATABASE()
          AND table_name = ?
          AND constraint_name = 'PRIMARY'
        ORDER BY ordinal_position
        "#,
    )
    .bind(table_name)
    .fetch_all(pool)
    .await
    .expect("primary key columns should be readable")
}

async fn index_columns(pool: &MySqlPool, table_name: &str, index_name: &str) -> Vec<String> {
    sqlx::query_scalar(
        r#"
        SELECT column_name
        FROM information_schema.statistics
        WHERE table_schema = DATABASE()
          AND table_name = ?
          AND index_name = ?
        ORDER BY seq_in_index
        "#,
    )
    .bind(table_name)
    .bind(index_name)
    .fetch_all(pool)
    .await
    .expect("index columns should be readable")
}

async fn foreign_key_delete_rule(pool: &MySqlPool, constraint_name: &str) -> String {
    sqlx::query_scalar(
        r#"
        SELECT delete_rule
        FROM information_schema.referential_constraints
        WHERE constraint_schema = DATABASE()
          AND constraint_name = ?
        "#,
    )
    .bind(constraint_name)
    .fetch_one(pool)
    .await
    .expect("foreign key delete rule should be readable")
}

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
#[ignore = "requires TEST_DATABASE_URL pointing to a disposable MariaDB database"]
async fn mariadb_problem_detail_repository_is_scoped_to_run_and_room() {
    let pool = connect_test_database().await;

    migrate(&pool).await.expect("migration should succeed");

    let user_id = Uuid::new_v4();
    let room_id = Uuid::new_v4();
    let problem_id = Uuid::new_v4();
    let run_id = Uuid::new_v4();

    let room_number = (room_id.as_u128() % 2_000_000_000) as i32 + 1;
    let provider_subject = format!("problem-detail-test-{user_id}");

    sqlx::query(
        r#"
        INSERT INTO users (
            user_id,
            auth_provider,
            provider_subject,
            display_name
        )
        VALUES (?, 'demo', ?, 'problem-detail-test-user')
        "#,
    )
    .bind(user_id)
    .bind(&provider_subject)
    .execute(&pool)
    .await
    .expect("test user should be inserted");

    sqlx::query(
        r#"
        INSERT INTO rooms (
            room_id,
            number,
            name,
            genre,
            description,
            is_published
        )
        VALUES (
            ?, ?, 'problem-detail-test-room',
            'test', 'problem detail repository test', 1
        )
        "#,
    )
    .bind(room_id)
    .bind(room_number)
    .execute(&pool)
    .await
    .expect("test room should be inserted");

    sqlx::query(
        r#"
        INSERT INTO problems (
            problem_id,
            room_id,
            number,
            problem_type,
            title,
            body_markdown,
            submission_type,
            assets,
            input_schema,
            hints,
            judge_config,
            depends_on_problem_id,
            is_required
        )
        VALUES (
            ?, ?, 1, 'small', 'MariaDB test problem',
            'MariaDBから取得する問題文です',
            'operation_sequence',
            ?, ?, ?, ?, NULL, 1
        )
        "#,
    )
    .bind(problem_id)
    .bind(room_id)
    .bind(Json(json!([
        {
            "type": "image",
            "object_key": "private/problem-assets/mariadb-test.png",
            "alt": "MariaDBテスト画像"
        }
    ])))
    .bind(Json(json!({
        "query": {
            "type": "operation_sequence",
            "allowed_controls": ["up", "down"],
            "max_operations": 20
        },
        "answer": {
            "type": "string",
            "max_length": 40
        }
    })))
    .bind(Json(json!([
        {
            "body_markdown": "非公開ヒント1"
        },
        {
            "body_markdown": "非公開ヒント2"
        }
    ])))
    .bind(Json(json!({
        "type": "operation_sequence",
        "correct_operations": [
            {
                "control": "up",
                "count": 1
            }
        ],
        "candidates": []
    })))
    .execute(&pool)
    .await
    .expect("test problem should be inserted");

    sqlx::query(
        r#"
        INSERT INTO runs (
            run_id,
            user_id,
            room_id,
            status,
            started_at,
            cleared_at
        )
        VALUES (?, ?, ?, 'active', CURRENT_TIMESTAMP(3), NULL)
        "#,
    )
    .bind(run_id)
    .bind(user_id)
    .bind(room_id)
    .execute(&pool)
    .await
    .expect("test run should be inserted");

    sqlx::query(
        r#"
        INSERT INTO problem_progress (
            run_id,
            problem_id,
            status,
            answer_attempt_count,
            cleared_at
        )
        VALUES (?, ?, 'available', 0, NULL)
        "#,
    )
    .bind(run_id)
    .bind(problem_id)
    .execute(&pool)
    .await
    .expect("test problem progress should be inserted");

    let repository = SqlxUserRepository::new(pool.clone());

    let record = repository
        .find_problem_for_run(run_id, room_id, problem_id)
        .await
        .expect("problem lookup should succeed")
        .expect("problem should be found for the active run");

    assert_eq!(record.id, problem_id);
    assert_eq!(record.number, 1);
    assert_eq!(record.problem_type, "small");
    assert_eq!(record.title, "MariaDB test problem");
    assert_eq!(record.body_markdown, "MariaDBから取得する問題文です");
    assert_eq!(record.submission_type, "operation_sequence");
    assert_eq!(record.status, "available");
    assert_eq!(record.hint_count, 2);

    assert_eq!(record.assets.0.len(), 1);
    assert_eq!(record.assets.0[0].asset_type, "image");
    assert_eq!(
        record.assets.0[0].object_key,
        "private/problem-assets/mariadb-test.png"
    );
    assert_eq!(record.assets.0[0].alt, "MariaDBテスト画像");

    let input_schema =
        serde_json::to_value(&record.input_schema.0).expect("input schema should serialize");

    assert_eq!(input_schema["query"]["type"], "operation_sequence");
    assert_eq!(input_schema["query"]["max_operations"], 20);
    assert_eq!(input_schema["answer"]["type"], "string");
    assert_eq!(input_schema["answer"]["max_length"], 40);

    let wrong_run = repository
        .find_problem_for_run(Uuid::new_v4(), room_id, problem_id)
        .await
        .expect("lookup with another run should succeed");

    assert!(
        wrong_run.is_none(),
        "problem must not be returned for another run"
    );

    let wrong_room = repository
        .find_problem_for_run(run_id, Uuid::new_v4(), problem_id)
        .await
        .expect("lookup with another room should succeed");

    assert!(
        wrong_room.is_none(),
        "problem must not be returned for another room"
    );

    let wrong_problem = repository
        .find_problem_for_run(run_id, room_id, Uuid::new_v4())
        .await
        .expect("lookup with another problem should succeed");

    assert!(
        wrong_problem.is_none(),
        "unknown problem must not be returned"
    );

    sqlx::query("DELETE FROM runs WHERE run_id = ?")
        .bind(run_id)
        .execute(&pool)
        .await
        .expect("test run should be removed");

    sqlx::query("DELETE FROM problems WHERE problem_id = ?")
        .bind(problem_id)
        .execute(&pool)
        .await
        .expect("test problem should be removed");

    sqlx::query("DELETE FROM rooms WHERE room_id = ?")
        .bind(room_id)
        .execute(&pool)
        .await
        .expect("test room should be removed");

    sqlx::query("DELETE FROM users WHERE user_id = ?")
        .bind(user_id)
        .execute(&pool)
        .await
        .expect("test user should be removed");

    pool.close().await;
}
