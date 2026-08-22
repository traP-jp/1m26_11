use serde::Serialize;
use sqlx::MySqlPool;
use thiserror::Error;
use uuid::Uuid;

use super::{ProblemCatalog, ProblemType, SubmissionType};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SeedSummary {
    pub room_count: usize,
    pub problem_count: usize,
}

#[derive(Debug, Error)]
pub enum ProblemSeedError {
    #[error("failed to serialize {field} for problem {problem_id}")]
    Json {
        problem_id: Uuid,
        field: &'static str,
        #[source]
        source: serde_json::Error,
    },

    #[error("failed to insert problem data into MariaDB")]
    Database(#[from] sqlx::Error),
}

pub async fn seed_problem_data(
    pool: &MySqlPool,
    catalog: &ProblemCatalog,
) -> Result<SeedSummary, ProblemSeedError> {
    let mut transaction = pool.begin().await?;
    let mut problem_count = 0;

    for room in &catalog.rooms {
        sqlx::query(
            r#"
            INSERT INTO rooms (
                room_id,
                number,
                name,
                genre,
                description,
                is_published
            )
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(room.room_id)
        .bind(room.number)
        .bind(&room.name)
        .bind(&room.genre)
        .bind(&room.description)
        .bind(false)
        .execute(&mut *transaction)
        .await?;

        let mut problems = room.problems.iter().collect::<Vec<_>>();
        problems.sort_by_key(|problem| problem.number);

        for problem in problems {
            let assets = serialize_json(&problem.assets, problem.problem_id, "assets")?;
            let input_schema =
                serialize_json(&problem.input_schema, problem.problem_id, "input_schema")?;
            let hints = serialize_json(&problem.hints, problem.problem_id, "hints")?;
            let judge_config =
                serialize_json(&problem.judge_config, problem.problem_id, "judge_config")?;

            sqlx::query(
                r#"
                INSERT INTO problems (
                    problem_id,
                    room_id,
                    number,
                    problem_type,
                    title,
                    body_markdown,
                    submission_type,
                    assets,
                    input_schema,
                    hints,
                    judge_config,
                    depends_on_problem_id,
                    is_required
                )
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(problem.problem_id)
            .bind(problem.room_id)
            .bind(problem.number)
            .bind(problem_type_name(problem.problem_type))
            .bind(&problem.title)
            .bind(&problem.body_markdown)
            .bind(submission_type_name(problem.submission_type))
            .bind(assets)
            .bind(input_schema)
            .bind(hints)
            .bind(judge_config)
            .bind(problem.depends_on_problem_id)
            .bind(problem.is_required)
            .execute(&mut *transaction)
            .await?;

            problem_count += 1;
        }
    }

    transaction.commit().await?;

    Ok(SeedSummary {
        room_count: catalog.rooms.len(),
        problem_count,
    })
}

fn serialize_json<T>(
    value: &T,
    problem_id: Uuid,
    field: &'static str,
) -> Result<serde_json::Value, ProblemSeedError>
where
    T: Serialize + ?Sized,
{
    serde_json::to_value(value).map_err(|source| ProblemSeedError::Json {
        problem_id,
        field,
        source,
    })
}

fn problem_type_name(problem_type: ProblemType) -> &'static str {
    match problem_type {
        ProblemType::Small => "small",
        ProblemType::Final => "final",
    }
}

fn submission_type_name(submission_type: SubmissionType) -> &'static str {
    match submission_type {
        SubmissionType::OperationSequence => "operation_sequence",
        SubmissionType::String => "string",
    }
}
