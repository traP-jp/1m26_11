use crate::{
    game_progress::{
        ActiveRunState, ClearProblemError, ClearProblemPlan, ProblemState, ProblemStatus,
        RunStatus, duration_to_elapsed_ms, plan_problem_clear,
    },
    problem::{
        AnswerJudgeError, Asset, InputSchema, Operation, decode_stored_judge_config, judge_answer,
    },
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, MySql, MySqlPool, Transaction};
use thiserror::Error;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthProvider {
    Demo,
    NeoShowcase,
}

impl AuthProvider {
    const fn as_db_str(self) -> &'static str {
        match self {
            Self::Demo => "demo",
            Self::NeoShowcase => "neoshowcase",
        }
    }
}

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("database operation failed")]
    Database(#[source] sqlx::Error),

    #[error("user was not found after get-or-create")]
    UserNotFoundAfterUpsert,

    #[error("active run was not found")]
    RunNotFound,

    #[error("problem was not found in the active run")]
    ProblemNotFound,

    #[error("problem is locked")]
    ProblemLocked,

    #[error("problem is already cleared")]
    ProblemAlreadyCleared,

    #[error("query count is outside the supported range")]
    InvalidQueryCount,

    #[error("answer is empty after normalization")]
    EmptyAnswer,

    #[error("answer exceeds the configured length limit")]
    AnswerLengthExceeded,

    #[error("problem does not accept string answers")]
    WrongAnswerSubmissionType,

    #[error("stored answer configuration is invalid")]
    InvalidStoredAnswerConfig,

    #[error("answer attempt count is outside the supported range")]
    InvalidAnswerAttemptCount,

    #[error("progress count is outside the supported range")]
    InvalidProgressCount,

    #[error("elapsed duration must not be negative")]
    InvalidElapsed,

    #[error("stored problem status is invalid: {status}")]
    InvalidProblemStatus { status: String },

    #[error("problem progress update affected an unexpected number of rows")]
    ProblemProgressUpdateConflict,

    #[error("run update affected an unexpected number of rows")]
    RunUpdateConflict,

    #[error("stored auth provider is invalid")]
    InvalidAuthProvider,
}

impl TryFrom<&str> for AuthProvider {
    type Error = RepositoryError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "demo" => Ok(Self::Demo),
            "neoshowcase" => Ok(Self::NeoShowcase),
            _ => Err(RepositoryError::InvalidAuthProvider),
        }
    }
}

impl From<ClearProblemError> for RepositoryError {
    fn from(error: ClearProblemError) -> Self {
        match error {
            ClearProblemError::ProblemNotFound => Self::ProblemNotFound,
            ClearProblemError::ProblemLocked => Self::ProblemLocked,
            ClearProblemError::InvalidElapsed => Self::InvalidElapsed,
        }
    }
}

impl From<AnswerJudgeError> for RepositoryError {
    fn from(error: AnswerJudgeError) -> Self {
        match error {
            AnswerJudgeError::EmptyAnswer => Self::EmptyAnswer,
            AnswerJudgeError::AnswerLengthExceeded => Self::AnswerLengthExceeded,
            AnswerJudgeError::WrongSubmissionType => Self::WrongAnswerSubmissionType,
            AnswerJudgeError::InvalidStoredConfig => Self::InvalidStoredAnswerConfig,
        }
    }
}

#[derive(Clone)]
pub struct SqlxUserRepository {
    pool: MySqlPool,
}

impl SqlxUserRepository {
    #[must_use]
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthUserRecord {
    pub user_id: Uuid,
    pub display_name: String,
    pub auth_provider: AuthProvider,
}

#[derive(FromRow)]
struct AuthUserRow {
    user_id: Uuid,
    display_name: String,
    auth_provider: String,
}

impl TryFrom<AuthUserRow> for AuthUserRecord {
    type Error = RepositoryError;

    fn try_from(row: AuthUserRow) -> Result<Self, Self::Error> {
        Ok(Self {
            user_id: row.user_id,
            display_name: row.display_name,
            auth_provider: AuthProvider::try_from(row.auth_provider.as_str())?,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, FromRow)]
pub struct RoomRecord {
    pub id: Uuid,
    pub number: i32,
    pub name: String,
    pub genre: String,
    pub description: String,
    pub is_published: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, FromRow)]
pub struct RunRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub room_id: Uuid,
    pub status: String,
    pub started_at: DateTime<Utc>,
    pub cleared_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, PartialEq, FromRow)]
