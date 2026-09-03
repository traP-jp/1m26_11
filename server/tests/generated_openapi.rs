use openapi_generated::{
    NullValue,
    models::{
        ActiveRunResponse, Asset, CorrectQueryResponse, IncorrectQueryResponse,
        LeaderboardResponse, MeDemoUnauthenticated, MeProgressResponse, Operation, RoomResponse,
        RoomRunStatus, UploadProblemAssetHeaderParams, UploadProblemAssetPathParams,
    },
    types::Nullable,
};
use uuid::Uuid;

const DEMO_UNAUTHENTICATED: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../openapi/examples/auth/me-demo-unauthenticated.json"
));

const LEADERBOARD_RANKED: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../openapi/examples/leaderboard/response-ranked.json"
));

const LEADERBOARD_UNAUTHENTICATED: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../openapi/examples/leaderboard/response-unauthenticated.json"
));

const LEADERBOARD_EMPTY: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../openapi/examples/leaderboard/response-empty.json"
));

const ROOM_ACTIVE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../openapi/examples/rooms/response-active.json"
));

const ROOM_NOT_STARTED: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../openapi/examples/rooms/response-not-started.json"
));

const ROOM_CLEARED: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../openapi/examples/rooms/response-cleared.json"
));

const ME_PROGRESS_SUMMARY: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../openapi/examples/progress/response-summary.json"
));

const ME_PROGRESS_EMPTY: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../openapi/examples/progress/response-empty.json"
));

const PROBLEM_ASSET_CREATED: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../openapi/examples/assets/response-created.json"
));

#[test]
fn generated_null_value_accepts_only_json_null() {
    assert_eq!(
        serde_json::to_value(NullValue).expect("NullValue should serialize"),
        serde_json::Value::Null
    );
    assert_eq!(
        serde_json::from_value::<NullValue>(serde_json::Value::Null)
            .expect("JSON null should deserialize"),
        NullValue
    );

    for value in [
        serde_json::json!(false),
        serde_json::json!(0),
        serde_json::json!("null"),
        serde_json::json!([]),
        serde_json::json!({}),
    ] {
        assert!(serde_json::from_value::<NullValue>(value).is_err());
    }
}

#[test]
fn generated_required_null_fields_match_the_fixture() {
    let expected: serde_json::Value =
        serde_json::from_str(DEMO_UNAUTHENTICATED).expect("fixture should be valid JSON");
    let model: MeDemoUnauthenticated =
        serde_json::from_value(expected.clone()).expect("fixture should match generated model");
    assert_eq!(
        serde_json::to_value(model).expect("generated model should serialize"),
        expected
    );

    let mut missing = expected.clone();
    missing
        .as_object_mut()
        .expect("fixture should be an object")
        .remove("user");
    assert!(serde_json::from_value::<MeDemoUnauthenticated>(missing).is_err());

    let mut non_null = expected;
    non_null["user"] = serde_json::json!({});
    assert!(serde_json::from_value::<MeDemoUnauthenticated>(non_null).is_err());
}

#[test]
fn generated_leaderboard_responses_match_fixtures() {
    for fixture in [
        LEADERBOARD_RANKED,
        LEADERBOARD_UNAUTHENTICATED,
        LEADERBOARD_EMPTY,
    ] {
        let expected: serde_json::Value =
            serde_json::from_str(fixture).expect("leaderboard fixture should be valid JSON");

        let model: LeaderboardResponse = serde_json::from_value(expected.clone())
            .expect("leaderboard fixture should match generated model");

        assert_eq!(
            serde_json::to_value(model).expect("generated leaderboard model should serialize"),
            expected
        );
    }

    let ranked: LeaderboardResponse = serde_json::from_str(LEADERBOARD_RANKED)
        .expect("ranked fixture should match generated model");

    assert!(
        matches!(ranked.me, Nullable::Present(_)),
        "authenticated fixture should contain me"
    );
    assert_eq!(
        ranked
            .entries
            .iter()
            .map(|entry| entry.rank)
            .collect::<Vec<_>>(),
        vec![1, 1, 3]
    );

    let unauthenticated: LeaderboardResponse = serde_json::from_str(LEADERBOARD_UNAUTHENTICATED)
        .expect("unauthenticated fixture should match generated model");

    assert!(
        matches!(unauthenticated.me, Nullable::Null),
        "unauthenticated fixture should contain an explicit null me"
    );

    let mut missing_me: serde_json::Value =
        serde_json::from_str(LEADERBOARD_EMPTY).expect("empty fixture should be valid JSON");

    missing_me
        .as_object_mut()
        .expect("leaderboard fixture should be an object")
        .remove("me");

    assert!(
        serde_json::from_value::<LeaderboardResponse>(missing_me).is_err(),
        "me must be required even though it accepts null"
    );
}

