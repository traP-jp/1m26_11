mod judge;
mod loader;
mod model;
mod public;
mod validation;

use std::{io, path::PathBuf};

use thiserror::Error;

pub use judge::{
    JudgeInputError, OperationJudgement, judge_operation_sequence, judge_string_answer,
};
pub use loader::load_problem_data;
pub use model::{
    Asset, InputSchema, JudgeConfig, Operation, Problem, ProblemCatalog, ProblemType, Room,
    SubmissionType,
};
pub use public::PublicProblem;

#[derive(Debug, Error)]
pub enum ProblemDataError {
    #[error("failed to read problem data file: {path}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("invalid problem data JSON: {path}")]
    Json {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    #[error("invalid problem data at {field}: {message}")]
    Validation {
        field: String,
        message: &'static str,
    },
}
