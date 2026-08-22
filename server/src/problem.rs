mod loader;
mod model;
mod seeder;
mod validation;

use std::{io, path::PathBuf};

use thiserror::Error;

pub use loader::load_problem_data;
pub use model::{
    Asset, InputSchema, JudgeConfig, Operation, Problem, ProblemCatalog, ProblemType, Room,
    SubmissionType,
};
pub use seeder::{ProblemSeedError, SeedSummary, seed_problem_data};

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
