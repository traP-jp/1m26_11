use std::{
    fs,
    path::{Path, PathBuf},
};

use server::problem::{
    JudgeConfig, ProblemDataError, ProblemType, SubmissionType, load_problem_data,
};

use uuid::Uuid;

struct TestProblemData {
    root: PathBuf,
}

impl TestProblemData {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("problem-loader-test-{}", Uuid::new_v4()));

        fs::create_dir_all(root.join("rooms"))
            .expect("temporary rooms directory should be created");

        Self { root }
    }

    fn root(&self) -> &Path {
        &self.root
    }

    fn write_room_json(&self, directory_name: &str, value: &serde_json::Value) {
        let room_directory = self.root.join("rooms").join(directory_name);

        fs::create_dir_all(&room_directory).expect("temporary room directory should be created");

        let json =
            serde_json::to_string_pretty(value).expect("test room JSON should be serializable");

        fs::write(room_directory.join("room.json"), json)
            .expect("test room JSON should be written");
    }

    fn write_raw_room_json(&self, directory_name: &str, json: &str) {
        let room_directory = self.root.join("rooms").join(directory_name);

        fs::create_dir_all(&room_directory).expect("temporary room directory should be created");

        fs::write(room_directory.join("room.json"), json)
            .expect("test room JSON should be written");
    }
}

impl Drop for TestProblemData {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn mock_problem_data_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../mock-problem-data")
}

fn source_room_json() -> serde_json::Value {
    let file_path = mock_problem_data_root()
        .join("rooms")
        .join("1411824c-d357-4941-af76-c76cb827dda6")
        .join("room.json");

    let json = fs::read_to_string(file_path).expect("source room JSON should be readable");

    serde_json::from_str(&json).expect("source room JSON should be valid")
}

fn source_room_id(value: &serde_json::Value) -> &str {
    value["room"]["room_id"]
        .as_str()
        .expect("source room ID should be a string")
}

#[test]
fn valid_problem_data_loads() {
    let root = mock_problem_data_root();

    let catalog = load_problem_data(root).expect("problem data should be valid");

    assert_eq!(catalog.rooms.len(), 1);

    let room = &catalog.rooms[0];

    assert_eq!(
        room.room_id,
        Uuid::parse_str("1411824c-d357-4941-af76-c76cb827dda6").unwrap()
    );
    assert_eq!(room.problems.len(), 4);

    assert_eq!(
        room.problems
            .iter()
            .filter(|problem| problem.problem_type == ProblemType::Small)
            .count(),
        3
    );

    assert_eq!(
        room.problems
            .iter()
            .filter(|problem| problem.problem_type == ProblemType::Final)
            .count(),
        1
    );

    assert!(room.problems.iter().any(|problem| {
        problem.submission_type == SubmissionType::OperationSequence
            && matches!(&problem.judge_config, JudgeConfig::OperationSequence { .. })
    }));

    assert!(room.problems.iter().any(|problem| {
        problem.submission_type == SubmissionType::String
            && matches!(&problem.judge_config, JudgeConfig::String { .. })
    }));
}

#[test]
fn malformed_json_is_rejected() {
    let test_data = TestProblemData::new();
    let room_id = "1411824c-d357-4941-af76-c76cb827dda6";

    test_data.write_raw_room_json(room_id, "{ invalid json");

    let error = load_problem_data(test_data.root()).expect_err("malformed JSON should be rejected");

    match error {
        ProblemDataError::Json { path, .. } => {
            assert!(path.ends_with("room.json"));
        }
        other => panic!("unexpected error: {other}"),
    }
}

#[test]
fn unknown_json_field_is_rejected() {
    let test_data = TestProblemData::new();
    let mut value = source_room_json();
    let room_id = source_room_id(&value).to_owned();

    value
        .as_object_mut()
        .expect("root JSON should be an object")
        .insert("unexpected".to_owned(), serde_json::json!(true));

    test_data.write_room_json(&room_id, &value);

    let error = load_problem_data(test_data.root()).expect_err("unknown field should be rejected");

    assert!(matches!(error, ProblemDataError::Json { .. }));
    assert!(error.to_string().contains("room.json"));
}

