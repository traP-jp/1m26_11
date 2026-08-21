use serde::Serialize;
use uuid::Uuid;

use super::model::{Asset, InputSchema, Problem, ProblemType, SubmissionType};

#[derive(Clone, Debug, Serialize)]
pub struct PublicProblem {
    pub id: Uuid,
    pub number: i32,

    #[serde(rename = "type")]
    pub problem_type: ProblemType,

    pub title: String,
    pub body_markdown: String,
    pub submission_type: SubmissionType,
    pub assets: Vec<Asset>,
    pub input_schema: InputSchema,
    pub hint_count: usize,
}

impl From<&Problem> for PublicProblem {
    fn from(problem: &Problem) -> Self {
        Self {
            id: problem.problem_id,
            number: problem.number,
            problem_type: problem.problem_type,
            title: problem.title.clone(),
            body_markdown: problem.body_markdown.clone(),
            submission_type: problem.submission_type,
            assets: problem.assets.clone(),
            input_schema: problem.input_schema.clone(),
            hint_count: problem.hints.len(),
        }
    }
}