pub struct ProblemRecord {
    pub id: Uuid,
    pub room_id: Uuid,
    pub number: i32,
    pub problem_type: String,
    pub title: String,
    pub body_markdown: String,
    pub submission_type: String,
    pub assets: sqlx::types::Json<serde_json::Value>,
    pub input_schema: sqlx::types::Json<serde_json::Value>,
    pub hints: sqlx::types::Json<serde_json::Value>,
    pub judge_config: sqlx::types::Json<serde_json::Value>,
    pub depends_on_problem_id: Option<Uuid>,
    pub is_required: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, FromRow)]
pub struct ProblemProgressRecord {
    pub run_id: Uuid,
    pub problem_id: Uuid,
    pub status: String,
    pub answer_attempt_count: i32,
    pub cleared_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Eq, PartialEq, FromRow)]
pub struct ProblemDetailRecord {
    pub id: Uuid,
    pub number: i32,
    pub problem_type: String,
    pub title: String,
    pub body_markdown: String,
    pub submission_type: String,
    pub assets: sqlx::types::Json<Vec<Asset>>,
    pub input_schema: sqlx::types::Json<InputSchema>,
    pub judge_config: sqlx::types::Json<serde_json::Value>,
    pub status: String,
    pub hint_count: i64,
}

#[derive(Clone, Eq, PartialEq)]
pub struct QuerySubmission {
    pub query_id: Uuid,
    pub run_id: Uuid,
    pub problem_id: Uuid,
    pub source: String,
    pub operations: Vec<Operation>,
    pub normalized_operations: Vec<Operation>,
    pub remaining_pattern_count: i32,
    pub is_correct: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuerySubmissionResult {
    pub query_count: u64,
    pub problem_status: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnswerSubmission {
    pub run_id: Uuid,
    pub problem_id: Uuid,
    pub answer: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnswerRunStatus {
    Active,
    Cleared,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AnswerSubmissionResult {
    Incorrect {
        answer_attempt_count: i32,
    },
    Correct {
        unlocked_problem_ids: Vec<Uuid>,
        run_status: AnswerRunStatus,
        cleared_problem_count: i32,
        total_problem_count: i32,
        elapsed_ms: u64,
    },
}

#[derive(FromRow)]
struct AnswerProblemRow {
    submission_type: String,
    input_schema: sqlx::types::Json<InputSchema>,
    judge_config: sqlx::types::Json<serde_json::Value>,
    status: String,
    answer_attempt_count: i32,
}

#[derive(FromRow)]
struct GameProgressProblemRow {
    problem_id: Uuid,
    room_id: Uuid,
    depends_on_problem_id: Option<Uuid>,
    is_required: bool,
    status: String,
}

fn parse_problem_status(status: &str) -> Result<ProblemStatus, RepositoryError> {
    match status {
        "locked" => Ok(ProblemStatus::Locked),
        "available" => Ok(ProblemStatus::Available),
        "cleared" => Ok(ProblemStatus::Cleared),
        _ => Err(RepositoryError::InvalidProblemStatus {
            status: status.to_owned(),
        }),
    }
}

#[async_trait]
pub trait AuthRepository: Send + Sync {
    async fn find_user_by_demo_session(
        &self,
        session_id: Uuid,
    ) -> Result<Option<AuthUserRecord>, RepositoryError>;

    async fn find_user_by_provider_subject(
        &self,
        auth_provider: AuthProvider,
        provider_subject: &str,
    ) -> Result<Option<AuthUserRecord>, RepositoryError>;

    async fn get_or_create_user(
        &self,
        auth_provider: AuthProvider,
        provider_subject: &str,
        display_name: &str,
    ) -> Result<AuthUserRecord, RepositoryError>;

    async fn create_demo_session(
        &self,
        _session_id: Uuid,
        _user_id: Uuid,
    ) -> Result<(), RepositoryError> {
        unimplemented!("create_demo_session is not implemented for this repository")
    }

    async fn delete_demo_session(&self, _session_id: Uuid) -> Result<(), RepositoryError> {
        unimplemented!("delete_demo_session is not implemented for this repository")
    }

    async fn find_room_by_id(&self, _room_id: Uuid) -> Result<Option<RoomRecord>, RepositoryError> {
        unimplemented!("find_room_by_id is not implemented for this repository")
    }

    async fn find_active_run(
        &self,
        _user_id: Uuid,
        _room_id: Uuid,
    ) -> Result<Option<RunRecord>, RepositoryError> {
        unimplemented!("find_active_run is not implemented for this repository")
    }

    async fn find_problem_for_run(
        &self,
        _run_id: Uuid,
        _room_id: Uuid,
        _problem_id: Uuid,
    ) -> Result<Option<ProblemDetailRecord>, RepositoryError> {
        unimplemented!("find_problem_for_run is not implemented for this repository")
    }

    async fn record_query_judgement(
        &self,
        _submission: QuerySubmission,
    ) -> Result<QuerySubmissionResult, RepositoryError> {
        unimplemented!("record_query_judgement is not implemented for this repository")
    }

    async fn record_answer_judgement(
        &self,
        _submission: AnswerSubmission,
    ) -> Result<AnswerSubmissionResult, RepositoryError> {
        unimplemented!("record_answer_judgement is not implemented for this repository")
    }

    async fn create_run(
        &self,
        _id: Uuid,
        _user_id: Uuid,
        _room_id: Uuid,
        _started_at: DateTime<Utc>,
    ) -> Result<RunRecord, RepositoryError> {
        unimplemented!("create_run is not implemented for this repository")
    }

    async fn find_cleared_run(
        &self,
        _user_id: Uuid,
        _room_id: Uuid,
    ) -> Result<Option<RunRecord>, RepositoryError> {
        unimplemented!("find_cleared_run is not implemented for this repository")
    }

    async fn find_problems_by_room_id(
        &self,
        _room_id: Uuid,
    ) -> Result<Vec<ProblemRecord>, RepositoryError> {
        unimplemented!("find_problems_by_room_id is not implemented for this repository")
    }

    async fn find_cleared_problem_ids(&self, _run_id: Uuid) -> Result<Vec<Uuid>, RepositoryError> {
        unimplemented!("find_cleared_problem_ids is not implemented for this repository")
    }
}

#[async_trait]
impl AuthRepository for SqlxUserRepository {
    async fn find_user_by_demo_session(
        &self,
        session_id: Uuid,
    ) -> Result<Option<AuthUserRecord>, RepositoryError> {
        let row = sqlx::query_as::<_, AuthUserRow>(
            r#"
            SELECT
                users.user_id,
                users.display_name,
                users.auth_provider
            FROM demo_sessions
            INNER JOIN users ON users.user_id = demo_sessions.user_id
            WHERE demo_sessions.session_id = ?
            AND users.auth_provider = 'demo'
            LIMIT 1
            "#,
        )
        .bind(session_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::Database)?;

        row.map(AuthUserRecord::try_from).transpose()
    }

    async fn find_user_by_provider_subject(
        &self,
        auth_provider: AuthProvider,
        provider_subject: &str,
    ) -> Result<Option<AuthUserRecord>, RepositoryError> {
        let row = sqlx::query_as::<_, AuthUserRow>(
            r#"
            SELECT user_id, display_name, auth_provider
            FROM users
            WHERE auth_provider = ?
            AND provider_subject = ?
            LIMIT 1
            "#,
        )
        .bind(auth_provider.as_db_str())
        .bind(provider_subject)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::Database)?;

        row.map(AuthUserRecord::try_from).transpose()
    }

    async fn get_or_create_user(
        &self,
        auth_provider: AuthProvider,
        provider_subject: &str,
        display_name: &str,
    ) -> Result<AuthUserRecord, RepositoryError> {
        let user_id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO users (
                user_id,
                auth_provider,
                provider_subject,
                display_name
            )
            VALUES (?, ?, ?, ?)
            ON DUPLICATE KEY UPDATE user_id = user_id
            "#,
        )
        .bind(user_id)
        .bind(auth_provider.as_db_str())
        .bind(provider_subject)
        .bind(display_name)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::Database)?;

        self.find_user_by_provider_subject(auth_provider, provider_subject)
            .await?
            .ok_or(RepositoryError::UserNotFoundAfterUpsert)
    }

    async fn create_demo_session(
        &self,
        session_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), RepositoryError> {
        sqlx::query(
            r#"
            INSERT INTO demo_sessions (session_id, user_id)
            VALUES (?, ?)
            "#,
        )
        .bind(session_id)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::Database)?;

        Ok(())
    }

