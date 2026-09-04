mod common;

use chrono::{DateTime, Duration, Utc};
use common::{connect_test_database, foreign_key_delete_rule, index_columns, primary_key_columns};
use openapi_generated::models::CreateProblemRequest;
use serde_json::{Value, json};
use server::{
    migrate,
    problem::{Asset, validate_problem_draft},
    repository::{
        AssetUploadClaimOutcome, AssetUploadClaimRequest, AuthProvider, AuthRepository,
        CompleteAssetUploadRequest, CreateProblemRecordOutcome, CreateProblemRecordRequest,
        RepositoryError, SqlxUserRepository,
    },
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
            "asset_upload_idempotency".to_owned(),
            "demo_sessions".to_owned(),
            "problem_create_idempotency".to_owned(),
            "problem_progress".to_owned(),
            "problems".to_owned(),
            "queries".to_owned(),
            "rooms".to_owned(),
            "runs".to_owned(),
            "users".to_owned(),
        ]
    );

    let primary_keys: &[(&str, &[&str])] = &[
        (
            "asset_upload_idempotency",
            &["request_method", "request_path", "idempotency_key"],
        ),
        (
            "problem_create_idempotency",
            &["request_method", "request_path", "idempotency_key"],
        ),
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
        (
            "asset_upload_idempotency",
            "idx_asset_upload_idempotency_expires_at",
            &["expires_at"],
        ),
        (
            "problem_create_idempotency",
            "idx_problem_create_idempotency_problem_id",
            &["problem_id"],
        ),
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
        ("fk_problem_create_idempotency_problem_id", "CASCADE"),
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
async fn mariadb_asset_upload_idempotency_schema_enforces_constraints() {
    let pool = connect_test_database().await;

    migrate(&pool).await.expect("migration should succeed");

    let idempotency_key = Uuid::new_v4();
    let claim_token = Uuid::new_v4();
    let competing_claim_token = Uuid::new_v4();
    let invalid_claim_token = Uuid::new_v4();
    let request_path = format!(
        "/api/rooms/{}/problems/{}/assets",
        Uuid::new_v4(),
        Uuid::new_v4()
    );
    let file_sha256 = vec![0xabu8; 32];

    sqlx::query(
        r#"
        INSERT INTO asset_upload_idempotency (
            request_method,
            request_path,
            idempotency_key,
            claim_token,
            file_sha256,
            alt,
            status,
            expires_at
        )
        VALUES (
            'POST',
            ?,
            ?,
            ?,
            ?,
            ?,
            'processing',
            DATE_ADD(CURRENT_TIMESTAMP(3), INTERVAL 24 HOUR)
        )
        "#,
    )
    .bind(&request_path)
    .bind(idempotency_key)
    .bind(claim_token.as_bytes().as_slice())
    .bind(&file_sha256)
    .bind("テスト画像")
    .execute(&pool)
    .await
    .expect("valid processing idempotency record should be inserted");

    let duplicate_result = sqlx::query(
        r#"
        INSERT INTO asset_upload_idempotency (
            request_method,
            request_path,
            idempotency_key,
            claim_token,
            file_sha256,
            alt,
            status,
            expires_at
        )
        VALUES (
            'POST',
            ?,
            ?,
            ?,
            ?,
            ?,
            'processing',
            DATE_ADD(CURRENT_TIMESTAMP(3), INTERVAL 24 HOUR)
        )
        "#,
    )
    .bind(&request_path)
    .bind(idempotency_key)
    .bind(competing_claim_token.as_bytes().as_slice())
    .bind(&file_sha256)
    .bind("テスト画像")
    .execute(&pool)
    .await;

    assert!(
        duplicate_result.is_err(),
        "method, path, and idempotency key must be unique"
    );

    let invalid_completed_result = sqlx::query(
        r#"
        INSERT INTO asset_upload_idempotency (
            request_method,
            request_path,
            idempotency_key,
            claim_token,
            file_sha256,
            alt,
            status,
            expires_at
        )
        VALUES (
            'POST',
            ?,
            ?,
            ?,
            ?,
            ?,
            'completed',
            DATE_ADD(CURRENT_TIMESTAMP(3), INTERVAL 24 HOUR)
        )
        "#,
    )
    .bind(format!("{request_path}/invalid"))
    .bind(Uuid::new_v4())
    .bind(invalid_claim_token.as_bytes().as_slice())
    .bind(&file_sha256)
    .bind("不正な完了record")
    .execute(&pool)
    .await;

    assert!(
        invalid_completed_result.is_err(),
        "completed record must contain object_key and completed_at"
    );

    sqlx::query(
        r#"
        DELETE FROM asset_upload_idempotency
        WHERE request_method = 'POST'
          AND request_path = ?
          AND idempotency_key = ?
        "#,
    )
    .bind(&request_path)
    .bind(idempotency_key)
    .execute(&pool)
    .await
    .expect("idempotency test record cleanup should succeed");

    pool.close().await;
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL pointing to a disposable MariaDB database"]
async fn mariadb_asset_upload_repository_handles_claim_replay_and_completion() {
    let pool = connect_test_database().await;

    migrate(&pool).await.expect("migration should succeed");

    let repository = SqlxUserRepository::new(pool.clone());

    let room_id = Uuid::new_v4();
    let problem_id = Uuid::new_v4();
    let room_number = (room_id.as_u128() % 2_000_000_000) as i32 + 1;
    let request_path = format!("/api/rooms/{room_id}/problems/{problem_id}/assets");

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
            ?,
            ?,
            'asset-upload-repository-test-room',
            'test',
            'asset upload repository integration test',
            0
        )
        "#,
    )
    .bind(room_id)
    .bind(room_number)
    .execute(&pool)
    .await
    .expect("unpublished test room should be inserted");

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
            ?,
            ?,
            1,
            'small',
            'asset upload repository test problem',
            'asset upload repository test body',
            'string',
            JSON_ARRAY(),
            JSON_OBJECT(),
            JSON_ARRAY(),
            JSON_OBJECT(),
            NULL,
            1
        )
        "#,
    )
    .bind(problem_id)
    .bind(room_id)
    .execute(&pool)
    .await
    .expect("test problem should be inserted");

    let target = repository
        .find_asset_upload_target(room_id, problem_id)
        .await
        .expect("asset upload target lookup should succeed")
        .expect("room and problem combination should exist");

    assert!(
        !target.is_published,
        "new asset upload target should be unpublished"
    );

    let wrong_problem = repository
        .find_asset_upload_target(room_id, Uuid::new_v4())
        .await
        .expect("unknown problem lookup should succeed");

    assert!(
        wrong_problem.is_none(),
        "unknown problem must not be accepted as an upload target"
    );

    let wrong_room = repository
        .find_asset_upload_target(Uuid::new_v4(), problem_id)
        .await
        .expect("wrong room lookup should succeed");

    assert!(
        wrong_room.is_none(),
        "problem must not be accepted for another room"
    );

    let claim_request = AssetUploadClaimRequest {
        request_method: "POST".to_owned(),
        request_path: request_path.clone(),
        idempotency_key: Uuid::new_v4(),
        file_sha256: [0xabu8; 32],
        alt: "テスト画像".to_owned(),
        expires_at: Utc::now() + Duration::hours(24),
    };

    let (first_claim, second_claim) = tokio::join!(
        repository.claim_asset_upload(&claim_request),
        repository.claim_asset_upload(&claim_request),
    );

    let first_claim = first_claim.expect("first concurrent claim should succeed");
    let second_claim = second_claim.expect("second concurrent claim should succeed");

    let claim_token = match (first_claim, second_claim) {
        (
            AssetUploadClaimOutcome::Acquired { claim_token },
            AssetUploadClaimOutcome::InProgress,
        )
        | (
            AssetUploadClaimOutcome::InProgress,
            AssetUploadClaimOutcome::Acquired { claim_token },
        ) => claim_token,
        outcomes => panic!("unexpected concurrent claim outcomes: {outcomes:?}"),
    };

    let different_alt_request = AssetUploadClaimRequest {
        alt: "異なる説明".to_owned(),
        ..claim_request.clone()
    };

    let different_alt_outcome = repository
        .claim_asset_upload(&different_alt_request)
        .await
        .expect("different alt claim should be checked");

    assert_eq!(
        different_alt_outcome,
        AssetUploadClaimOutcome::Reused,
        "same key with a different alt must be rejected as reused"
    );

    let different_file_request = AssetUploadClaimRequest {
        file_sha256: [0xcdu8; 32],
        ..claim_request.clone()
    };

    let different_file_outcome = repository
        .claim_asset_upload(&different_file_request)
        .await
        .expect("different file claim should be checked");

    assert_eq!(
        different_file_outcome,
        AssetUploadClaimOutcome::Reused,
        "same key with a different file must be rejected as reused"
    );

    let asset = Asset {
        asset_type: "image".to_owned(),
        object_key: format!("v1/problems/{room_id}/{problem_id}/{}.png", Uuid::new_v4()),
        alt: claim_request.alt.clone(),
    };

    let completed_at = Utc::now();

    repository
        .complete_asset_upload(&CompleteAssetUploadRequest {
            request_method: claim_request.request_method.clone(),
            request_path: claim_request.request_path.clone(),
            idempotency_key: claim_request.idempotency_key,
            claim_token,
            room_id,
            problem_id,
            asset: asset.clone(),
            completed_at,
        })
        .await
        .expect("asset upload completion should succeed");

    let stored_assets = sqlx::query_scalar::<_, Json<Vec<Asset>>>(
        r#"
        SELECT assets
        FROM problems
        WHERE problem_id = ?
        "#,
    )
    .bind(problem_id)
    .fetch_one(&pool)
    .await
    .expect("stored problem assets should be readable");

    assert_eq!(
        stored_assets.0,
        vec![asset.clone()],
        "completed asset must be appended to problems.assets"
    );

    let (stored_status, stored_object_key, stored_completed_at) =
        sqlx::query_as::<_, (String, Option<String>, Option<DateTime<Utc>>)>(
            r#"
            SELECT
                status,
                CAST(
                    object_key AS CHAR CHARACTER SET utf8mb4
                ) AS object_key,
                completed_at
            FROM asset_upload_idempotency
            WHERE request_method = ?
              AND request_path = ?
              AND idempotency_key = ?
            "#,
        )
        .bind(&claim_request.request_method)
        .bind(&claim_request.request_path)
        .bind(claim_request.idempotency_key)
        .fetch_one(&pool)
        .await
        .expect("completed idempotency record should be readable");

    assert_eq!(stored_status, "completed");
    assert_eq!(
        stored_object_key.as_deref(),
        Some(asset.object_key.as_str())
    );
    assert_eq!(
        stored_completed_at.map(|value| value.timestamp_millis()),
        Some(completed_at.timestamp_millis()),
        "completed_at must be stored with MariaDB TIMESTAMP(3) precision"
    );

    let replay_outcome = repository
        .claim_asset_upload(&claim_request)
        .await
        .expect("completed request replay should succeed");

    assert_eq!(
        replay_outcome,
        AssetUploadClaimOutcome::Completed {
            asset: asset.clone(),
        },
        "same key, file, and alt must return the first completed asset"
    );

    let release_request = AssetUploadClaimRequest {
        idempotency_key: Uuid::new_v4(),
        file_sha256: [0xdeu8; 32],
        alt: "解放確認画像".to_owned(),
        ..claim_request.clone()
    };

    let first_release_claim = repository
        .claim_asset_upload(&release_request)
        .await
        .expect("release test claim should succeed");

    let first_release_token = match first_release_claim {
        AssetUploadClaimOutcome::Acquired { claim_token } => claim_token,
        outcome => panic!("release test should acquire a claim, got {outcome:?}"),
    };

    repository
        .release_asset_upload_claim(
            &release_request.request_method,
            &release_request.request_path,
            release_request.idempotency_key,
            first_release_token,
        )
        .await
        .expect("owned processing claim should be released");

    let second_release_claim = repository
        .claim_asset_upload(&release_request)
        .await
        .expect("released request should be claimable again");

    let second_release_token = match second_release_claim {
        AssetUploadClaimOutcome::Acquired { claim_token } => claim_token,
        outcome => panic!("released request should acquire a new claim, got {outcome:?}"),
    };

    assert_ne!(
        first_release_token, second_release_token,
        "reacquired request must receive a new claim token"
    );

    let expired_request = AssetUploadClaimRequest {
        idempotency_key: Uuid::new_v4(),
        file_sha256: [0xe1u8; 32],
        alt: "期限切れ確認画像".to_owned(),
        ..claim_request.clone()
    };

    let initial_expired_claim = repository
        .claim_asset_upload(&expired_request)
        .await
        .expect("expiration test claim should succeed");

    let initial_expired_token = match initial_expired_claim {
        AssetUploadClaimOutcome::Acquired { claim_token } => claim_token,
        outcome => panic!("expiration test should acquire a claim, got {outcome:?}"),
    };

    sqlx::query(
        r#"
        UPDATE asset_upload_idempotency
        SET created_at = DATE_SUB(CURRENT_TIMESTAMP(3), INTERVAL 25 HOUR),
            expires_at = DATE_SUB(CURRENT_TIMESTAMP(3), INTERVAL 1 HOUR)
        WHERE request_method = ?
          AND request_path = ?
          AND idempotency_key = ?
        "#,
    )
    .bind(&expired_request.request_method)
    .bind(&expired_request.request_path)
    .bind(expired_request.idempotency_key)
    .execute(&pool)
    .await
    .expect("expiration test record should be moved into the past");

    let renewed_expired_claim = repository
        .claim_asset_upload(&expired_request)
        .await
        .expect("expired request should be claimable again");

    let renewed_expired_token = match renewed_expired_claim {
        AssetUploadClaimOutcome::Acquired { claim_token } => claim_token,
        outcome => panic!("expired request should acquire a new claim, got {outcome:?}"),
    };

    assert_ne!(
        initial_expired_token, renewed_expired_token,
        "expired request must receive a new claim token"
    );

    let publish_race_request = AssetUploadClaimRequest {
        idempotency_key: Uuid::new_v4(),
        file_sha256: [0xefu8; 32],
        alt: "公開競合確認画像".to_owned(),
        ..claim_request.clone()
    };

    let publish_race_claim = repository
        .claim_asset_upload(&publish_race_request)
        .await
        .expect("publish race claim should succeed");

    let publish_race_token = match publish_race_claim {
        AssetUploadClaimOutcome::Acquired { claim_token } => claim_token,
        outcome => panic!("publish race request should acquire a claim, got {outcome:?}"),
    };

    sqlx::query(
        r#"
        UPDATE rooms
        SET is_published = 1
        WHERE room_id = ?
        "#,
    )
    .bind(room_id)
    .execute(&pool)
    .await
    .expect("test room should be published");

    let published_target = repository
        .find_asset_upload_target(room_id, problem_id)
        .await
        .expect("published target lookup should succeed")
        .expect("published room and problem should still exist");

    assert!(
        published_target.is_published,
        "repository must expose that the target room is published"
    );

    let publish_race_asset = Asset {
        asset_type: "image".to_owned(),
        object_key: format!("v1/problems/{room_id}/{problem_id}/{}.webp", Uuid::new_v4()),
        alt: publish_race_request.alt.clone(),
    };

    let publish_race_result = repository
        .complete_asset_upload(&CompleteAssetUploadRequest {
            request_method: publish_race_request.request_method.clone(),
            request_path: publish_race_request.request_path.clone(),
            idempotency_key: publish_race_request.idempotency_key,
            claim_token: publish_race_token,
            room_id,
            problem_id,
            asset: publish_race_asset,
            completed_at: Utc::now(),
        })
        .await;

    assert!(
        matches!(
            publish_race_result,
            Err(RepositoryError::PublishedRoomImmutable)
        ),
        "room published during upload must reject the database completion"
    );

    let assets_after_publish_conflict = sqlx::query_scalar::<_, Json<Vec<Asset>>>(
        r#"
        SELECT assets
        FROM problems
        WHERE problem_id = ?
        "#,
    )
    .bind(problem_id)
    .fetch_one(&pool)
    .await
    .expect("assets after publish conflict should be readable");

    assert_eq!(
        assets_after_publish_conflict.0,
        vec![asset],
        "published room conflict must not append another asset"
    );

    sqlx::query(
        r#"
        DELETE FROM asset_upload_idempotency
        WHERE request_method = 'POST'
          AND request_path = ?
        "#,
    )
    .bind(&request_path)
    .execute(&pool)
    .await
    .expect("asset upload idempotency records should be removed");

    sqlx::query("DELETE FROM problems WHERE problem_id = ?")
        .bind(problem_id)
        .execute(&pool)
        .await
        .expect("asset upload test problem should be removed");

    sqlx::query("DELETE FROM rooms WHERE room_id = ?")
        .bind(room_id)
        .execute(&pool)
        .await
        .expect("asset upload test room should be removed");

    pool.close().await;
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL pointing to a disposable MariaDB database"]
async fn mariadb_problem_creation_is_transactional_and_idempotent() {
    const OPERATION_REQUEST: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../openapi/examples/problems/create-operation-sequence-request.json"
    ));

    let pool = connect_test_database().await;
    migrate(&pool).await.expect("migration should succeed");

    let repository = SqlxUserRepository::new(pool.clone());

    let room_id = Uuid::new_v4();
    let other_room_id = Uuid::new_v4();
    let dependency_id = Uuid::new_v4();
    let other_room_problem_id = Uuid::new_v4();

    let room_number = (room_id.as_u128() % 1_000_000_000) as i32 + 1;
    let other_room_number = room_number + 1;

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
        VALUES (?, ?, 'problem-create-test-room', 'test', 'problem creation test', 0)
        "#,
    )
    .bind(room_id)
    .bind(room_number)
    .execute(&pool)
    .await
    .expect("unpublished authoring room should be inserted");

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
        VALUES (?, ?, 'other-problem-create-test-room', 'test', 'other test room', 0)
        "#,
    )
    .bind(other_room_id)
    .bind(other_room_number)
    .execute(&pool)
    .await
    .expect("other authoring room should be inserted");

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
            ?, ?, 1, 'small', 'dependency problem', 'dependency body', 'string',
            JSON_ARRAY(),
            JSON_OBJECT(
                'query',
                JSON_OBJECT(
                    'type', 'operation_sequence',
                    'allowed_controls', JSON_ARRAY('down'),
                    'max_operations', 100
                ),
                'answer',
                JSON_OBJECT('type', 'string', 'max_length', 50)
            ),
            JSON_ARRAY(),
            JSON_OBJECT(
                'type', 'string',
                'accepted_answers', JSON_ARRAY('answer'),
                'normalization',
                JSON_OBJECT(
                    'unicode', 'nfkc',
                    'trim_outer_whitespace', TRUE,
                    'collapse_internal_whitespace', FALSE,
                    'case_sensitive', FALSE
                )
            ),
            NULL,
            1
        )
        "#,
    )
    .bind(dependency_id)
    .bind(room_id)
    .execute(&pool)
    .await
    .expect("same-room dependency problem should be inserted");

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
            ?, ?, 1, 'small', 'other room problem', 'other room body', 'string',
            JSON_ARRAY(),
            JSON_OBJECT(),
            JSON_ARRAY(),
            JSON_OBJECT(),
            NULL,
            1
        )
        "#,
    )
    .bind(other_room_problem_id)
    .bind(other_room_id)
    .execute(&pool)
    .await
    .expect("other-room dependency problem should be inserted");

    let payload: CreateProblemRequest = serde_json::from_str(OPERATION_REQUEST)
        .expect("operation request fixture should match generated model");

    let mut draft =
        validate_problem_draft(room_id, payload).expect("fixture should produce a valid draft");

    draft.number = 2;
    draft.depends_on_problem_id = Some(dependency_id);

    let request_path = format!("/api/rooms/{room_id}/problems");
    let idempotency_key = Uuid::new_v4();

    let create_request = CreateProblemRecordRequest {
        request_method: "POST".to_owned(),
        request_path: request_path.clone(),
        idempotency_key,
        payload_sha256: [0x11; 32],
        draft: draft.clone(),
    };

    let first_outcome = repository
        .create_problem(&create_request)
        .await
        .expect("first problem creation should succeed");

    let created_problem_id = match first_outcome {
        CreateProblemRecordOutcome::Created { problem_id } => problem_id,
        outcome => panic!("first request should create a problem, got {outcome:?}"),
    };

    let stored = sqlx::query_as::<
        _,
        (
            i32,
            String,
            String,
            String,
            String,
            Json<Vec<Asset>>,
            Json<Value>,
            Json<Value>,
            Json<Value>,
            Option<Uuid>,
            bool,
        ),
    >(
        r#"
        SELECT
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
        FROM problems
        WHERE problem_id = ?
        "#,
    )
    .bind(created_problem_id)
    .fetch_one(&pool)
    .await
    .expect("created problem should be readable");

    assert_eq!(stored.0, draft.number);
    assert_eq!(stored.1, "small");
    assert_eq!(stored.2, draft.title);
    assert_eq!(stored.3, draft.body_markdown);
    assert_eq!(stored.4, "operation_sequence");
    assert!(stored.5.0.is_empty());
    assert_eq!(
        stored.6.0,
        serde_json::to_value(&draft.input_schema).expect("input schema should serialize")
    );
    assert_eq!(
        stored.7.0,
        serde_json::to_value(&draft.hints).expect("hints should serialize")
    );
    assert_eq!(
        stored.8.0,
        serde_json::to_value(&draft.judge_config).expect("judge config should serialize")
    );
    assert_eq!(stored.9, Some(dependency_id));
    assert!(stored.10);

    let replay_outcome = repository
        .create_problem(&create_request)
        .await
        .expect("same request replay should succeed");

    assert_eq!(
        replay_outcome,
        CreateProblemRecordOutcome::Replayed {
            problem_id: created_problem_id,
        }
    );

    let reused_outcome = repository
        .create_problem(&CreateProblemRecordRequest {
            payload_sha256: [0x22; 32],
            ..create_request.clone()
        })
        .await
        .expect("different payload should be classified");

    assert_eq!(reused_outcome, CreateProblemRecordOutcome::Reused);

    let number_conflict_result = repository
        .create_problem(&CreateProblemRecordRequest {
            idempotency_key: Uuid::new_v4(),
            payload_sha256: [0x33; 32],
            ..create_request.clone()
        })
        .await;

    assert!(matches!(
        number_conflict_result,
        Err(RepositoryError::ProblemNumberConflict)
    ));

    let mut invalid_dependency_draft = draft.clone();
    invalid_dependency_draft.number = 3;
    invalid_dependency_draft.depends_on_problem_id = Some(other_room_problem_id);

    let invalid_dependency_result = repository
        .create_problem(&CreateProblemRecordRequest {
            request_method: "POST".to_owned(),
            request_path: request_path.clone(),
            idempotency_key: Uuid::new_v4(),
            payload_sha256: [0x44; 32],
            draft: invalid_dependency_draft,
        })
        .await;

    assert!(matches!(
        invalid_dependency_result,
        Err(RepositoryError::InvalidProblemDependency)
    ));

    let invalid_method_result = sqlx::query(
        r#"
        INSERT INTO problem_create_idempotency (
            request_method,
            request_path,
            idempotency_key,
            payload_sha256,
            problem_id
        )
        VALUES ('GET', ?, ?, ?, ?)
        "#,
    )
    .bind(&request_path)
    .bind(Uuid::new_v4())
    .bind(vec![0x55_u8; 32])
    .bind(created_problem_id)
    .execute(&pool)
    .await;

    assert!(
        invalid_method_result.is_err(),
        "migration must reject methods other than POST"
    );

    sqlx::query(
        r#"
        UPDATE rooms
        SET is_published = 1
        WHERE room_id = ?
        "#,
    )
    .bind(room_id)
    .execute(&pool)
    .await
    .expect("test room should be published");

    let replay_after_publish = repository
        .create_problem(&create_request)
        .await
        .expect("completed request should still replay after publication");

    assert_eq!(
        replay_after_publish,
        CreateProblemRecordOutcome::Replayed {
            problem_id: created_problem_id,
        }
    );

    let mut published_draft = draft.clone();
    published_draft.number = 3;

    let published_result = repository
        .create_problem(&CreateProblemRecordRequest {
            request_method: "POST".to_owned(),
            request_path: request_path.clone(),
            idempotency_key: Uuid::new_v4(),
            payload_sha256: [0x66; 32],
            draft: published_draft,
        })
        .await;

    assert!(matches!(
        published_result,
        Err(RepositoryError::PublishedRoomImmutable)
    ));

    let missing_room_id = Uuid::new_v4();
    let mut missing_room_draft = draft.clone();
    missing_room_draft.room_id = missing_room_id;

    let missing_room_result = repository
        .create_problem(&CreateProblemRecordRequest {
            request_method: "POST".to_owned(),
            request_path: format!("/api/rooms/{missing_room_id}/problems"),
            idempotency_key: Uuid::new_v4(),
            payload_sha256: [0x77; 32],
            draft: missing_room_draft,
        })
        .await;

    assert!(matches!(
        missing_room_result,
        Err(RepositoryError::RoomNotFound)
    ));

    let created_number_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM problems
        WHERE room_id = ?
          AND number = 2
        "#,
    )
    .bind(room_id)
    .fetch_one(&pool)
    .await
    .expect("created problem count should be readable");

    assert_eq!(
        created_number_count, 1,
        "replay and conflicts must not create duplicate problems"
    );

    let idempotency_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM problem_create_idempotency
        WHERE request_method = 'POST'
          AND request_path = ?
          AND idempotency_key = ?
        "#,
    )
    .bind(&request_path)
    .bind(idempotency_key)
    .fetch_one(&pool)
    .await
    .expect("problem creation idempotency record should be readable");

    assert_eq!(idempotency_count, 1);

    sqlx::query(
        r#"
        DELETE FROM problem_create_idempotency
        WHERE request_path = ?
        "#,
    )
    .bind(&request_path)
    .execute(&pool)
    .await
    .expect("problem creation idempotency record should be removed");

    sqlx::query("DELETE FROM problems WHERE problem_id = ?")
        .bind(created_problem_id)
        .execute(&pool)
        .await
        .expect("created problem should be removed");

    sqlx::query("DELETE FROM problems WHERE problem_id = ?")
        .bind(dependency_id)
        .execute(&pool)
        .await
        .expect("dependency problem should be removed");

    sqlx::query("DELETE FROM problems WHERE problem_id = ?")
        .bind(other_room_problem_id)
        .execute(&pool)
        .await
        .expect("other-room problem should be removed");

    sqlx::query("DELETE FROM rooms WHERE room_id = ?")
        .bind(room_id)
        .execute(&pool)
        .await
        .expect("authoring room should be removed");

    sqlx::query("DELETE FROM rooms WHERE room_id = ?")
        .bind(other_room_id)
        .execute(&pool)
        .await
        .expect("other authoring room should be removed");

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

    let problem_count = repository
        .count_problems_by_room_id(room_id)
        .await
        .expect("problem count lookup should succeed");
    assert_eq!(problem_count, 2);

    let empty_problem_count = repository
        .count_problems_by_room_id(Uuid::new_v4())
        .await
        .expect("unknown room problem count should succeed");
    assert_eq!(empty_problem_count, 0);

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

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL pointing to a disposable MariaDB database"]
async fn mariadb_published_rooms_with_progress_and_ranking_flow() {
    let pool = connect_test_database().await;
    migrate(&pool).await.expect("migration should succeed");

    let repository = SqlxUserRepository::new(pool.clone());

    let room1_id = Uuid::new_v4();
    let room2_id = Uuid::new_v4();
    let private_room_id = Uuid::new_v4();

    let user_a_id = Uuid::new_v4();
    let user_b_id = Uuid::new_v4();
    let user_c_id = Uuid::new_v4();

    let room_number_base = (Uuid::new_v4().as_u128() % 1_900_000_000) as i32 + 1;

    for (user_id, name) in [
        (user_a_id, "UserA"),
        (user_b_id, "UserB"),
        (user_c_id, "UserC"),
    ] {
        sqlx::query(
            r#"
            INSERT INTO users (user_id, auth_provider, provider_subject, display_name)
            VALUES (?, 'demo', ?, ?)
            "#,
        )
        .bind(user_id)
        .bind(format!("rooms-test-{user_id}"))
        .bind(name)
        .execute(&pool)
        .await
        .expect("user should be inserted");
    }

    // Insert rooms (reverse numbers to test ASC ordering: room2 has lower number than room1)
    sqlx::query(
        r#"
        INSERT INTO rooms (room_id, number, name, genre, description, is_published)
        VALUES (?, ?, 'Room 1', 'OSINT', 'Desc 1', 1)
        "#,
    )
    .bind(room1_id)
    .bind(room_number_base + 2)
    .execute(&pool)
    .await
    .expect("room1 should be inserted");

    sqlx::query(
        r#"
        INSERT INTO rooms (room_id, number, name, genre, description, is_published)
        VALUES (?, ?, 'Room 2', 'Web', 'Desc 2', 1)
        "#,
    )
    .bind(room2_id)
    .bind(room_number_base + 1)
    .execute(&pool)
    .await
    .expect("room2 should be inserted");

    sqlx::query(
        r#"
        INSERT INTO rooms (room_id, number, name, genre, description, is_published)
        VALUES (?, ?, 'Private Room', 'Web', 'Private Desc', 0)
        "#,
    )
    .bind(private_room_id)
    .bind(room_number_base + 3)
    .execute(&pool)
    .await
    .expect("private room should be inserted");

    // Insert problems for room2 (2 required, 1 optional)
    let p1_id = Uuid::new_v4();
    let p2_id = Uuid::new_v4();
    let p3_id = Uuid::new_v4();

    for (p_id, num, is_req, sub_type) in [
        (p1_id, 1, 1, "operation_sequence"),
        (p2_id, 2, 1, "string"),
        (p3_id, 3, 0, "operation_sequence"),
    ] {
        sqlx::query(
            r#"
            INSERT INTO problems (
                problem_id, room_id, number, problem_type, title, body_markdown,
                submission_type, assets, input_schema, hints, judge_config, is_required
            )
            VALUES (?, ?, ?, 'small', 'Title', 'Body', ?, '[]', '{"query":{"allowed_controls":[],"max_operations":10},"answer":{"max_length":50}}', '[]', '{}', ?)
            "#,
        )
        .bind(p_id)
        .bind(room2_id)
        .bind(num)
        .bind(sub_type)
        .bind(is_req)
        .execute(&pool)
        .await
        .expect("problem should be inserted");
    }

    // 1. Test unauthenticated (user_id = None)
    let unauth_rooms = repository
        .find_published_rooms_with_progress(None)
        .await
        .expect("find_published_rooms_with_progress should succeed for unauthenticated");

    let matching_rooms: Vec<_> = unauth_rooms
        .iter()
        .filter(|r| r.room_id == room1_id || r.room_id == room2_id || r.room_id == private_room_id)
        .collect();
    assert_eq!(matching_rooms.len(), 2);
    assert_eq!(matching_rooms[0].room_id, room2_id);
    assert_eq!(matching_rooms[0].problem_count, 3);
    assert_eq!(matching_rooms[0].required_count, 2);
    assert_eq!(matching_rooms[0].progress_status, "not_started");
    assert_eq!(matching_rooms[0].cleared_count, 0);
    assert!(matching_rooms[0].best_record.is_none());

    // 2. Test authenticated with no runs (User A)
    let user_a_rooms = repository
        .find_published_rooms_with_progress(Some(user_a_id))
        .await
        .expect("find_published_rooms_with_progress should succeed for User A");
    let room2_a = user_a_rooms.iter().find(|r| r.room_id == room2_id).unwrap();
    assert_eq!(room2_a.progress_status, "not_started");
    assert_eq!(room2_a.cleared_count, 0);
    assert_eq!(room2_a.required_count, 2);
    assert!(room2_a.best_record.is_none());

    // 3. Test active run on room2 with 1 required + 1 optional problem cleared
    let active_run_id = Uuid::new_v4();
    let started_at = Utc::now();
    sqlx::query(
        r#"
        INSERT INTO runs (run_id, user_id, room_id, status, started_at)
        VALUES (?, ?, ?, 'active', ?)
        "#,
    )
    .bind(active_run_id)
    .bind(user_a_id)
    .bind(room2_id)
    .bind(started_at)
    .execute(&pool)
    .await
    .expect("active run should be inserted");

    // Clear p1 (required) and p3 (optional)
    for (p_id, count) in [(p1_id, 3), (p3_id, 2)] {
        sqlx::query(
            r#"
            INSERT INTO problem_progress (run_id, problem_id, status, answer_attempt_count, cleared_at)
            VALUES (?, ?, 'cleared', ?, ?)
            "#,
        )
        .bind(active_run_id)
        .bind(p_id)
        .bind(count)
        .bind(started_at)
        .execute(&pool)
        .await
        .expect("problem progress should be inserted");
    }

    let user_a_active = repository
        .find_published_rooms_with_progress(Some(user_a_id))
        .await
        .expect("find_published_rooms_with_progress should succeed with active run");
    let room2_active = user_a_active
        .iter()
        .find(|r| r.room_id == room2_id)
        .unwrap();
    assert_eq!(room2_active.progress_status, "active");
    assert_eq!(room2_active.cleared_count, 1);
    assert_eq!(room2_active.required_count, 2);
    assert!(room2_active.best_record.is_none());

    // Clean up active run before testing cleared runs & rankings
    sqlx::query("DELETE FROM problem_progress WHERE run_id = ?")
        .bind(active_run_id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM runs WHERE run_id = ?")
        .bind(active_run_id)
        .execute(&pool)
        .await
        .unwrap();

    // 4. Test multiple cleared runs and rankings
    let base_time = DateTime::parse_from_rfc3339("2026-08-01T12:00:00Z")
        .unwrap()
        .with_timezone(&Utc);

    // Run A1: 50000ms
    let run_a1_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO runs (run_id, user_id, room_id, status, started_at, cleared_at) VALUES (?, ?, ?, 'cleared', ?, ?)",
    )
    .bind(run_a1_id)
    .bind(user_a_id)
    .bind(room2_id)
    .bind(base_time)
    .bind(base_time + chrono::Duration::milliseconds(50000))
    .execute(&pool)
    .await
    .unwrap();

    // Run A2: 40000ms, query count = 5
    let run_a2_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO runs (run_id, user_id, room_id, status, started_at, cleared_at) VALUES (?, ?, ?, 'cleared', ?, ?)",
    )
    .bind(run_a2_id)
    .bind(user_a_id)
    .bind(room2_id)
    .bind(base_time)
    .bind(base_time + chrono::Duration::milliseconds(40000))
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO problem_progress (run_id, problem_id, status, answer_attempt_count, cleared_at) VALUES (?, ?, 'cleared', 5, ?)",
    )
    .bind(run_a2_id)
    .bind(p1_id)
    .bind(base_time)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO problem_progress (run_id, problem_id, status, answer_attempt_count, cleared_at) VALUES (?, ?, 'cleared', 10, ?)",
    )
    .bind(run_a2_id)
    .bind(p2_id)
    .bind(base_time)
    .execute(&pool)
    .await
    .unwrap();

    // Run B1: 40000ms, query count = 3
    let run_b1_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO runs (run_id, user_id, room_id, status, started_at, cleared_at) VALUES (?, ?, ?, 'cleared', ?, ?)",
    )
    .bind(run_b1_id)
    .bind(user_b_id)
    .bind(room2_id)
    .bind(base_time)
    .bind(base_time + chrono::Duration::milliseconds(40000))
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO problem_progress (run_id, problem_id, status, answer_attempt_count, cleared_at) VALUES (?, ?, 'cleared', 3, ?)",
    )
    .bind(run_b1_id)
    .bind(p1_id)
    .bind(base_time)
    .execute(&pool)
    .await
    .unwrap();

    // Run C1: 40000ms, query count = 3, same cleared_at
    let run_c1_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO runs (run_id, user_id, room_id, status, started_at, cleared_at) VALUES (?, ?, ?, 'cleared', ?, ?)",
    )
    .bind(run_c1_id)
    .bind(user_c_id)
    .bind(room2_id)
    .bind(base_time)
    .bind(base_time + chrono::Duration::milliseconds(40000))
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO problem_progress (run_id, problem_id, status, answer_attempt_count, cleared_at) VALUES (?, ?, 'cleared', 3, ?)",
    )
    .bind(run_c1_id)
    .bind(p1_id)
    .bind(base_time)
    .execute(&pool)
    .await
    .unwrap();

    // Verify User A best record
    let user_a_cleared = repository
        .find_published_rooms_with_progress(Some(user_a_id))
        .await
        .unwrap();
    let room2_cleared_a = user_a_cleared
        .iter()
        .find(|r| r.room_id == room2_id)
        .unwrap();
    assert_eq!(room2_cleared_a.progress_status, "cleared");
    assert_eq!(room2_cleared_a.cleared_count, 2);
    let best_a = room2_cleared_a.best_record.as_ref().unwrap();
    assert_eq!(best_a.elapsed_ms, 40000);
    assert_eq!(best_a.query_count, 5);
    assert_eq!(best_a.rank, 3);

    // Verify User B best record
    let user_b_cleared = repository
        .find_published_rooms_with_progress(Some(user_b_id))
        .await
        .unwrap();
    let room2_cleared_b = user_b_cleared
        .iter()
        .find(|r| r.room_id == room2_id)
        .unwrap();
    let best_b = room2_cleared_b.best_record.as_ref().unwrap();
    assert_eq!(best_b.elapsed_ms, 40000);
    assert_eq!(best_b.query_count, 3);
    assert_eq!(best_b.rank, 1);

    // Verify User C best record
    let user_c_cleared = repository
        .find_published_rooms_with_progress(Some(user_c_id))
        .await
        .unwrap();
    let room2_cleared_c = user_c_cleared
        .iter()
        .find(|r| r.room_id == room2_id)
        .unwrap();
    let best_c = room2_cleared_c.best_record.as_ref().unwrap();
    assert_eq!(best_c.elapsed_ms, 40000);
    assert_eq!(best_c.query_count, 3);
    assert_eq!(best_c.rank, 1);

    // 5. Test active and cleared conflict error
    let conflict_active_run_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO runs (run_id, user_id, room_id, status, started_at) VALUES (?, ?, ?, 'active', ?)",
    )
    .bind(conflict_active_run_id)
    .bind(user_a_id)
    .bind(room2_id)
    .bind(Utc::now())
    .execute(&pool)
    .await
    .unwrap();

    let conflict_result = repository
        .find_published_rooms_with_progress(Some(user_a_id))
        .await;
    assert!(matches!(
        conflict_result,
        Err(RepositoryError::InvalidRunStatus { .. })
    ));

    // Cleanup
    sqlx::query("DELETE FROM problem_progress WHERE run_id IN (?, ?, ?, ?)")
        .bind(run_a1_id)
        .bind(run_a2_id)
        .bind(run_b1_id)
        .bind(run_c1_id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM runs WHERE room_id = ?")
        .bind(room2_id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM problems WHERE room_id = ?")
        .bind(room2_id)
        .execute(&pool)
        .await
        .unwrap();
    for r_id in [room1_id, room2_id, private_room_id] {
        sqlx::query("DELETE FROM rooms WHERE room_id = ?")
            .bind(r_id)
            .execute(&pool)
            .await
            .unwrap();
    }
    for u_id in [user_a_id, user_b_id, user_c_id] {
        sqlx::query("DELETE FROM users WHERE user_id = ?")
            .bind(u_id)
            .execute(&pool)
            .await
            .unwrap();
    }

    pool.close().await;
}
