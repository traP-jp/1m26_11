use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RoomFileInput {
    pub room: RoomInput,
    pub problems: Vec<ProblemInput>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RoomInput {
    pub room_id: String,
    pub number: i32,
    pub name: String,
    pub genre: String,
    pub description: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ProblemInput {
    pub problem_id: String,
    pub room_id: String,
    pub number: i32,
    pub problem_type: ProblemType,
    pub title: String,
    pub body_markdown: String,
    pub submission_type: SubmissionType,
    pub assets: Vec<Asset>,
    pub input_schema: InputSchema,
    pub hints: Vec<Hint>,
    pub judge_config: JudgeConfigInput,
    pub depends_on_problem_id: RequiredNullable<String>,
    pub is_required: bool,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub(super) enum RequiredNullable<T> {
    Value(T),
    Null,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ProblemType {
    Small,
    Final,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SubmissionType {
    OperationSequence,
    String,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum JudgeConfigInput {
    OperationSequence {
        correct_operations: Vec<Operation>,
        candidates: Vec<Candidate>,
    },
    String {
        accepted_answers: Vec<String>,
        normalization: StringNormalization,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum JudgeConfig {
    OperationSequence {
        correct_operations: Vec<Operation>,
        candidates: Vec<Candidate>,
    },
    String {
        accepted_answers: Vec<String>,
        normalization: StringNormalization,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Asset {
    #[serde(rename = "type")]
    pub asset_type: String,
    pub object_key: String,
    pub alt: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct InputSchema {
    pub query: QueryInputSchema,
    pub answer: AnswerInputSchema,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct QueryInputSchema {
    #[serde(rename = "type")]
    pub input_type: QueryInputType,
    pub allowed_controls: Vec<String>,
    pub max_operations: i32,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum QueryInputType {
    OperationSequence,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AnswerInputSchema {
    #[serde(rename = "type")]
    pub input_type: AnswerInputType,
    pub max_length: i32,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AnswerInputType {
    String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Hint {
    pub body_markdown: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Operation {
    pub control: String,
    pub count: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Candidate {
    pub candidate_id: String,
    pub operations: Vec<Operation>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct StringNormalization {
    pub unicode: UnicodeNormalization,
    pub trim_outer_whitespace: bool,
    pub collapse_internal_whitespace: bool,
    pub case_sensitive: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum UnicodeNormalization {
    Nfkc,
}

#[derive(Clone)]
pub struct ProblemCatalog {
    pub rooms: Vec<Room>,
}

#[derive(Clone)]
pub struct Room {
    pub room_id: Uuid,
    pub number: i32,
    pub name: String,
    pub genre: String,
    pub description: String,
    pub problems: Vec<Problem>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProblemDraft {
    pub room_id: Uuid,
    pub number: i32,
    pub problem_type: ProblemType,
    pub title: String,
    pub body_markdown: String,
    pub submission_type: SubmissionType,
    pub input_schema: InputSchema,
    pub hints: Vec<Hint>,
    pub judge_config: JudgeConfig,
    pub depends_on_problem_id: Option<Uuid>,
    pub is_required: bool,
}

#[derive(Clone)]
pub struct Problem {
    pub problem_id: Uuid,
    pub room_id: Uuid,
    pub number: i32,
    pub problem_type: ProblemType,
    pub title: String,
    pub body_markdown: String,
    pub submission_type: SubmissionType,
    pub assets: Vec<Asset>,
    pub input_schema: InputSchema,
    pub hints: Vec<Hint>,
    pub judge_config: JudgeConfig,
    pub depends_on_problem_id: Option<Uuid>,
    pub is_required: bool,
}