#[test]
fn missing_required_json_field_is_rejected() {
    let test_data = TestProblemData::new();
    let mut value = source_room_json();
    let room_id = source_room_id(&value).to_owned();

    value["room"]
        .as_object_mut()
        .expect("room should be an object")
        .remove("name");

    test_data.write_room_json(&room_id, &value);

    let error =
        load_problem_data(test_data.root()).expect_err("missing required field should be rejected");

    assert!(matches!(error, ProblemDataError::Json { .. }));
}

#[test]
fn room_id_must_match_directory_name() {
    let test_data = TestProblemData::new();
    let value = source_room_json();
    let different_directory_id = Uuid::new_v4().to_string();

    test_data.write_room_json(&different_directory_id, &value);

    let error =
        load_problem_data(test_data.root()).expect_err("directory ID mismatch should be rejected");

    match error {
        ProblemDataError::Validation { field, message } => {
            assert_eq!(field, "room.room_id");
            assert_eq!(message, "room ID must match its directory name");
        }
        other => panic!("unexpected error: {other}"),
    }
}

#[test]
fn submission_type_and_judge_config_must_match() {
    let test_data = TestProblemData::new();
    let mut value = source_room_json();
    let room_id = source_room_id(&value).to_owned();

    value["problems"][1]["submission_type"] = serde_json::json!("operation_sequence");

    test_data.write_room_json(&room_id, &value);

    let error = load_problem_data(test_data.root())
        .expect_err("mismatched judge config should be rejected");

    match error {
        ProblemDataError::Validation { field, message } => {
            assert_eq!(field, "problems[1].judge_config");
            assert_eq!(message, "judge config type must match submission type");
        }
        other => panic!("unexpected error: {other}"),
    }
}

#[test]
fn object_key_asset_is_loaded() {
    let catalog =
        load_problem_data(mock_problem_data_root()).expect("problem data should be valid");

    let asset = &catalog.rooms[0].problems[0].assets[0];

    assert_eq!(asset.asset_type, "image");
    assert_eq!(
        asset.object_key,
        "problems/1411824c-d357-4941-af76-c76cb827dda6/birthday.png"
    );
    assert_eq!(asset.alt, "生年月日の問題用画像");
}

#[test]
fn empty_asset_object_key_is_rejected() {
    let test_data = TestProblemData::new();
    let mut value = source_room_json();
    let room_id = source_room_id(&value).to_owned();

    value["problems"][0]["assets"][0]["object_key"] = serde_json::json!("");

    test_data.write_room_json(&room_id, &value);

    let error =
        load_problem_data(test_data.root()).expect_err("empty object key should be rejected");

    match error {
        ProblemDataError::Validation { field, message } => {
            assert_eq!(field, "problems[0].assets[0].object_key");
            assert_eq!(message, "must not be empty");
        }
        other => panic!("unexpected error: {other}"),
    }
}

#[test]
fn empty_problem_catalog_is_rejected() {
    let test_data = TestProblemData::new();

    let error =
        load_problem_data(test_data.root()).expect_err("empty problem catalog should be rejected");

    match error {
        ProblemDataError::Validation { field, message } => {
            assert_eq!(field, "rooms");
            assert_eq!(message, "at least one room is required");
        }
        other => panic!("unexpected error: {other}"),
    }
}

#[test]
fn validation_error_does_not_expose_judge_values() {
    let test_data = TestProblemData::new();
    let mut value = source_room_json();
    let room_id = source_room_id(&value).to_owned();

    value["problems"][3]["judge_config"]["accepted_answers"] = serde_json::json!([]);

    test_data.write_room_json(&room_id, &value);

    let error =
        load_problem_data(test_data.root()).expect_err("empty accepted answers should be rejected");

    let display = error.to_string();
    let debug = format!("{error:?}");

    assert!(display.contains("accepted_answers"));
    assert!(!display.contains("ワンマンソン"));
    assert!(!debug.contains("ワンマンソン"));
}
