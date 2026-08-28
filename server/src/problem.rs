mod asset_url;
mod loader;
mod model;
mod public;
mod query;
mod seeder;
mod validation;

use std::{io, path::PathBuf};

use thiserror::Error;

pub(crate) use asset_url::UnconfiguredAssetUrlResolver;
pub use asset_url::{AssetUrlResolveError, AssetUrlResolver};
pub use loader::load_problem_data;
pub use model::{
    Asset, Hint, InputSchema, JudgeConfig, Operation, Problem, ProblemCatalog, ProblemType, Room,
    SubmissionType,
};
pub use public::{ProblemProjectionError, build_problem_response};
pub(crate) use query::{QueryJudgeError, decode_stored_judge_config, judge_query};
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
