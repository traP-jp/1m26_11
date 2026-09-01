mod common;

use chrono::{DateTime, Utc};
use common::{connect_test_database, foreign_key_delete_rule, index_columns, primary_key_columns};
use serde_json::json;
use server::{
    migrate,
    repository::{AuthProvider, AuthRepository, RepositoryError, SqlxUserRepository},
};
use sqlx::types::Json;
use uuid::Uuid;

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL pointing to a disposable MariaDB database"]
async fn mariadb_auth_repository_flow() {
    let pool = connect_test_database().await;

    migrate(&pool).await.expect("migration should succeed");

    let repository = SqlxUserRepository::new(pool.clone());

    let session_id = Uuid::new_v4();
    let demo_subject = format!("integration-demo-{}", Uuid::new_v4());

    let created_demo_user = repository
        .get_or_create_demo_user_and_session(session_id, &demo_subject, "demo-user")
        .await
        .expect("demo login creation should succeed");

    let resolved_demo_user = repository
        .find_user_by_demo_session(session_id)
        .await
        .expect("demo session lookup should succeed")
        .expect("demo session should resolve a user");

    let neo_subject = format!("integration-neoshowcase-{}", Uuid::new_v4());

    let first_neo_user = repository
        .get_or_create_user(AuthProvider::NeoShowcase, &neo_subject, "neo-user")
        .await
        .expect("first NeoShowcase lookup should succeed");

    let second_neo_user = repository
        .get_or_create_user(AuthProvider::NeoShowcase, &neo_subject, "neo-user")
        .await
        .expect("second NeoShowcase lookup should succeed");

    sqlx::query("DELETE FROM demo_sessions WHERE session_id = ?")
        .bind(session_id)
        .execute(&pool)
        .await
        .expect("demo session cleanup should succeed");

    sqlx::query("DELETE FROM users WHERE user_id = ?")
        .bind(created_demo_user.user_id)
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

    assert_eq!(created_demo_user, resolved_demo_user);
    assert_eq!(created_demo_user.display_name, "demo-user");
    assert_eq!(created_demo_user.auth_provider, AuthProvider::Demo);
    assert_eq!(first_neo_user, second_neo_user);
    assert_eq!(first_neo_user.display_name, "neo-user");
    assert_eq!(first_neo_user.auth_provider, AuthProvider::NeoShowcase,);
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL pointing to a disposable MariaDB database"]
async fn mariadb_demo_login_rolls_back_user_when_session_creation_fails() {
    let pool = connect_test_database().await;
    migrate(&pool).await.expect("migration should succeed");

    let repository = SqlxUserRepository::new(pool.clone());

    let existing_user_id = Uuid::new_v4();
    let occupied_session_id = Uuid::new_v4();
    let existing_subject = format!("integration-existing-demo-{}", Uuid::new_v4());
    let failed_subject = format!("integration-rollback-demo-{}", Uuid::new_v4());

    sqlx::query(
        r#"
        INSERT INTO users (
            user_id,
            auth_provider,
            provider_subject,
            display_name
        )
        VALUES (?, 'demo', ?, 'existing-demo-user')
        "#,
    )
    .bind(existing_user_id)
    .bind(&existing_subject)
    .execute(&pool)
    .await
    .expect("existing demo user insertion should succeed");

    sqlx::query(
        r#"
        INSERT INTO demo_sessions (session_id, user_id)
        VALUES (?, ?)
        "#,
    )
    .bind(occupied_session_id)
    .bind(existing_user_id)
    .execute(&pool)
    .await
    .expect("existing demo session insertion should succeed");

    let result = repository
        .get_or_create_demo_user_and_session(
            occupied_session_id,
            &failed_subject,
            "must-be-rolled-back",
        )
        .await;

    assert!(matches!(result, Err(RepositoryError::Database(_))));

    let failed_user_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM users
        WHERE auth_provider = 'demo'
          AND provider_subject = ?
        "#,
    )
    .bind(&failed_subject)
    .fetch_one(&pool)
    .await
    .expect("rolled-back user lookup should succeed");

    assert_eq!(failed_user_count, 0);

    sqlx::query("DELETE FROM demo_sessions WHERE session_id = ?")
        .bind(occupied_session_id)
        .execute(&pool)
        .await
        .expect("existing demo session cleanup should succeed");

    sqlx::query("DELETE FROM users WHERE user_id = ?")
        .bind(existing_user_id)
        .execute(&pool)
        .await
        .expect("existing demo user cleanup should succeed");

    pool.close().await;
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

    let query_count_column_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM information_schema.columns
        WHERE table_schema = DATABASE()
          AND column_name = 'query_count'
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("query_count column lookup should succeed");

    assert_eq!(
        query_count_column_count, 0,
        "query_count must be derived from query history, not stored in a dedicated column",
    );

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

    assert_eq!(record.judge_config.0["type"], "operation_sequence");
    assert_eq!(
        record.judge_config.0["correct_operations"][0]["control"],
        "up"
    );
    assert_eq!(record.judge_config.0["correct_operations"][0]["count"], 1);

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

fn leaderboard_timestamp(value: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(value)
        .expect("leaderboard test timestamp should be valid")
        .with_timezone(&Utc)
}

#[expect(
    clippy::too_many_arguments,
    reason = "test fixture helper keeps each stored leaderboard field explicit"
)]
async fn insert_cleared_leaderboard_run(
    pool: &sqlx::MySqlPool,
    run_id: Uuid,
    user_id: Uuid,
    room_id: Uuid,
    query_problem_id: Uuid,
    string_problem_id: Uuid,
    started_at: DateTime<Utc>,
    cleared_at: DateTime<Utc>,
    query_attempt_count: i32,
    string_attempt_count: i32,
) {
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
        VALUES (?, ?, ?, 'cleared', ?, ?)
        "#,
    )
    .bind(run_id)
    .bind(user_id)
    .bind(room_id)
    .bind(started_at)
    .bind(cleared_at)
    .execute(pool)
    .await
    .expect("cleared leaderboard run should be inserted");

    sqlx::query(
        r#"
        INSERT INTO problem_progress (
            run_id,
            problem_id,
            status,
            answer_attempt_count,
            cleared_at
        )
        VALUES
            (?, ?, 'cleared', ?, ?),
            (?, ?, 'cleared', ?, ?)
        "#,
    )
    .bind(run_id)
    .bind(query_problem_id)
    .bind(query_attempt_count)
    .bind(cleared_at)
    .bind(run_id)
    .bind(string_problem_id)
    .bind(string_attempt_count)
    .bind(cleared_at)
    .execute(pool)
    .await
    .expect("leaderboard problem progress should be inserted");
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL pointing to a disposable MariaDB database"]
async fn mariadb_leaderboard_selects_user_best_runs_and_competition_ranks() {
    let pool = connect_test_database().await;
    migrate(&pool).await.expect("migration should succeed");

    let room_id = Uuid::new_v4();
    let query_problem_id = Uuid::new_v4();
    let string_problem_id = Uuid::new_v4();

    let alice_id = Uuid::parse_str("44444444-4444-4444-8444-444444444444")
        .expect("Alice UUID should be valid");
    let bob_id =
        Uuid::parse_str("55555555-5555-4555-8555-555555555555").expect("Bob UUID should be valid");
    let carol_id = Uuid::parse_str("66666666-6666-4666-8666-666666666666")
        .expect("Carol UUID should be valid");

    let room_number = (room_id.as_u128() % 2_000_000_000) as i32 + 1;

    for (user_id, display_name) in [(alice_id, "Alice"), (bob_id, "Bob"), (carol_id, "Carol")] {
        let provider_subject = format!("leaderboard-test-{room_id}-{display_name}");

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
        .bind(user_id)
        .bind(provider_subject)
        .bind(display_name)
        .execute(&pool)
        .await
        .expect("leaderboard user should be inserted");
    }

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
            ?, ?, 'leaderboard-test-room',
            'test', 'leaderboard repository test', 1
        )
        "#,
    )
    .bind(room_id)
    .bind(room_number)
    .execute(&pool)
    .await
    .expect("leaderboard room should be inserted");

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
            ?, ?, 1, 'small', 'query problem', 'query body',
            'operation_sequence',
            JSON_ARRAY(), JSON_OBJECT(), JSON_ARRAY(), JSON_OBJECT(),
            NULL, 1
        )
        "#,
    )
    .bind(query_problem_id)
    .bind(room_id)
    .execute(&pool)
    .await
    .expect("query problem should be inserted");

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
            ?, ?, 2, 'small', 'string problem', 'string body',
            'string',
            JSON_ARRAY(), JSON_OBJECT(), JSON_ARRAY(), JSON_OBJECT(),
            NULL, 1
        )
        "#,
    )
    .bind(string_problem_id)
    .bind(room_id)
    .execute(&pool)
    .await
    .expect("string problem should be inserted");

    // Alice: elapsedは同じだがquery_countが3なので採用されないrun。
    insert_cleared_leaderboard_run(
        &pool,
        Uuid::new_v4(),
        alice_id,
        room_id,
        query_problem_id,
        string_problem_id,
        leaderboard_timestamp("2026-08-06T10:00:00Z"),
        leaderboard_timestamp("2026-08-06T10:01:00Z"),
        3,
        99,
    )
    .await;

    // Alice: query_countが2で、同条件の中ではcleared_atも早いbest run。
    let tied_cleared_at = leaderboard_timestamp("2026-08-06T10:05:00Z");

    insert_cleared_leaderboard_run(
        &pool,
        Uuid::new_v4(),
        alice_id,
        room_id,
        query_problem_id,
        string_problem_id,
        leaderboard_timestamp("2026-08-06T10:04:00Z"),
        tied_cleared_at,
        2,
        99,
    )
    .await;

    // Alice: elapsedとquery_countは同じだがcleared_atが遅いため採用されないrun。
    insert_cleared_leaderboard_run(
        &pool,
        Uuid::new_v4(),
        alice_id,
        room_id,
        query_problem_id,
        string_problem_id,
        leaderboard_timestamp("2026-08-06T10:09:00Z"),
        leaderboard_timestamp("2026-08-06T10:10:00Z"),
        2,
        99,
    )
    .await;

    // Bob: Aliceのbest runと3項目が完全に同じなので同率1位。
    insert_cleared_leaderboard_run(
        &pool,
        Uuid::new_v4(),
        bob_id,
        room_id,
        query_problem_id,
        string_problem_id,
        leaderboard_timestamp("2026-08-06T10:04:00Z"),
        tied_cleared_at,
        2,
        88,
    )
    .await;

    // Carol: Alice・Bobより遅いため、competition rankingで3位。
    insert_cleared_leaderboard_run(
        &pool,
        Uuid::new_v4(),
        carol_id,
        room_id,
        query_problem_id,
        string_problem_id,
        leaderboard_timestamp("2026-08-06T10:05:00Z"),
        leaderboard_timestamp("2026-08-06T10:06:10.123Z"),
        1,
        77,
    )
    .await;

    // active runは、仮に最速に見えるstarted_atでもleaderboardへ含めない。
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
        VALUES (?, ?, ?, 'active', ?, NULL)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(alice_id)
    .bind(room_id)
    .bind(leaderboard_timestamp("2026-08-06T10:20:00Z"))
    .execute(&pool)
    .await
    .expect("active run should be inserted");

    let repository = SqlxUserRepository::new(pool.clone());

    let leaderboard = repository
        .find_leaderboard_by_room_id(room_id)
        .await
        .expect("leaderboard lookup should succeed");

    assert_eq!(leaderboard.len(), 3);

    assert_eq!(leaderboard[0].rank, 1);
    assert_eq!(leaderboard[0].user_id, alice_id);
    assert_eq!(leaderboard[0].display_name, "Alice");
    assert_eq!(leaderboard[0].elapsed_ms, 60_000);
    assert_eq!(leaderboard[0].query_count, 2);
    assert_eq!(leaderboard[0].cleared_at, tied_cleared_at);

    assert_eq!(leaderboard[1].rank, 1);
    assert_eq!(leaderboard[1].user_id, bob_id);
    assert_eq!(leaderboard[1].display_name, "Bob");
    assert_eq!(leaderboard[1].elapsed_ms, 60_000);
    assert_eq!(leaderboard[1].query_count, 2);
    assert_eq!(leaderboard[1].cleared_at, tied_cleared_at);

    assert_eq!(leaderboard[2].rank, 3);
    assert_eq!(leaderboard[2].user_id, carol_id);
    assert_eq!(leaderboard[2].display_name, "Carol");
    assert_eq!(leaderboard[2].elapsed_ms, 70_123);
    assert_eq!(leaderboard[2].query_count, 1);

    let empty = repository
        .find_leaderboard_by_room_id(Uuid::new_v4())
        .await
        .expect("unknown room leaderboard lookup should succeed");

    assert!(empty.is_empty());

    sqlx::query("DELETE FROM runs WHERE room_id = ?")
        .bind(room_id)
        .execute(&pool)
        .await
        .expect("leaderboard runs should be removed");

    sqlx::query("DELETE FROM problems WHERE room_id = ?")
        .bind(room_id)
        .execute(&pool)
        .await
        .expect("leaderboard problems should be removed");

    sqlx::query("DELETE FROM rooms WHERE room_id = ?")
        .bind(room_id)
        .execute(&pool)
        .await
        .expect("leaderboard room should be removed");

    for user_id in [alice_id, bob_id, carol_id] {
        sqlx::query("DELETE FROM users WHERE user_id = ?")
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("leaderboard user should be removed");
    }

    pool.close().await;
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL pointing to a disposable MariaDB database"]
async fn mariadb_user_progress_counts_public_rooms_by_genre_without_duplicate_clears() {
    let pool = connect_test_database().await;
    migrate(&pool).await.expect("migration should succeed");

    let user_id = Uuid::new_v4();
    let no_clear_user_id = Uuid::new_v4();

    for (id, display_name) in [
        (user_id, "progress-user"),
        (no_clear_user_id, "progress-no-clear-user"),
    ] {
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
        .bind(id)
        .bind(format!("progress-test-{id}"))
        .bind(display_name)
        .execute(&pool)
        .await
        .expect("progress test user should be inserted");
    }

    let public_osint_cleared = Uuid::new_v4();
    let public_osint_active = Uuid::new_v4();
    let public_web_cleared = Uuid::new_v4();
    let private_osint_cleared = Uuid::new_v4();

    let room_number_base = (Uuid::new_v4().as_u128() % 1_900_000_000) as i32 + 1;

    for (index, room_id, genre, is_published) in [
        (0, public_osint_cleared, "OSINT", true),
        (1, public_osint_active, "OSINT", true),
        (2, public_web_cleared, "Web", true),
        (3, private_osint_cleared, "OSINT", false),
    ] {
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
            VALUES (?, ?, ?, ?, 'progress repository test', ?)
            "#,
        )
        .bind(room_id)
        .bind(room_number_base + index)
        .bind(format!("progress-room-{room_id}"))
        .bind(genre)
        .bind(is_published)
        .execute(&pool)
        .await
        .expect("progress test room should be inserted");
    }

    let started_at = DateTime::parse_from_rfc3339("2026-08-06T10:00:00Z")
        .expect("started_at should be valid")
        .with_timezone(&Utc);

    let cleared_at = DateTime::parse_from_rfc3339("2026-08-06T10:01:00Z")
        .expect("cleared_at should be valid")
        .with_timezone(&Utc);

    // 同じ公開OSINT roomを2回clearしても1件として数える。
    for room_id in [
        public_osint_cleared,
        public_osint_cleared,
        public_web_cleared,
        private_osint_cleared,
    ] {
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
            VALUES (?, ?, ?, 'cleared', ?, ?)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(room_id)
        .bind(started_at)
        .bind(cleared_at)
        .execute(&pool)
        .await
        .expect("cleared progress run should be inserted");
    }

    // active runはclear数へ含めない。
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
        VALUES (?, ?, ?, 'active', ?, NULL)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(public_osint_active)
    .bind(started_at)
    .execute(&pool)
    .await
    .expect("active progress run should be inserted");

    let repository = SqlxUserRepository::new(pool.clone());

    let progress = repository
        .find_user_progress(user_id)
        .await
        .expect("user progress lookup should succeed");

    assert_eq!(progress.cleared_room_count, 2);
    assert_eq!(progress.total_room_count, 3);
    assert_eq!(progress.by_genre.len(), 2);

    assert_eq!(progress.by_genre[0].genre, "OSINT");
    assert_eq!(progress.by_genre[0].cleared_room_count, 1);
    assert_eq!(progress.by_genre[0].total_room_count, 2);

    assert_eq!(progress.by_genre[1].genre, "Web");
    assert_eq!(progress.by_genre[1].cleared_room_count, 1);
    assert_eq!(progress.by_genre[1].total_room_count, 1);

    let no_clear_progress = repository
        .find_user_progress(no_clear_user_id)
        .await
        .expect("no-clear user progress lookup should succeed");

    assert_eq!(no_clear_progress.cleared_room_count, 0);
    assert_eq!(no_clear_progress.total_room_count, 3);
    assert_eq!(
        no_clear_progress
            .by_genre
            .iter()
            .map(|progress| progress.cleared_room_count)
            .collect::<Vec<_>>(),
        vec![0, 0]
    );

    sqlx::query("DELETE FROM runs WHERE user_id = ?")
        .bind(user_id)
        .execute(&pool)
        .await
        .expect("progress runs should be removed");

    for room_id in [
        public_osint_cleared,
        public_osint_active,
        public_web_cleared,
        private_osint_cleared,
    ] {
        sqlx::query("DELETE FROM rooms WHERE room_id = ?")
            .bind(room_id)
            .execute(&pool)
            .await
            .expect("progress room should be removed");
    }

    let empty_progress = repository
        .find_user_progress(user_id)
        .await
        .expect("empty progress lookup should succeed");

    assert_eq!(empty_progress.cleared_room_count, 0);
    assert_eq!(empty_progress.total_room_count, 0);
    assert!(empty_progress.by_genre.is_empty());

    for id in [user_id, no_clear_user_id] {
        sqlx::query("DELETE FROM users WHERE user_id = ?")
            .bind(id)
            .execute(&pool)
            .await
            .expect("progress test user should be removed");
    }

    pool.close().await;
}