    async fn delete_demo_session(&self, session_id: Uuid) -> Result<(), RepositoryError> {
        sqlx::query(
            r#"
            DELETE FROM demo_sessions
            WHERE session_id = ?
            "#,
        )
        .bind(session_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::Database)?;

        Ok(())
    }

    async fn find_room_by_id(&self, room_id: Uuid) -> Result<Option<RoomRecord>, RepositoryError> {
        sqlx::query_as::<_, RoomRecord>(
            r#"
            SELECT room_id AS id, number, name, genre, description, is_published, created_at
            FROM rooms
            WHERE room_id = ?
            LIMIT 1
            "#,
        )
        .bind(room_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::Database)
    }

    async fn find_active_run(
        &self,
        user_id: Uuid,
        room_id: Uuid,
    ) -> Result<Option<RunRecord>, RepositoryError> {
        sqlx::query_as::<_, RunRecord>(
            r#"
            SELECT run_id AS id, user_id, room_id, status, started_at, cleared_at
            FROM runs
            WHERE user_id = ? AND room_id = ? AND status = 'active'
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .bind(room_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::Database)
    }

    async fn find_problem_for_run(
        &self,
        run_id: Uuid,
        room_id: Uuid,
        problem_id: Uuid,
    ) -> Result<Option<ProblemDetailRecord>, RepositoryError> {
        sqlx::query_as::<_, ProblemDetailRecord>(
            r#"
            SELECT
                problems.problem_id AS id,
                problems.number,
                problems.problem_type,
                problems.title,
                problems.body_markdown,
                problems.submission_type,
                problems.assets,
                problems.input_schema,
                problems.judge_config,
                problem_progress.status,
                CAST(JSON_LENGTH(problems.hints) AS SIGNED) AS hint_count
            FROM problems
            INNER JOIN problem_progress
                ON problem_progress.problem_id = problems.problem_id
               AND problem_progress.run_id = ?
            WHERE problems.room_id = ?
              AND problems.problem_id = ?
            LIMIT 1
            "#,
        )
        .bind(run_id)
        .bind(room_id)
        .bind(problem_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::Database)
    }

    async fn record_query_judgement(
        &self,
        submission: QuerySubmission,
    ) -> Result<QuerySubmissionResult, RepositoryError> {
        let mut transaction = self.pool.begin().await.map_err(RepositoryError::Database)?;

        let room_id = sqlx::query_scalar::<_, Uuid>(
            r#"
            SELECT room_id
            FROM runs
            WHERE run_id = ?
              AND status = 'active'
            FOR UPDATE
            "#,
        )
        .bind(submission.run_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(RepositoryError::Database)?
        .ok_or(RepositoryError::RunNotFound)?;

        let stored_status = sqlx::query_scalar::<_, String>(
            r#"
            SELECT problem_progress.status
            FROM problem_progress
            INNER JOIN problems
                ON problems.problem_id =
                   problem_progress.problem_id
            WHERE problem_progress.run_id = ?
              AND problem_progress.problem_id = ?
              AND problems.room_id = ?
            FOR UPDATE
            "#,
        )
        .bind(submission.run_id)
        .bind(submission.problem_id)
        .bind(room_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(RepositoryError::Database)?
        .ok_or(RepositoryError::ProblemNotFound)?;

        match stored_status.as_str() {
            "available" => {}
            "locked" => {
                return Err(RepositoryError::ProblemLocked);
            }
            "cleared" => {
                return Err(RepositoryError::ProblemAlreadyCleared);
            }
            _ => {
                return Err(RepositoryError::InvalidProblemStatus {
                    status: stored_status,
                });
            }
        }

        sqlx::query(
            r#"
            INSERT INTO queries (
                query_id,
                run_id,
                problem_id,
                source,
                operations,
                normalized_operations,
                remaining_pattern_count,
                is_correct
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(submission.query_id)
        .bind(submission.run_id)
        .bind(submission.problem_id)
        .bind(&submission.source)
        .bind(sqlx::types::Json(&submission.operations))
        .bind(sqlx::types::Json(&submission.normalized_operations))
        .bind(submission.remaining_pattern_count)
        .bind(submission.is_correct)
        .execute(&mut *transaction)
        .await
        .map_err(RepositoryError::Database)?;

        let problem_status = if submission.is_correct {
            let plan = apply_problem_clear_in_transaction(
                &mut transaction,
                submission.run_id,
                submission.problem_id,
            )
            .await?;

            match plan.target_problem_status {
                ProblemStatus::Locked => "locked",
                ProblemStatus::Available => "available",
                ProblemStatus::Cleared => "cleared",
            }
            .to_owned()
        } else {
            "available".to_owned()
        };

        let query_count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM queries
            WHERE run_id = ?
              AND problem_id = ?
            "#,
        )
        .bind(submission.run_id)
        .bind(submission.problem_id)
        .fetch_one(&mut *transaction)
        .await
        .map_err(RepositoryError::Database)?;

        let query_count =
            u64::try_from(query_count).map_err(|_| RepositoryError::InvalidQueryCount)?;

        transaction
            .commit()
            .await
            .map_err(RepositoryError::Database)?;

        Ok(QuerySubmissionResult {
            query_count,
            problem_status,
        })
    }

    async fn record_answer_judgement(
        &self,
        submission: AnswerSubmission,
    ) -> Result<AnswerSubmissionResult, RepositoryError> {
        let mut transaction = self.pool.begin().await.map_err(RepositoryError::Database)?;

        let room_id = sqlx::query_scalar::<_, Uuid>(
            r#"
            SELECT room_id
            FROM runs
            WHERE run_id = ?
              AND status = 'active'
            FOR UPDATE
            "#,
        )
        .bind(submission.run_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(RepositoryError::Database)?
        .ok_or(RepositoryError::RunNotFound)?;

        let problem = sqlx::query_as::<_, AnswerProblemRow>(
            r#"
            SELECT
                problems.submission_type,
                problems.input_schema,
                problems.judge_config,
                problem_progress.status,
                problem_progress.answer_attempt_count
            FROM problem_progress
            INNER JOIN problems
                ON problems.problem_id = problem_progress.problem_id
            WHERE problem_progress.run_id = ?
              AND problem_progress.problem_id = ?
              AND problems.room_id = ?
            FOR UPDATE
            "#,
        )
        .bind(submission.run_id)
        .bind(submission.problem_id)
        .bind(room_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(RepositoryError::Database)?
        .ok_or(RepositoryError::ProblemNotFound)?;

        match problem.status.as_str() {
            "available" => {}
            "locked" => return Err(RepositoryError::ProblemLocked),
            "cleared" => return Err(RepositoryError::ProblemAlreadyCleared),
            _ => {
                return Err(RepositoryError::InvalidProblemStatus {
                    status: problem.status,
                });
            }
        }

        let judge_config = decode_stored_judge_config(
            &problem.submission_type,
            &problem.judge_config.0,
            &problem.input_schema.0,
        )
        .map_err(|_| RepositoryError::InvalidStoredAnswerConfig)?;

        let judgement = judge_answer(&submission.answer, &problem.input_schema.0, &judge_config)?;

        let result = if judgement.correct {
            let plan = apply_problem_clear_in_transaction(
                &mut transaction,
                submission.run_id,
                submission.problem_id,
            )
            .await?;

            let cleared_problem_count = i32::try_from(plan.progress.cleared_problem_count)
                .map_err(|_| RepositoryError::InvalidProgressCount)?;

            let total_problem_count = i32::try_from(plan.progress.total_problem_count)
                .map_err(|_| RepositoryError::InvalidProgressCount)?;

            let elapsed_ms = duration_to_elapsed_ms(plan.elapsed)?;

            let run_status = match plan.run_status {
                RunStatus::Active => AnswerRunStatus::Active,
                RunStatus::Cleared => AnswerRunStatus::Cleared,
            };

            AnswerSubmissionResult::Correct {
                unlocked_problem_ids: plan.unlocked_problem_ids,
                run_status,
                cleared_problem_count,
                total_problem_count,
                elapsed_ms,
            }
        } else {
            if problem.answer_attempt_count < 0 {
                return Err(RepositoryError::InvalidAnswerAttemptCount);
            }

            let answer_attempt_count = problem
                .answer_attempt_count
                .checked_add(1)
                .ok_or(RepositoryError::InvalidAnswerAttemptCount)?;

            let update = sqlx::query(
                r#"
                UPDATE problem_progress
                SET answer_attempt_count = ?
                WHERE run_id = ?
                  AND problem_id = ?
                  AND status = 'available'
                  AND answer_attempt_count = ?
                "#,
            )
            .bind(answer_attempt_count)
            .bind(submission.run_id)
            .bind(submission.problem_id)
            .bind(problem.answer_attempt_count)
            .execute(&mut *transaction)
            .await
            .map_err(RepositoryError::Database)?;

            if update.rows_affected() != 1 {
                return Err(RepositoryError::ProblemProgressUpdateConflict);
            }

            AnswerSubmissionResult::Incorrect {
                answer_attempt_count,
            }
        };

        transaction
            .commit()
            .await
            .map_err(RepositoryError::Database)?;

        Ok(result)
    }

    async fn create_run(
        &self,
        id: Uuid,
        user_id: Uuid,
        room_id: Uuid,
        started_at: DateTime<Utc>,
    ) -> Result<RunRecord, RepositoryError> {
        let mut tx = self.pool.begin().await.map_err(RepositoryError::Database)?;

        sqlx::query(
            r#"
            INSERT INTO runs (run_id, user_id, room_id, status, started_at, cleared_at)
            VALUES (?, ?, ?, 'active', ?, NULL)
            "#,
        )
        .bind(id)
        .bind(user_id)
        .bind(room_id)
        .bind(started_at)
        .execute(&mut *tx)
        .await
        .map_err(RepositoryError::Database)?;

        let problems = sqlx::query_as::<_, ProblemRecord>(
            r#"
            SELECT problem_id AS id, room_id, number, problem_type, title, body_markdown, submission_type,
                   assets, input_schema, hints, judge_config, depends_on_problem_id, is_required
            FROM problems
            WHERE room_id = ?
            ORDER BY number ASC
            "#,
        )
        .bind(room_id)
        .fetch_all(&mut *tx)
        .await
        .map_err(RepositoryError::Database)?;

        for problem in &problems {
            let initial_status = if problem.depends_on_problem_id.is_none() {
                "available"
            } else {
                "locked"
            };

            sqlx::query(
                r#"
                INSERT INTO problem_progress (run_id, problem_id, status, answer_attempt_count, cleared_at)
                VALUES (?, ?, ?, 0, NULL)
                "#,
            )
            .bind(id)
            .bind(problem.id)
            .bind(initial_status)
            .execute(&mut *tx)
            .await
            .map_err(RepositoryError::Database)?;
        }

        tx.commit().await.map_err(RepositoryError::Database)?;

        Ok(RunRecord {
            id,
            user_id,
            room_id,
            status: "active".to_owned(),
            started_at,
            cleared_at: None,
        })
    }

    async fn find_cleared_run(
        &self,
        user_id: Uuid,
        room_id: Uuid,
    ) -> Result<Option<RunRecord>, RepositoryError> {
        sqlx::query_as::<_, RunRecord>(
            r#"
            SELECT run_id AS id, user_id, room_id, status, started_at, cleared_at
            FROM runs
            WHERE user_id = ? AND room_id = ? AND status = 'cleared'
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .bind(room_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::Database)
    }

    async fn find_problems_by_room_id(
        &self,
        room_id: Uuid,
    ) -> Result<Vec<ProblemRecord>, RepositoryError> {
        let problems = sqlx::query_as::<_, ProblemRecord>(
            r#"
            SELECT problem_id AS id, room_id, number, problem_type, title, body_markdown, submission_type,
                   assets, input_schema, hints, judge_config, depends_on_problem_id, is_required
            FROM problems
            WHERE room_id = ?
            ORDER BY number ASC
            "#,
        )
        .bind(room_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::Database)?;

        Ok(problems)
    }

    async fn find_cleared_problem_ids(&self, run_id: Uuid) -> Result<Vec<Uuid>, RepositoryError> {
        #[derive(FromRow)]
        struct ProblemIdRow {
            problem_id: Uuid,
        }

        let rows = sqlx::query_as::<_, ProblemIdRow>(
            r#"
            SELECT problem_id
            FROM problem_progress
            WHERE run_id = ? AND status = 'cleared'
            ORDER BY problem_id ASC
            "#,
        )
        .bind(run_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::Database)?;

        Ok(rows.into_iter().map(|r| r.problem_id).collect())
    }
}

/// Locks the active run and its problem progress, calculates the clear plan,
/// and applies the updates to the supplied transaction.
///
/// This function intentionally does not commit or roll back. Query and answer
/// handlers must include their own records in the same transaction and decide
/// whether to commit the complete operation.
pub(crate) async fn apply_problem_clear_in_transaction(
    transaction: &mut Transaction<'_, MySql>,
    run_id: Uuid,
    target_problem_id: Uuid,
) -> Result<ClearProblemPlan, RepositoryError> {
    let run = sqlx::query_as::<_, RunRecord>(
        r#"
        SELECT
            run_id AS id,
            user_id,
            room_id,
            status,
            started_at,
            cleared_at
        FROM runs
        WHERE run_id = ?
          AND status = 'active'
        FOR UPDATE
        "#,
    )
    .bind(run_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(RepositoryError::Database)?
    .ok_or(RepositoryError::RunNotFound)?;

    let rows = sqlx::query_as::<_, GameProgressProblemRow>(
        r#"
        SELECT
            problems.problem_id,
            problems.room_id,
            problems.depends_on_problem_id,
            problems.is_required,
            problem_progress.status
        FROM problem_progress
        INNER JOIN problems
            ON problems.problem_id = problem_progress.problem_id
        WHERE problem_progress.run_id = ?
          AND problems.room_id = ?
        ORDER BY problems.number
        FOR UPDATE
        "#,
    )
    .bind(run.id)
    .bind(run.room_id)
    .fetch_all(&mut **transaction)
    .await
    .map_err(RepositoryError::Database)?;

    let mut problems = Vec::with_capacity(rows.len());

    for row in rows {
        problems.push(ProblemState {
            problem_id: row.problem_id,
            room_id: row.room_id,
            depends_on_problem_id: row.depends_on_problem_id,
            is_required: row.is_required,
            status: parse_problem_status(&row.status)?,
        });
    }

    let active_run = ActiveRunState {
        run_id: run.id,
        room_id: run.room_id,
        started_at: run.started_at,
    };

    let now = Utc::now();

    let plan = plan_problem_clear(&active_run, &problems, target_problem_id, now)
        .map_err(RepositoryError::from)?;

    if let Some(problem_cleared_at) = plan.problem_cleared_at.as_ref() {
        let result = sqlx::query(
            r#"
            UPDATE problem_progress
            SET status = 'cleared',
                cleared_at = ?
            WHERE run_id = ?
              AND problem_id = ?
              AND status = 'available'
            "#,
        )
        .bind(problem_cleared_at.to_owned())
        .bind(run.id)
        .bind(target_problem_id)
        .execute(&mut **transaction)
        .await
        .map_err(RepositoryError::Database)?;

        if result.rows_affected() != 1 {
            return Err(RepositoryError::ProblemProgressUpdateConflict);
        }
    }

    for unlocked_problem_id in &plan.unlocked_problem_ids {
        let result = sqlx::query(
            r#"
            UPDATE problem_progress
            SET status = 'available'
            WHERE run_id = ?
              AND problem_id = ?
              AND status = 'locked'
            "#,
        )
        .bind(run.id)
        .bind(*unlocked_problem_id)
        .execute(&mut **transaction)
        .await
        .map_err(RepositoryError::Database)?;

        if result.rows_affected() != 1 {
            return Err(RepositoryError::ProblemProgressUpdateConflict);
        }
    }

    if let Some(run_cleared_at) = plan.run_cleared_at.as_ref() {
        let result = sqlx::query(
            r#"
            UPDATE runs
            SET status = 'cleared',
                cleared_at = ?
            WHERE run_id = ?
              AND status = 'active'
            "#,
        )
        .bind(run_cleared_at.to_owned())
        .bind(run.id)
        .execute(&mut **transaction)
        .await
        .map_err(RepositoryError::Database)?;

        if result.rows_affected() != 1 {
            return Err(RepositoryError::RunUpdateConflict);
        }
    }

    Ok(plan)
}

#[cfg(test)]
mod tests {
    use super::{AuthProvider, RepositoryError};

    #[test]
    fn auth_provider_decodes_known_values() {
        let demo = AuthProvider::try_from("demo").expect("demo should decode as an auth provider");
        let neoshowcase = AuthProvider::try_from("neoshowcase")
            .expect("neoshowcase should decode as an auth provider");

        assert_eq!(demo, AuthProvider::Demo);
        assert_eq!(neoshowcase, AuthProvider::NeoShowcase);
    }

    #[test]
    fn auth_provider_rejects_unknown_value() {
        assert!(matches!(
            AuthProvider::try_from("unknown"),
            Err(RepositoryError::InvalidAuthProvider),
        ));
    }
}

#[cfg(test)]
mod game_progress_tests;