#[test]
fn generated_me_progress_responses_match_fixtures() {
    for fixture in [ME_PROGRESS_SUMMARY, ME_PROGRESS_EMPTY] {
        let expected: serde_json::Value =
            serde_json::from_str(fixture).expect("progress fixture should be valid JSON");

        let model: MeProgressResponse = serde_json::from_value(expected.clone())
            .expect("progress fixture should match generated model");

        assert_eq!(
            serde_json::to_value(model).expect("generated progress model should serialize"),
            expected
        );
    }

    let summary: MeProgressResponse = serde_json::from_str(ME_PROGRESS_SUMMARY)
        .expect("summary fixture should match generated model");

    assert_eq!(summary.cleared_room_count, 5);
    assert_eq!(summary.total_room_count, 20);

    assert_eq!(
        summary
            .by_genre
            .iter()
            .map(|progress| progress.genre.as_str())
            .collect::<Vec<_>>(),
        vec!["OSINT", "Web"]
    );

    assert_eq!(
        summary
            .by_genre
            .iter()
            .map(|progress| progress.cleared_room_count)
            .sum::<u32>(),
        summary.cleared_room_count
    );

    assert_eq!(
        summary
            .by_genre
            .iter()
            .map(|progress| progress.total_room_count)
            .sum::<u32>(),
        summary.total_room_count
    );

    let empty: MeProgressResponse = serde_json::from_str(ME_PROGRESS_EMPTY)
        .expect("empty fixture should match generated model");

    assert_eq!(empty.cleared_room_count, 0);
    assert_eq!(empty.total_room_count, 0);
    assert!(empty.by_genre.is_empty());

    let mut negative_total: serde_json::Value =
        serde_json::from_str(ME_PROGRESS_SUMMARY).expect("summary fixture should be valid JSON");

    negative_total["total_room_count"] = serde_json::json!(-1);

    assert!(
        serde_json::from_value::<MeProgressResponse>(negative_total).is_err(),
        "negative total_room_count must not deserialize into the generated u32 field"
    );

    let mut negative_genre_count: serde_json::Value =
        serde_json::from_str(ME_PROGRESS_SUMMARY).expect("summary fixture should be valid JSON");

    negative_genre_count["by_genre"][0]["cleared_room_count"] = serde_json::json!(-1);

    assert!(
        serde_json::from_value::<MeProgressResponse>(negative_genre_count).is_err(),
        "negative genre count must not deserialize into the generated u32 field"
    );
}

#[test]
fn generated_problem_asset_upload_contract_matches_fixture() {
    let expected: serde_json::Value =
        serde_json::from_str(PROBLEM_ASSET_CREATED).expect("asset fixture should be valid JSON");

    let model: Asset = serde_json::from_value(expected.clone())
        .expect("asset fixture should match generated model");

    assert_eq!(
        serde_json::to_value(model).expect("generated asset model should serialize"),
        expected
    );

    assert!(
        expected.get("object_key").is_none(),
        "public asset response must not expose object_key"
    );

    let idempotency_key = Uuid::parse_str("44444444-4444-4444-8444-444444444444").unwrap();

    let header = UploadProblemAssetHeaderParams { idempotency_key };
    assert_eq!(header.idempotency_key, idempotency_key);

    let room_id = Uuid::parse_str("11111111-1111-4111-8111-111111111111").unwrap();
    let problem_id = Uuid::parse_str("22222222-2222-4222-8222-222222222221").unwrap();

    let path = UploadProblemAssetPathParams {
        room_id,
        problem_id,
    };

    assert_eq!(path.room_id, room_id);
    assert_eq!(path.problem_id, problem_id);
}

