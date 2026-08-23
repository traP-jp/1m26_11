use std::{env, path::PathBuf, str::FromStr};

use server::{
    migrate,
    problem::{load_problem_data, seed_problem_data},
};
use sqlx::{
    MySqlPool,
    mysql::{MySqlConnectOptions, MySqlPoolOptions},
};
use uuid::Uuid;

const ROOM_ID: &str = "1411824c-d357-4941-af76-c76cb827dda6";

const PROBLEM_IDS: [&str; 4] = [
    "52ed5a58-bc88-4e0f-97a4-0f64a112acd4",
    "9ebaa649-9c28-4bed-9dc1-fd7b9fedaa9b",
    "6853a228-0462-4413-91f4-6b8ef672cefc",
    "9ca65619-6ad2-4e74-bf4a-4f146b238067",
];

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL pointing to a disposable MariaDB database"]
async fn mock_problem_data_is_seeded_into_mariadb() {
    let pool = connect_test_database().await;

    migrate(&pool).await.expect("migration should succeed");

    let room_id = parse_uuid(ROOM_ID);

    cleanup_seeded_data(&pool, room_id).await;

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../mock-problem-data");

    let catalog = load_problem_data(root).expect("mock problem data should be valid");

    let summary = seed_problem_data(&pool, &catalog)
        .await
        .expect("problem data seeding should succeed");

    assert_eq!(summary.room_count, 1);
    assert_eq!(summary.problem_count, 4);

    let room = sqlx::query_as::<_, (i32, String, String, String, bool)>(
        r#"
        SELECT number, name, genre, description, is_published
        FROM rooms
        WHERE room_id = ?
        "#,
    )
    .bind(room_id)
    .fetch_one(&pool)
    .await
    .expect("seeded room should exist");

    assert_eq!(room.0, 1);
    assert_eq!(room.1, "最初の部屋");
    assert_eq!(room.2, "logic");
    assert_eq!(room.3, "動作確認用の問題セットです");
    assert!(!room.4);

    let problems = sqlx::query_as::<_, (Uuid, i32, String, String, Option<Uuid>, bool)>(
        r#"
            SELECT
                problem_id,
                number,
                problem_type,
                submission_type,
                depends_on_problem_id,
                is_required
            FROM problems
            WHERE room_id = ?
            ORDER BY number
            "#,
    )
    .bind(room_id)
    .fetch_all(&pool)
    .await
    .expect("seeded problems should be readable");

    assert_eq!(problems.len(), 4);

    assert_eq!(problems[0].0, parse_uuid(PROBLEM_IDS[0]));
    assert_eq!(problems[0].1, 1);
    assert_eq!(problems[0].2, "small");
    assert_eq!(problems[0].3, "operation_sequence");
    assert_eq!(problems[0].4, None);
    assert!(problems[0].5);

    assert_eq!(problems[1].0, parse_uuid(PROBLEM_IDS[1]));
    assert_eq!(problems[1].1, 2);
    assert_eq!(problems[1].2, "small");
    assert_eq!(problems[1].3, "string");
    assert_eq!(problems[1].4, Some(parse_uuid(PROBLEM_IDS[0])));
    assert!(problems[1].5);

    assert_eq!(problems[2].0, parse_uuid(PROBLEM_IDS[2]));
    assert_eq!(problems[2].1, 3);
    assert_eq!(problems[2].2, "small");
    assert_eq!(problems[2].3, "operation_sequence");
    assert_eq!(problems[2].4, Some(parse_uuid(PROBLEM_IDS[1])));
    assert!(problems[2].5);

    assert_eq!(problems[3].0, parse_uuid(PROBLEM_IDS[3]));
    assert_eq!(problems[3].1, 4);
    assert_eq!(problems[3].2, "final");
    assert_eq!(problems[3].3, "string");
    assert_eq!(problems[3].4, None);
    assert!(problems[3].5);

    let (assets, input_schema, hints, judge_config) = sqlx::query_as::<
        _,
        (
            serde_json::Value,
            serde_json::Value,
            serde_json::Value,
            serde_json::Value,
        ),
    >(
        r#"
            SELECT assets, input_schema, hints, judge_config
            FROM problems
            WHERE problem_id = ?
            "#,
    )
    .bind(parse_uuid(PROBLEM_IDS[0]))
    .fetch_one(&pool)
    .await
    .expect("seeded JSON columns should be readable");

    assert_eq!(
        assets,
        serde_json::json!([
            {
                "type": "image",
                "object_key":
                    "problems/1411824c-d357-4941-af76-c76cb827dda6/birthday.png",
                "alt": "生年月日の問題用画像"
            }
        ])
    );

    assert_eq!(input_schema["query"]["max_operations"], 100);
    assert!(hints.is_array());
    assert_eq!(judge_config["type"], "operation_sequence");

    cleanup_seeded_data(&pool, room_id).await;
    pool.close().await;
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

async fn cleanup_seeded_data(pool: &MySqlPool, room_id: Uuid) {
    sqlx::query("DELETE FROM runs WHERE room_id = ?")
        .bind(room_id)
        .execute(pool)
        .await
        .expect("seeded runs should be removable");

    for problem_id in PROBLEM_IDS.iter().rev() {
        sqlx::query("DELETE FROM problems WHERE problem_id = ?")
            .bind(parse_uuid(problem_id))
            .execute(pool)
            .await
            .expect("seeded problem should be removable");
    }

    sqlx::query("DELETE FROM rooms WHERE room_id = ?")
        .bind(room_id)
        .execute(pool)
        .await
        .expect("seeded room should be removable");
}

fn parse_uuid(value: &str) -> Uuid {
    Uuid::parse_str(value).expect("test UUID should be valid")
}
