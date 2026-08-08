use openapi_generated::{NullValue, models::MeLocalUnauthenticated};

const LOCAL_UNAUTHENTICATED: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../openapi/examples/auth/me-local-unauthenticated.json"
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
        serde_json::from_str(LOCAL_UNAUTHENTICATED).expect("fixture should be valid JSON");
    let model: MeLocalUnauthenticated =
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
    assert!(serde_json::from_value::<MeLocalUnauthenticated>(missing).is_err());

    let mut non_null = expected;
    non_null["user"] = serde_json::json!({});
    assert!(serde_json::from_value::<MeLocalUnauthenticated>(non_null).is_err());
}