#[test]
fn generated_run_and_query_numeric_types_match_contract() {
    let started_at = chrono::DateTime::parse_from_rfc3339("2026-08-06T10:00:00.000Z")
        .expect("fixture timestamp should be valid")
        .with_timezone(&chrono::Utc);

    let elapsed_ms: u64 = 65_000;
    let active_run =
        ActiveRunResponse::new("active".to_owned(), started_at, elapsed_ms, Vec::new());

    let active_run_json = serde_json::to_value(&active_run).expect("active run should serialize");

    assert_eq!(active_run_json["elapsed_ms"], elapsed_ms);
    assert!(
        active_run_json.get("query_count").is_none(),
        "active run response must not contain query_count",
    );

    let mut negative_elapsed = active_run_json;
    negative_elapsed["elapsed_ms"] = serde_json::json!(-1);

    assert!(
        serde_json::from_value::<ActiveRunResponse>(negative_elapsed).is_err(),
        "negative elapsed_ms must not deserialize into the generated u64 field",
    );

    let positive_query_count: u64 = 2;
    let correct_query = CorrectQueryResponse::new(
        Uuid::new_v4(),
        true,
        vec![Operation::new("down".to_owned(), 1)],
        1,
        positive_query_count,
        "cleared".to_owned(),
    );

    let correct_query_json =
        serde_json::to_value(&correct_query).expect("correct query response should serialize");

    assert_eq!(correct_query_json["query_count"], positive_query_count);

    let zero_query_count: u64 = 0;
    let incorrect_query = IncorrectQueryResponse::new(
        Uuid::new_v4(),
        false,
        vec![Operation::new("down".to_owned(), 1)],
        2,
        zero_query_count,
        "available".to_owned(),
    );

    let incorrect_query_json =
        serde_json::to_value(&incorrect_query).expect("incorrect query response should serialize");

    assert_eq!(incorrect_query_json["query_count"], 0);

    let mut negative_query_count = correct_query_json;
    negative_query_count["query_count"] = serde_json::json!(-1);

    assert!(
        serde_json::from_value::<CorrectQueryResponse>(negative_query_count).is_err(),
        "negative query_count must not deserialize into the generated u64 field",
    );
}

#[test]
fn generated_room_responses_match_fixtures() {
    for fixture in [ROOM_ACTIVE, ROOM_NOT_STARTED, ROOM_CLEARED] {
        let expected: serde_json::Value =
            serde_json::from_str(fixture).expect("room fixture should be valid JSON");

        let model: RoomResponse = serde_json::from_value(expected.clone())
            .expect("room fixture should match generated model");

        assert_eq!(
            serde_json::to_value(model).expect("generated room model should serialize"),
            expected
        );
    }

    let active: RoomResponse =
        serde_json::from_str(ROOM_ACTIVE).expect("active fixture should match generated model");
    assert_eq!(active.run_status, RoomRunStatus::Active);
    assert_eq!(active.ranking_summary.player_count, 84);
    assert_eq!(active.ranking_summary.my_rank, Nullable::Present(14));

    let not_started: RoomResponse = serde_json::from_str(ROOM_NOT_STARTED)
        .expect("not_started fixture should match generated model");
    assert_eq!(not_started.run_status, RoomRunStatus::NotStarted);
    assert_eq!(not_started.ranking_summary.my_rank, Nullable::Null);

    let cleared: RoomResponse =
        serde_json::from_str(ROOM_CLEARED).expect("cleared fixture should match generated model");
    assert_eq!(cleared.run_status, RoomRunStatus::Cleared);
    assert_eq!(cleared.ranking_summary.my_rank, Nullable::Present(14));

    let mut missing_my_rank: serde_json::Value =
        serde_json::from_str(ROOM_NOT_STARTED).expect("fixture should be valid JSON");
    missing_my_rank
        .as_object_mut()
        .unwrap()
        .get_mut("ranking_summary")
        .unwrap()
        .as_object_mut()
        .unwrap()
        .remove("my_rank");

    assert!(
        serde_json::from_value::<RoomResponse>(missing_my_rank).is_err(),
        "my_rank must be required in ranking_summary even though it accepts null"
    );
}
