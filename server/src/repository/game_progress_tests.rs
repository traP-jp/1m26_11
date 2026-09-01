use std::{env, path::PathBuf, str::FromStr};

use chrono::{DateTime, Duration, TimeZone, Utc};
use sqlx::{
    MySqlPool,
    mysql::{MySqlConnectOptions, MySqlPoolOptions},
    types::Json,
};
use uuid::Uuid;

use super::{
    AnswerRunStatus, AnswerSubmission, AnswerSubmissionResult, AuthRepository, QuerySubmission,
    RepositoryError, SqlxUserRepository, apply_problem_clear_in_transaction,
};
use crate::{
    game_progress::{ProblemStatus, Progress, RunStatus},
    migrate,
    problem::{Operation, load_problem_data, seed_problem_data},
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
async fn mariadb_concurrent_run_creation_returns_existing_active_run() {
    let pool = connect_test_database().await;

    migrate(&pool).await.expect("migration should succeed");
    cleanup_test_data(&pool).await;
    seed_room_and_problems(&pool).await;
    insert_test_user(&pool).await;

    let repository = SqlxUserRepository::new(pool.clone());
    let user_id = parse_uuid(USER_ID);
    let room_id = parse_uuid(ROOM_ID);
    let first_run_id = Uuid::new_v4();
    let second_run_id = Uuid::new_v4();

    let (first_result, second_result) = tokio::join!(
        repository.create_run(first_run_id, user_id, room_id, test_time(0)),
        repository.create_run(second_run_id, user_id, room_id, test_time(1)),
    );

    let first_run = first_result.expect("first run request should succeed");
    let second_run = second_result.expect("second run request should succeed");

    assert_eq!(
        first_run.id, second_run.id,
        "both requests should return the same active run"
    );
    assert_eq!(
        first_run.started_at, second_run.started_at,
        "the resumed response should preserve the original started_at"
    );
    assert!(
        first_run.id == first_run_id || first_run.id == second_run_id,
        "one of the requested run IDs should become the active run"
    );

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

    assert_eq!(active_run_count, 1, "only one active run should be stored");

    let progress_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM problem_progress
        WHERE run_id = ?
        "#,
    )
    .bind(first_run.id)
    .fetch_one(&pool)
    .await
    .expect("problem progress count should be readable");

    assert_eq!(
        progress_count,
        PROBLEM_IDS.len() as i64,
        "problem progress should be created exactly once"
    );

    cleanup_test_data(&pool).await;
    pool.close().await;
}

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

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL pointing to a disposable MariaDB database"]
async fn mariadb_query_judgement_is_recorded_transactionally() {
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

    let repository = SqlxUserRepository::new(pool.clone());

    repository
        .create_run(run_id, user_id, room_id, test_time(0))
        .await
        .expect("active run should be created");

    let locked_error = repository
        .record_query_judgement(query_submission(
            Uuid::new_v4(),
            run_id,
            second_problem_id,
            false,
        ))
        .await
        .expect_err("locked problem should reject query");

    assert!(matches!(locked_error, RepositoryError::ProblemLocked));

    assert_eq!(
        stored_query_count(&pool, run_id, second_problem_id).await,
        0
    );
    assert_eq!(
        stored_answer_attempt_count(&pool, run_id, second_problem_id).await,
        0
    );

    let first_query_id = Uuid::new_v4();

    let first_result = repository
        .record_query_judgement(query_submission(
            first_query_id,
            run_id,
            first_problem_id,
            false,
        ))
        .await
        .expect("incorrect query should be recorded");

    assert_eq!(first_result.query_count, 1);
    assert_eq!(first_result.problem_status, "available");
    assert_eq!(
        stored_answer_attempt_count(&pool, run_id, first_problem_id).await,
        1
    );

    let stored = sqlx::query_as::<
        _,
        (
            String,
            Json<Vec<Operation>>,
            Json<Vec<Operation>>,
            i32,
            bool,
        ),
    >(
        r#"
        SELECT
            source,
            operations,
            normalized_operations,
            remaining_pattern_count,
            is_correct
        FROM queries
        WHERE query_id = ?
        "#,
    )
    .bind(first_query_id)
    .fetch_one(&pool)
    .await
    .expect("stored query should be readable");

    assert_eq!(stored.0, "serial");
    assert!(stored.1.0 == vec![operation("down", 1), operation("down", 1),]);
    assert!(stored.2.0 == vec![operation("down", 2)]);
    assert_eq!(stored.3, 2);
    assert!(!stored.4);

    let second_result = repository
        .record_query_judgement(query_submission(
            Uuid::new_v4(),
            run_id,
            first_problem_id,
            false,
        ))
        .await
        .expect("second incorrect query should be recorded");

    assert_eq!(second_result.query_count, 2);
    assert_eq!(second_result.problem_status, "available");
    assert_eq!(
        stored_answer_attempt_count(&pool, run_id, first_problem_id).await,
        2
    );

    let correct_result = repository
        .record_query_judgement(query_submission(
            Uuid::new_v4(),
            run_id,
            first_problem_id,
            true,
        ))
        .await
        .expect("correct query should be recorded");

    assert_eq!(correct_result.query_count, 3);
    assert_eq!(correct_result.problem_status, "cleared");
    assert_eq!(
        stored_answer_attempt_count(&pool, run_id, first_problem_id).await,
        3
    );

    assert_eq!(
        progress_statuses(&pool, run_id).await,
        vec!["cleared", "available", "locked", "available"]
    );

    assert_eq!(stored_query_count(&pool, run_id, first_problem_id).await, 3);
    assert_eq!(
        stored_answer_attempt_count(&pool, run_id, first_problem_id).await,
        3
    );

    let cleared_error = repository
        .record_query_judgement(query_submission(
            Uuid::new_v4(),
            run_id,
            first_problem_id,
            false,
        ))
        .await
        .expect_err("cleared problem should reject query");

    assert!(matches!(
        cleared_error,
        RepositoryError::ProblemAlreadyCleared
    ));

    assert_eq!(stored_query_count(&pool, run_id, first_problem_id).await, 3);
    assert_eq!(
        stored_answer_attempt_count(&pool, run_id, first_problem_id).await,
        3,
        "rejected query must not increment the shared attempt counter"
    );

    cleanup_test_data(&pool).await;
    pool.close().await;
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL pointing to a disposable MariaDB database"]
async fn mariadb_answer_judgement_updates_counter_transactionally() {
    let pool = connect_test_database().await;

    migrate(&pool).await.expect("migration should succeed");
    cleanup_test_data(&pool).await;
    seed_room_and_problems(&pool).await;
    insert_test_user(&pool).await;

    let room_id = parse_uuid(ROOM_ID);
    let user_id = parse_uuid(USER_ID);
    let run_id = parse_uuid(RUN_ID);

    let operation_problem_id = parse_uuid(PROBLEM_IDS[0]);
    let locked_string_problem_id = parse_uuid(PROBLEM_IDS[1]);
    let final_problem_id = parse_uuid(PROBLEM_IDS[3]);

    let repository = SqlxUserRepository::new(pool.clone());

    repository
        .create_run(run_id, user_id, room_id, test_time(0))
        .await
        .expect("active run should be created");

    let locked_error = repository
        .record_answer_judgement(answer_submission(
            run_id,
            locked_string_problem_id,
            "かおもじくん",
        ))
        .await
        .expect_err("locked problem should reject answer");

    assert!(matches!(locked_error, RepositoryError::ProblemLocked));
    assert_eq!(
        stored_answer_attempt_count(&pool, run_id, locked_string_problem_id).await,
        0
    );

    let wrong_type_error = repository
        .record_answer_judgement(answer_submission(run_id, operation_problem_id, "answer"))
        .await
        .expect_err("operation sequence problem should reject string answer");

    assert!(matches!(
        wrong_type_error,
        RepositoryError::WrongAnswerSubmissionType
    ));
    assert_eq!(
        stored_answer_attempt_count(&pool, run_id, operation_problem_id).await,
        0
    );

    let too_long_error = repository
        .record_answer_judgement(answer_submission(run_id, final_problem_id, &"x".repeat(51)))
        .await
        .expect_err("answer over max length should be rejected");

    assert!(matches!(
        too_long_error,
        RepositoryError::AnswerLengthExceeded
    ));
    assert_eq!(
        stored_answer_attempt_count(&pool, run_id, final_problem_id).await,
        0
    );

    let first_incorrect = repository
        .record_answer_judgement(answer_submission(run_id, final_problem_id, "incorrect"))
        .await
        .expect("incorrect answer should be recorded");

    assert_eq!(
        first_incorrect,
        AnswerSubmissionResult::Incorrect {
            answer_attempt_count: 1,
        }
    );

    let second_incorrect = repository
        .record_answer_judgement(answer_submission(
            run_id,
            final_problem_id,
            "still incorrect",
        ))
        .await
        .expect("second incorrect answer should be recorded");

    assert_eq!(
        second_incorrect,
        AnswerSubmissionResult::Incorrect {
            answer_attempt_count: 2,
        }
    );

    assert_eq!(
        stored_answer_attempt_count(&pool, run_id, final_problem_id).await,
        2
    );

    let correct = repository
        .record_answer_judgement(answer_submission(
            run_id,
            final_problem_id,
            "  ワンマンソン  ",
        ))
        .await
        .expect("normalized correct answer should clear the problem");

    match correct {
        AnswerSubmissionResult::Correct {
            unlocked_problem_ids,
            run_status,
            cleared_problem_count,
            total_problem_count,
            elapsed_ms,
        } => {
            assert!(unlocked_problem_ids.is_empty());
            assert_eq!(run_status, AnswerRunStatus::Active);
            assert_eq!(cleared_problem_count, 1);
            assert_eq!(total_problem_count, 4);
            assert!(elapsed_ms > 0);
        }
        AnswerSubmissionResult::Incorrect { .. } => {
            panic!("correct answer should return the correct result");
        }
    }

    assert_eq!(
        stored_answer_attempt_count(&pool, run_id, final_problem_id).await,
        2,
        "correct answer must not increment the counter"
    );

    assert_eq!(
        progress_statuses(&pool, run_id).await,
        vec!["available", "locked", "locked", "cleared"]
    );

    let (stored_run_status, stored_run_cleared_at) = run_status(&pool, run_id).await;

    assert_eq!(stored_run_status, "active");
    assert_eq!(stored_run_cleared_at, None);

    let repeated_error = repository
        .record_answer_judgement(answer_submission(run_id, final_problem_id, "ワンマンソン"))
        .await
        .expect_err("already cleared problem should reject repeated answer");

    assert!(matches!(
        repeated_error,
        RepositoryError::ProblemAlreadyCleared
    ));

    assert_eq!(
        stored_answer_attempt_count(&pool, run_id, final_problem_id).await,
        2
    );

    cleanup_test_data(&pool).await;
    pool.close().await;
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL pointing to a disposable MariaDB database"]
async fn mariadb_correct_answer_can_complete_run() {
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

    repository
        .record_query_judgement(query_submission(
            Uuid::new_v4(),
            run_id,
            first_problem_id,
            true,
        ))
        .await
        .expect("first operation problem should be cleared");

    let second_result = repository
        .record_answer_judgement(answer_submission(run_id, second_problem_id, "顔文字くん"))
        .await
        .expect("second string problem should be cleared");

    match second_result {
        AnswerSubmissionResult::Correct {
            unlocked_problem_ids,
            run_status,
            cleared_problem_count,
            total_problem_count,
            ..
        } => {
            assert_eq!(unlocked_problem_ids, vec![third_problem_id]);
            assert_eq!(run_status, AnswerRunStatus::Active);
            assert_eq!(cleared_problem_count, 2);
            assert_eq!(total_problem_count, 4);
        }
        AnswerSubmissionResult::Incorrect { .. } => {
            panic!("accepted answer should be correct");
        }
    }

    assert_eq!(
        stored_answer_attempt_count(&pool, run_id, second_problem_id).await,
        0,
        "correct answer must not increment the counter"
    );

    repository
        .record_query_judgement(query_submission(
            Uuid::new_v4(),
            run_id,
            third_problem_id,
            true,
        ))
        .await
        .expect("third operation problem should be cleared");

    let final_result = repository
        .record_answer_judgement(answer_submission(run_id, final_problem_id, "ワンマンソン"))
        .await
        .expect("last required answer should clear the run");

    match final_result {
        AnswerSubmissionResult::Correct {
            unlocked_problem_ids,
            run_status,
            cleared_problem_count,
            total_problem_count,
            elapsed_ms,
        } => {
            assert!(unlocked_problem_ids.is_empty());
            assert_eq!(run_status, AnswerRunStatus::Cleared);
            assert_eq!(cleared_problem_count, 4);
            assert_eq!(total_problem_count, 4);
            assert!(elapsed_ms > 0);
        }
        AnswerSubmissionResult::Incorrect { .. } => {
            panic!("accepted final answer should be correct");
        }
    }

    assert_eq!(
        progress_statuses(&pool, run_id).await,
        vec!["cleared", "cleared", "cleared", "cleared"]
    );

    assert_eq!(
        stored_answer_attempt_count(&pool, run_id, final_problem_id).await,
        0
    );

    let (stored_run_status, stored_run_cleared_at) = run_status(&pool, run_id).await;

    assert_eq!(stored_run_status, "cleared");
    assert!(
        stored_run_cleared_at.is_some(),
        "cleared run should have cleared_at"
    );

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

fn operation(control: &str, count: i32) -> Operation {
    Operation {
        control: control.to_owned(),
        count,
    }
}

fn query_submission(
    query_id: Uuid,
    run_id: Uuid,
    problem_id: Uuid,
    is_correct: bool,
) -> QuerySubmission {
    let (operations, normalized_operations, remaining_pattern_count) = if is_correct {
        (
            vec![operation("down", 2), operation("right", 1)],
            vec![operation("down", 2), operation("right", 1)],
            1,
        )
    } else {
        (
            vec![operation("down", 1), operation("down", 1)],
            vec![operation("down", 2)],
            2,
        )
    };

    QuerySubmission {
        query_id,
        run_id,
        problem_id,
        source: "serial".to_owned(),
        operations,
        normalized_operations,
        remaining_pattern_count,
        is_correct,
    }
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
    .expect("query count should be readable")
}

fn parse_uuid(value: &str) -> Uuid {
    Uuid::parse_str(value).expect("test UUID should be valid")
}

fn answer_submission(run_id: Uuid, problem_id: Uuid, answer: &str) -> AnswerSubmission {
    AnswerSubmission {
        run_id,
        problem_id,
        answer: answer.to_owned(),
    }
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
    .expect("answer attempt count should be readable")
}
