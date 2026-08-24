use std::{env, path::PathBuf, str::FromStr};

use chrono::{DateTime, Duration, TimeZone, Utc};
use sqlx::{
    MySqlPool,
    mysql::{MySqlConnectOptions, MySqlPoolOptions},
};
use uuid::Uuid;

use super::{
    AuthRepository, RepositoryError, SqlxUserRepository, apply_problem_clear_in_transaction,
};
use crate::{
    game_progress::{ProblemStatus, Progress, RunStatus},
    migrate,
    problem::{load_problem_data, seed_problem_data},
};

const ROOM_ID: &str = "1411824c-d357-4941-af76-c76cb827dda6";
const USER_ID: &str = "77777777-7777-4777-8777-777777777777";
const RUN_ID: &str = "88888888-8888-4888-8888-888888888888";

const PROBLEM_IDS: [&str; 4] = [
    "52ed5a58-bc88-4e0f-97a4-0f64a112acd4",
    "9ebaa649-9c28-4bed-9dc1-fd7b9fedaa9b",
    "6853a228-0462-4413-91f4-6b8ef672cefc",
    "9ca65619-6ad2-4e74-bf4a-4f146b238067",
];

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL pointing to a disposable MariaDB database"]
async fn mariadb_problem_clear_flow_is_transactional() {
    let pool = connect_test_database().await;

    migrate(&pool).await.expect("migration should succeed");
    cleanup_test_data(&pool).await;
    seed_room_and_problems(&pool).await;
    insert_test_user(&pool).await;

    let room_id = parse_uuid(ROOM_ID);
    let user_id = parse_uuid(USER_ID);
    let run_id = parse_uuid(RUN_ID);

    let first_problem_id = parse_uuid(PROBLEM_IDS[0]);
    let second_problem_id = parse_uuid(PROBLEM_IDS[1]);
    let third_problem_id = parse_uuid(PROBLEM_IDS[2]);
    let final_problem_id = parse_uuid(PROBLEM_IDS[3]);

    let repository = SqlxUserRepository::new(pool.clone());

    repository
        .create_run(run_id, user_id, room_id, test_time(0))
        .await
        .expect("active run should be created");

    assert_eq!(
        progress_statuses(&pool, run_id).await,
        vec!["available", "locked", "locked", "available"]
    );

    let mut transaction = pool.begin().await.expect("transaction should begin");

    let locked_error =
        apply_problem_clear_in_transaction(&mut transaction, run_id, second_problem_id)
            .await
            .expect_err("locked problem should be rejected");

    assert!(matches!(locked_error, RepositoryError::ProblemLocked));

    transaction
        .rollback()
        .await
        .expect("locked-problem transaction should roll back");

    let mut transaction = pool.begin().await.expect("transaction should begin");

    let rollback_plan =
        apply_problem_clear_in_transaction(&mut transaction, run_id, first_problem_id)
            .await
            .expect("first problem should be clearable");

    assert_eq!(rollback_plan.unlocked_problem_ids, vec![second_problem_id]);

    let forced_error = sqlx::query(
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
    .bind(first_problem_id)
    .execute(&mut *transaction)
    .await;

    assert!(
        forced_error.is_err(),
        "duplicate progress row should force a transaction error"
    );

    transaction
        .rollback()
        .await
        .expect("test transaction should roll back");

    assert_eq!(
        progress_statuses(&pool, run_id).await,
        vec!["available", "locked", "locked", "available"]
    );

    let mut blocking_transaction = pool
        .begin()
        .await
        .expect("blocking transaction should begin");

    sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT run_id
        FROM runs
        WHERE run_id = ?
        FOR UPDATE
        "#,
    )
    .bind(run_id)
    .fetch_one(&mut *blocking_transaction)
    .await
    .expect("blocking transaction should lock the active run");

    let worker_pool = pool.clone();
    let (worker_started_sender, worker_started_receiver) = tokio::sync::oneshot::channel();

    let mut waiting_clear_task = tokio::spawn(async move {
        let mut transaction = worker_pool
            .begin()
            .await
            .expect("waiting transaction should begin");

        worker_started_sender
            .send(())
            .expect("test should receive worker start notification");

        let plan = apply_problem_clear_in_transaction(&mut transaction, run_id, first_problem_id)
            .await
            .expect("first problem should be clearable after lock release");

        transaction
            .commit()
            .await
            .expect("first clear should commit");

        plan
    });

    worker_started_receiver
        .await
        .expect("waiting transaction should start");

    assert!(
        tokio::time::timeout(
            std::time::Duration::from_millis(100),
            &mut waiting_clear_task,
        )
        .await
        .is_err(),
        "clear transaction should wait for the run lock",
    );

    let immediately_before_lock_release = Utc::now();

    blocking_transaction
        .rollback()
        .await
        .expect("blocking transaction should release the lock");

    let first_plan = waiting_clear_task
        .await
        .expect("waiting clear task should finish");

    assert_eq!(first_plan.target_problem_status, ProblemStatus::Cleared);
    assert_eq!(first_plan.unlocked_problem_ids, vec![second_problem_id]);
    assert_eq!(
        first_plan.progress,
        Progress {
            cleared_problem_count: 1,
            total_problem_count: 4,
        }
    );
    assert_eq!(first_plan.run_status, RunStatus::Active);

    let first_plan_cleared_at = first_plan
        .problem_cleared_at
        .expect("newly cleared problem should receive cleared_at");

    assert!(
        first_plan_cleared_at >= immediately_before_lock_release,
        "cleared_at must be obtained after the FOR UPDATE lock is released",
    );
    assert!(first_plan.elapsed >= Duration::zero());

    assert_eq!(
        progress_statuses(&pool, run_id).await,
        vec!["cleared", "available", "locked", "available"]
    );

    let first_stored_cleared_at = problem_cleared_at(&pool, run_id, first_problem_id)
        .await
        .expect("first problem should have cleared_at");

    assert_eq!(
        first_stored_cleared_at.timestamp_millis(),
        first_plan_cleared_at.timestamp_millis()
    );

    let mut transaction = pool.begin().await.expect("transaction should begin");

    let repeated_plan =
        apply_problem_clear_in_transaction(&mut transaction, run_id, first_problem_id)
            .await
            .expect("already-cleared problem should be handled successfully");

    assert_eq!(repeated_plan.problem_cleared_at, None);
    assert!(repeated_plan.unlocked_problem_ids.is_empty());
    assert_eq!(
        repeated_plan.progress,
        Progress {
            cleared_problem_count: 1,
            total_problem_count: 4,
        }
    );
    assert_eq!(repeated_plan.run_status, RunStatus::Active);
    assert!(repeated_plan.elapsed >= first_plan.elapsed);

    transaction
        .commit()
        .await
        .expect("re-evaluation should commit");

    assert_eq!(
        problem_cleared_at(&pool, run_id, first_problem_id).await,
        Some(first_stored_cleared_at)
    );

    let mut transaction = pool.begin().await.expect("transaction should begin");

    let final_plan = apply_problem_clear_in_transaction(&mut transaction, run_id, final_problem_id)
        .await
        .expect("final problem should be clearable first");

    assert_eq!(final_plan.run_status, RunStatus::Active);
    assert_eq!(
        final_plan.progress,
        Progress {
            cleared_problem_count: 2,
            total_problem_count: 4,
        }
    );

    transaction
        .commit()
        .await
        .expect("final problem clear should commit");

    let mut transaction = pool.begin().await.expect("transaction should begin");

    let second_plan =
        apply_problem_clear_in_transaction(&mut transaction, run_id, second_problem_id)
            .await
            .expect("second problem should be clearable");

    assert_eq!(second_plan.unlocked_problem_ids, vec![third_problem_id]);
    assert_eq!(second_plan.run_status, RunStatus::Active);

    transaction
        .commit()
        .await
        .expect("second problem clear should commit");

    let mut transaction = pool.begin().await.expect("transaction should begin");

    let third_plan = apply_problem_clear_in_transaction(&mut transaction, run_id, third_problem_id)
        .await
        .expect("third problem should be clearable");

    assert_eq!(
        third_plan.progress,
        Progress {
            cleared_problem_count: 4,
            total_problem_count: 4,
        }
    );
    assert_eq!(third_plan.run_status, RunStatus::Cleared);
    assert_eq!(third_plan.run_cleared_at, third_plan.problem_cleared_at);

    let expected_run_cleared_at = third_plan
        .run_cleared_at
        .expect("completed run should receive cleared_at");

    transaction
        .commit()
        .await
        .expect("last required problem should commit");

    let (run_status, run_cleared_at) = run_status(&pool, run_id).await;

    assert_eq!(run_status, "cleared");

    let stored_run_cleared_at = run_cleared_at.expect("cleared run should have cleared_at");

    assert_eq!(
        stored_run_cleared_at.timestamp_millis(),
        expected_run_cleared_at.timestamp_millis()
    );

    let mut transaction = pool.begin().await.expect("transaction should begin");

    let cleared_run_error =
        apply_problem_clear_in_transaction(&mut transaction, run_id, third_problem_id)
            .await
            .expect_err("cleared run should no longer be active");

    assert!(matches!(cleared_run_error, RepositoryError::RunNotFound));

    transaction
        .rollback()
        .await
        .expect("not-found transaction should roll back");

    cleanup_test_data(&pool).await;
    pool.close().await;
}

