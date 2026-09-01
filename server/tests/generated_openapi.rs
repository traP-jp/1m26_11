use openapi_generated::{
    NullValue,
    models::{
        ActiveRunResponse, CorrectQueryResponse, IncorrectQueryResponse, LeaderboardResponse,
        MeDemoUnauthenticated, Operation,
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