async fn connect_test_database() -> MySqlPool {
    let database_url = env::var("TEST_DATABASE_URL")
        .expect("TEST_DATABASE_URL must point to a disposable test database");

    let options =
        MySqlConnectOptions::from_str(&database_url).expect("TEST_DATABASE_URL should be valid");

    MySqlPoolOptions::new()
        .max_connections(2)
        .connect_with(options)
        .await
        .expect("test database should be reachable")
}

async fn seed_room_and_problems(pool: &MySqlPool) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../mock-problem-data");

    let catalog = load_problem_data(root).expect("mock problem data should be valid");

    seed_problem_data(pool, &catalog)
        .await
        .expect("problem data should be seeded");
}

async fn insert_test_user(pool: &MySqlPool) {
    sqlx::query(
        r#"
        INSERT INTO users (
            user_id,
            auth_provider,
            provider_subject,
            display_name
        )
        VALUES (?, 'demo', 'game-progress-test', 'game-progress-test')
        "#,
    )
    .bind(parse_uuid(USER_ID))
    .execute(pool)
    .await
    .expect("test user should be inserted");
}

async fn progress_statuses(pool: &MySqlPool, run_id: Uuid) -> Vec<String> {
    sqlx::query_scalar::<_, String>(
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
    .fetch_all(pool)
    .await
    .expect("problem progress should be readable")
}

async fn problem_cleared_at(
    pool: &MySqlPool,
    run_id: Uuid,
    problem_id: Uuid,
) -> Option<DateTime<Utc>> {
    sqlx::query_scalar::<_, Option<DateTime<Utc>>>(
        r#"
        SELECT cleared_at
        FROM problem_progress
        WHERE run_id = ?
          AND problem_id = ?
        "#,
    )
    .bind(run_id)
    .bind(problem_id)
    .fetch_one(pool)
    .await
    .expect("problem cleared_at should be readable")
}

async fn run_status(pool: &MySqlPool, run_id: Uuid) -> (String, Option<DateTime<Utc>>) {
    sqlx::query_as::<_, (String, Option<DateTime<Utc>>)>(
        r#"
        SELECT status, cleared_at
        FROM runs
        WHERE run_id = ?
        "#,
    )
    .bind(run_id)
    .fetch_one(pool)
    .await
    .expect("run status should be readable")
}

async fn cleanup_test_data(pool: &MySqlPool) {
    let room_id = parse_uuid(ROOM_ID);

    sqlx::query("DELETE FROM runs WHERE room_id = ?")
        .bind(room_id)
        .execute(pool)
        .await
        .expect("test runs should be removable");

    sqlx::query("DELETE FROM users WHERE provider_subject = 'game-progress-test'")
        .execute(pool)
        .await
        .expect("test user should be removable");

    for problem_id in PROBLEM_IDS.iter().rev() {
        sqlx::query("DELETE FROM problems WHERE problem_id = ?")
            .bind(parse_uuid(problem_id))
            .execute(pool)
            .await
            .expect("test problem should be removable");
    }

    sqlx::query("DELETE FROM rooms WHERE room_id = ?")
        .bind(room_id)
        .execute(pool)
        .await
        .expect("test room should be removable");
}

fn test_time(seconds_after_start: i64) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, 24, 10, 0, 0)
        .single()
        .expect("test time should be valid")
        + Duration::seconds(seconds_after_start)
}

fn parse_uuid(value: &str) -> Uuid {
    Uuid::parse_str(value).expect("test UUID should be valid")
}
