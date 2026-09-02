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

    #[error("leaderboard rank is outside the supported range")]
    InvalidLeaderboardRank,

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

    #[error("stored run status is invalid: {status}")]
    InvalidRunStatus { status: String },

    #[error("problem progress update affected an unexpected number of rows")]
    ProblemProgressUpdateConflict,

    #[error("run update affected an unexpected number of rows")]
    RunUpdateConflict,

    #[error("stored asset upload idempotency record is invalid")]
    InvalidAssetUploadIdempotencyRecord,

    #[error("asset upload target changed during processing")]
    AssetUploadTargetChanged,

    #[error("published room cannot be modified")]
    PublishedRoomImmutable,

    #[error("problem assets update affected an unexpected number of rows")]
    ProblemAssetsUpdateConflict,

    #[error("asset upload idempotency update affected an unexpected number of rows")]
    AssetUploadIdempotencyUpdateConflict,

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LeaderboardRecord {
    pub rank: u32,
    pub user_id: Uuid,
    pub display_name: String,
    pub elapsed_ms: u64,
    pub query_count: u64,
    pub cleared_at: DateTime<Utc>,
}

#[derive(FromRow)]
struct LeaderboardRow {
    rank: i64,
    user_id: Uuid,
    display_name: String,
    started_at: DateTime<Utc>,
    query_count: i64,
    cleared_at: DateTime<Utc>,
}

impl TryFrom<LeaderboardRow> for LeaderboardRecord {
    type Error = RepositoryError;

    fn try_from(row: LeaderboardRow) -> Result<Self, Self::Error> {
        let rank = u32::try_from(row.rank).map_err(|_| RepositoryError::InvalidLeaderboardRank)?;
        let elapsed = row.cleared_at.signed_duration_since(row.started_at);
        let elapsed_ms = duration_to_elapsed_ms(elapsed)?;
        let query_count =
            u64::try_from(row.query_count).map_err(|_| RepositoryError::InvalidQueryCount)?;

        Ok(Self {
            rank,
            user_id: row.user_id,
            display_name: row.display_name,
            elapsed_ms,
            query_count,
            cleared_at: row.cleared_at,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserProgressRecord {
    pub cleared_room_count: u32,
    pub total_room_count: u32,
    pub by_genre: Vec<GenreProgressRecord>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenreProgressRecord {
    pub genre: String,
    pub cleared_room_count: u32,
    pub total_room_count: u32,
}

#[derive(FromRow)]
struct GenreProgressRow {
    genre: String,
    cleared_room_count: i64,
    total_room_count: i64,
}

impl TryFrom<GenreProgressRow> for GenreProgressRecord {
    type Error = RepositoryError;

    fn try_from(row: GenreProgressRow) -> Result<Self, Self::Error> {
        let cleared_room_count = u32::try_from(row.cleared_room_count)
            .map_err(|_| RepositoryError::InvalidProgressCount)?;

        let total_room_count = u32::try_from(row.total_room_count)
            .map_err(|_| RepositoryError::InvalidProgressCount)?;

        Ok(Self {
            genre: row.genre,
            cleared_room_count,
            total_room_count,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HintRecord {
    pub level: i32,
    pub body_markdown: String,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, FromRow)]
pub struct AssetUploadTargetRecord {
    pub is_published: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetUploadClaimRequest {
    pub request_method: String,
    pub request_path: String,
    pub idempotency_key: Uuid,
    pub file_sha256: [u8; 32],
    pub alt: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AssetUploadClaimOutcome {
    Acquired { claim_token: Uuid },
    InProgress,
    Reused,
    Completed { asset: Asset },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompleteAssetUploadRequest {
    pub request_method: String,
    pub request_path: String,
    pub idempotency_key: Uuid,
    pub claim_token: Uuid,
    pub room_id: Uuid,
    pub problem_id: Uuid,
    pub asset: Asset,
    pub completed_at: DateTime<Utc>,
}

#[derive(FromRow)]
struct AssetUploadIdempotencyRow {
    claim_token: Uuid,
    file_sha256: Vec<u8>,
    alt: String,
    status: String,
    object_key: Option<String>,
}

#[derive(FromRow)]
struct AssetUploadCompletionTargetRow {
    is_published: bool,
    assets: sqlx::types::Json<Vec<Asset>>,
}

#[derive(FromRow)]
struct AssetUploadCompletionRow {
    claim_token: Uuid,
    alt: String,
    status: String,
}

#[derive(Clone, Debug, Eq, PartialEq, FromRow)]
pub struct ProblemProgressRecord {
    pub run_id: Uuid,
    pub problem_id: Uuid,
    pub status: String,
    pub answer_attempt_count: i32,
    pub max_hint_level: i32,
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
pub struct RoomBestRecordRecord {
    pub elapsed_ms: u64,
    pub rank: u32,
    pub query_count: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoomSummaryRecord {
    pub room_id: Uuid,
    pub number: i32,
    pub name: String,
    pub genre: String,
    pub description: String,
    pub problem_count: u32,
    pub progress_status: String,
    pub cleared_count: u32,
    pub required_count: u32,
    pub best_record: Option<RoomBestRecordRecord>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AnswerSubmissionResult {
    Incorrect {
        answer_attempt_count: i32,
    },
    Correct {
        unlocked_problem_ids: Vec<Uuid>,
        run_status: AnswerRunStatus,
        cleared_count: u32,
        required_count: u32,
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

    async fn get_or_create_demo_user_and_session(
        &self,
        _session_id: Uuid,
        _provider_subject: &str,
        _display_name: &str,
    ) -> Result<AuthUserRecord, RepositoryError> {
        unimplemented!("get_or_create_demo_user_and_session is not implemented for this repository")
    }

    async fn delete_demo_session(&self, _session_id: Uuid) -> Result<(), RepositoryError> {
        unimplemented!("delete_demo_session is not implemented for this repository")
    }

    async fn find_published_rooms_with_progress(
        &self,
        _user_id: Option<Uuid>,
    ) -> Result<Vec<RoomSummaryRecord>, RepositoryError> {
        unimplemented!("find_published_rooms_with_progress is not implemented for this repository")
    }

    async fn find_room_by_id(&self, _room_id: Uuid) -> Result<Option<RoomRecord>, RepositoryError> {
        unimplemented!("find_room_by_id is not implemented for this repository")
    }

    async fn find_asset_upload_target(
        &self,
        _room_id: Uuid,
        _problem_id: Uuid,
    ) -> Result<Option<AssetUploadTargetRecord>, RepositoryError> {
        unimplemented!("find_asset_upload_target is not implemented for this repository")
    }

    async fn claim_asset_upload(
        &self,
        _request: &AssetUploadClaimRequest,
    ) -> Result<AssetUploadClaimOutcome, RepositoryError> {
        unimplemented!("claim_asset_upload is not implemented for this repository")
    }

    async fn complete_asset_upload(
        &self,
        _request: &CompleteAssetUploadRequest,
    ) -> Result<(), RepositoryError> {
        unimplemented!("complete_asset_upload is not implemented for this repository")
    }

    async fn release_asset_upload_claim(
        &self,
        _request_method: &str,
        _request_path: &str,
        _idempotency_key: Uuid,
        _claim_token: Uuid,
    ) -> Result<(), RepositoryError> {
        unimplemented!("release_asset_upload_claim is not implemented for this repository")
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

    async fn find_leaderboard_by_room_id(
        &self,
        _room_id: Uuid,
    ) -> Result<Vec<LeaderboardRecord>, RepositoryError> {
        unimplemented!("find_leaderboard_by_room_id is not implemented for this repository")
    }

    async fn find_user_progress(
        &self,
        _user_id: Uuid,
    ) -> Result<UserProgressRecord, RepositoryError> {
        unimplemented!("find_user_progress is not implemented for this repository")
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

    async fn find_hint_for_run(
        &self,
        _run_id: Uuid,
        _room_id: Uuid,
        _problem_id: Uuid,
        _level: i32,
    ) -> Result<Option<HintRecord>, RepositoryError> {
        unimplemented!("find_hint_for_run is not implemented for this repository")
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

    async fn get_or_create_demo_user_and_session(
        &self,
        session_id: Uuid,
        provider_subject: &str,
        display_name: &str,
    ) -> Result<AuthUserRecord, RepositoryError> {
        let mut transaction = self.pool.begin().await.map_err(RepositoryError::Database)?;

        let new_user_id = Uuid::new_v4();

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
        .bind(new_user_id)
        .bind(AuthProvider::Demo.as_db_str())
        .bind(provider_subject)
        .bind(display_name)
        .execute(&mut *transaction)
        .await
        .map_err(RepositoryError::Database)?;

        let user_record = sqlx::query_as::<_, AuthUserRow>(
            r#"
            SELECT user_id, display_name, auth_provider
            FROM users
            WHERE auth_provider = ?
              AND provider_subject = ?
            LIMIT 1
            "#,
        )
        .bind(AuthProvider::Demo.as_db_str())
        .bind(provider_subject)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(RepositoryError::Database)?
        .map(AuthUserRecord::try_from)
        .transpose()?
        .ok_or(RepositoryError::UserNotFoundAfterUpsert)?;

        sqlx::query(
            r#"
            INSERT INTO demo_sessions (session_id, user_id)
            VALUES (?, ?)
            "#,
        )
        .bind(session_id)
        .bind(user_record.user_id)
        .execute(&mut *transaction)
        .await
        .map_err(RepositoryError::Database)?;

        transaction
            .commit()
            .await
            .map_err(RepositoryError::Database)?;

        Ok(user_record)
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

    async fn find_published_rooms_with_progress(
        &self,
        user_id: Option<Uuid>,
    ) -> Result<Vec<RoomSummaryRecord>, RepositoryError> {
        #[derive(FromRow)]
        struct RoomRow {
            room_id: Uuid,
            number: i32,
            name: String,
            genre: String,
            description: String,
            problem_count: i64,
            required_count: i64,
        }

        let room_rows = sqlx::query_as::<_, RoomRow>(
            r#"
            SELECT
                rooms.room_id,
                rooms.number,
                rooms.name,
                rooms.genre,
                rooms.description,
                COUNT(problems.problem_id) AS problem_count,
                CAST(
                    COALESCE(
                        SUM(CASE WHEN problems.is_required = 1 THEN 1 ELSE 0 END),
                        0
                    ) AS SIGNED
                ) AS required_count
            FROM rooms
            LEFT JOIN problems ON problems.room_id = rooms.room_id
            WHERE rooms.is_published = 1
            GROUP BY rooms.room_id, rooms.number, rooms.name, rooms.genre, rooms.description
            ORDER BY rooms.number ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::Database)?;

        if room_rows.is_empty() {
            return Ok(Vec::new());
        }

        let user_id = match user_id {
            Some(uid) => uid,
            None => {
                return room_rows
                    .into_iter()
                    .map(|row| {
                        Ok(RoomSummaryRecord {
                            room_id: row.room_id,
                            number: row.number,
                            name: row.name,
                            genre: row.genre,
                            description: row.description,
                            problem_count: u32::try_from(row.problem_count)
                                .map_err(|_| RepositoryError::InvalidProgressCount)?,
                            progress_status: "not_started".to_owned(),
                            cleared_count: 0,
                            required_count: u32::try_from(row.required_count)
                                .map_err(|_| RepositoryError::InvalidProgressCount)?,
                            best_record: None,
                        })
                    })
                    .collect();
            }
        };

        #[derive(FromRow)]
        struct UserRunRow {
            run_id: Uuid,
            room_id: Uuid,
            status: String,
        }

        let user_runs = sqlx::query_as::<_, UserRunRow>(
            r#"
            SELECT
                runs.run_id,
                runs.room_id,
                runs.status
            FROM runs
            INNER JOIN rooms ON rooms.room_id = runs.room_id
            WHERE runs.user_id = ?
              AND rooms.is_published = 1
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::Database)?;

        let mut user_active_rooms: std::collections::HashMap<Uuid, Uuid> =
            std::collections::HashMap::new();
        let mut user_cleared_rooms: std::collections::HashSet<Uuid> =
            std::collections::HashSet::new();

        for run in &user_runs {
            match run.status.as_str() {
                "active" => {
                    if user_cleared_rooms.contains(&run.room_id) {
                        return Err(RepositoryError::InvalidRunStatus {
                            status: "active and cleared run conflict".to_owned(),
                        });
                    }
                    user_active_rooms.insert(run.room_id, run.run_id);
                }
                "cleared" => {
                    if user_active_rooms.contains_key(&run.room_id) {
                        return Err(RepositoryError::InvalidRunStatus {
                            status: "active and cleared run conflict".to_owned(),
                        });
                    }
                    user_cleared_rooms.insert(run.room_id);
                }
                _ => {}
            }
        }

        #[derive(FromRow)]
        struct ActiveProgressRow {
            room_id: Uuid,
            cleared_count: i64,
        }

        let active_progress_rows = sqlx::query_as::<_, ActiveProgressRow>(
            r#"
            SELECT
                runs.room_id,
                COUNT(problem_progress.problem_id) AS cleared_count
            FROM runs
            INNER JOIN problem_progress ON problem_progress.run_id = runs.run_id
            INNER JOIN problems ON problems.problem_id = problem_progress.problem_id
            WHERE runs.user_id = ?
              AND runs.status = 'active'
              AND problem_progress.status = 'cleared'
              AND problems.is_required = 1
            GROUP BY runs.room_id
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::Database)?;

        let active_cleared_counts: std::collections::HashMap<Uuid, u32> = active_progress_rows
            .into_iter()
            .map(|r| {
                let count = u32::try_from(r.cleared_count)
                    .map_err(|_| RepositoryError::InvalidProgressCount)?;
                Ok((r.room_id, count))
            })
            .collect::<Result<_, RepositoryError>>()?;

        #[derive(FromRow)]
        struct UserRoomBestRow {
            room_id: Uuid,
            rank: i64,
            elapsed_ms: i64,
            query_count: i64,
        }

        let user_room_best_rows = if !user_cleared_rooms.is_empty() {
            let mut query_builder = sqlx::QueryBuilder::<sqlx::MySql>::new(
                r#"
                WITH run_records AS (
                    SELECT
                        runs.run_id,
                        runs.user_id,
                        runs.room_id,
                        runs.started_at,
                        CAST(
                            TIMESTAMPDIFF(
                                MICROSECOND,
                                runs.started_at,
                                runs.cleared_at
                            ) DIV 1000
                            AS SIGNED
                        ) AS elapsed_ms,
                        CAST(
                            COALESCE(
                                SUM(
                                    CASE
                                        WHEN problems.submission_type = 'operation_sequence'
                                        THEN problem_progress.answer_attempt_count
                                        ELSE 0
                                    END
                                ),
                                0
                            )
                            AS SIGNED
                        ) AS query_count,
                        runs.cleared_at
                    FROM runs
                    INNER JOIN rooms ON rooms.room_id = runs.room_id
                    LEFT JOIN problem_progress ON problem_progress.run_id = runs.run_id
                    LEFT JOIN problems
                        ON problems.problem_id = problem_progress.problem_id
                       AND problems.room_id = runs.room_id
                    WHERE rooms.is_published = 1
                      AND runs.status = 'cleared'
                      AND runs.cleared_at IS NOT NULL
                      AND runs.room_id IN (
                "#,
            );
            let mut separated = query_builder.separated(", ");
            for room_id in &user_cleared_rooms {
                separated.push_bind(*room_id);
            }
            separated.push_unseparated(
                r#")
                    GROUP BY
                        runs.run_id,
                        runs.user_id,
                        runs.room_id,
                        runs.started_at,
                        runs.cleared_at
                ),
                user_best_candidates AS (
                    SELECT
                        run_id,
                        user_id,
                        room_id,
                        started_at,
                        elapsed_ms,
                        query_count,
                        cleared_at,
                        ROW_NUMBER() OVER (
                            PARTITION BY room_id, user_id
                            ORDER BY
                                elapsed_ms,
                                query_count,
                                cleared_at,
                                run_id
                        ) AS user_position
                    FROM run_records
                ),
                best_runs AS (
                    SELECT
                        run_id,
                        user_id,
                        room_id,
                        started_at,
                        elapsed_ms,
                        query_count,
                        cleared_at
                    FROM user_best_candidates
                    WHERE user_position = 1
                ),
                ranked_best_runs AS (
                    SELECT
                        room_id,
                        user_id,
                        elapsed_ms,
                        query_count,
                        CAST(
                            RANK() OVER (
                                PARTITION BY room_id
                                ORDER BY
                                    elapsed_ms,
                                    query_count,
                                    cleared_at
                            )
                            AS SIGNED
                        ) AS leaderboard_rank
                    FROM best_runs
                )
                SELECT
                    room_id,
                    leaderboard_rank AS `rank`,
                    elapsed_ms,
                    query_count
                FROM ranked_best_runs
                WHERE user_id = "#,
            );
            query_builder.push_bind(user_id);

            query_builder
                .build_query_as::<UserRoomBestRow>()
                .fetch_all(&self.pool)
                .await
                .map_err(RepositoryError::Database)?
        } else {
            Vec::new()
        };

        let mut user_room_best_records: std::collections::HashMap<Uuid, RoomBestRecordRecord> =
            std::collections::HashMap::new();

        for row in user_room_best_rows {
            if row.elapsed_ms < 0 {
                return Err(RepositoryError::InvalidElapsed);
            }
            let elapsed_ms =
                u64::try_from(row.elapsed_ms).map_err(|_| RepositoryError::InvalidElapsed)?;
            let query_count =
                u64::try_from(row.query_count).map_err(|_| RepositoryError::InvalidQueryCount)?;
            let rank =
                u32::try_from(row.rank).map_err(|_| RepositoryError::InvalidProgressCount)?;

            user_room_best_records.insert(
                row.room_id,
                RoomBestRecordRecord {
                    elapsed_ms,
                    rank,
                    query_count,
                },
            );
        }

        room_rows
            .into_iter()
            .map(|row| {
                let problem_count = u32::try_from(row.problem_count)
                    .map_err(|_| RepositoryError::InvalidProgressCount)?;
                let required_count = u32::try_from(row.required_count)
                    .map_err(|_| RepositoryError::InvalidProgressCount)?;

                let (progress_status, cleared_count, best_record) =
                    if user_cleared_rooms.contains(&row.room_id) {
                        let best = user_room_best_records.get(&row.room_id).cloned();
                        ("cleared".to_owned(), required_count, best)
                    } else if user_active_rooms.contains_key(&row.room_id) {
                        let cleared = active_cleared_counts
                            .get(&row.room_id)
                            .copied()
                            .unwrap_or(0);
                        ("active".to_owned(), cleared, None)
                    } else {
                        ("not_started".to_owned(), 0, None)
                    };

                Ok(RoomSummaryRecord {
                    room_id: row.room_id,
                    number: row.number,
                    name: row.name,
                    genre: row.genre,
                    description: row.description,
                    problem_count,
                    progress_status,
                    cleared_count,
                    required_count,
                    best_record,
                })
            })
            .collect()
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

    async fn find_asset_upload_target(
        &self,
        room_id: Uuid,
        problem_id: Uuid,
    ) -> Result<Option<AssetUploadTargetRecord>, RepositoryError> {
        sqlx::query_as::<_, AssetUploadTargetRecord>(
            r#"
            SELECT rooms.is_published
            FROM rooms
            INNER JOIN problems
                ON problems.room_id = rooms.room_id
            WHERE rooms.room_id = ?
              AND problems.problem_id = ?
            "#,
        )
        .bind(room_id)
        .bind(problem_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::Database)
    }

    async fn claim_asset_upload(
        &self,
        request: &AssetUploadClaimRequest,
    ) -> Result<AssetUploadClaimOutcome, RepositoryError> {
        let claim_token = Uuid::new_v4();
        let mut transaction = self.pool.begin().await.map_err(RepositoryError::Database)?;

        sqlx::query(
            r#"
            INSERT INTO asset_upload_idempotency (
                request_method,
                request_path,
                idempotency_key,
                claim_token,
                file_sha256,
                alt,
                status,
                expires_at
            )
            VALUES (?, ?, ?, ?, ?, ?, 'processing', ?)
            ON DUPLICATE KEY UPDATE
                claim_token = IF(
                    expires_at <= CURRENT_TIMESTAMP(3),
                    VALUES(claim_token),
                    claim_token
                ),
                file_sha256 = IF(
                    expires_at <= CURRENT_TIMESTAMP(3),
                    VALUES(file_sha256),
                    file_sha256
                ),
                alt = IF(
                    expires_at <= CURRENT_TIMESTAMP(3),
                    VALUES(alt),
                    alt
                ),
                status = IF(
                    expires_at <= CURRENT_TIMESTAMP(3),
                    VALUES(status),
                    status
                ),
                object_key = IF(
                    expires_at <= CURRENT_TIMESTAMP(3),
                    NULL,
                    object_key
                ),
                created_at = IF(
                    expires_at <= CURRENT_TIMESTAMP(3),
                    CURRENT_TIMESTAMP(3),
                    created_at
                ),
                completed_at = IF(
                    expires_at <= CURRENT_TIMESTAMP(3),
                    NULL,
                    completed_at
                ),
                expires_at = IF(
                    expires_at <= CURRENT_TIMESTAMP(3),
                    VALUES(expires_at),
                    expires_at
                )
            "#,
        )
        .bind(&request.request_method)
        .bind(&request.request_path)
        .bind(request.idempotency_key)
        .bind(claim_token)
        .bind(request.file_sha256.as_slice())
        .bind(&request.alt)
        .bind(request.expires_at.to_owned())
        .execute(&mut *transaction)
        .await
        .map_err(RepositoryError::Database)?;

        let row = sqlx::query_as::<_, AssetUploadIdempotencyRow>(
            r#"
            SELECT
                claim_token,
                file_sha256,
                alt,
                status,
                CAST(
                    object_key AS CHAR CHARACTER SET utf8mb4
                ) AS object_key
            FROM asset_upload_idempotency
            WHERE request_method = ?
              AND request_path = ?
              AND idempotency_key = ?
            FOR UPDATE
            "#,
        )
        .bind(&request.request_method)
        .bind(&request.request_path)
        .bind(request.idempotency_key)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(RepositoryError::Database)?
        .ok_or(RepositoryError::InvalidAssetUploadIdempotencyRecord)?;

        let same_payload = row.file_sha256.as_slice() == request.file_sha256.as_slice()
            && row.alt.as_str() == request.alt.as_str();

        let outcome = if row.claim_token == claim_token {
            AssetUploadClaimOutcome::Acquired { claim_token }
        } else if !same_payload {
            AssetUploadClaimOutcome::Reused
        } else {
            match row.status.as_str() {
                "processing" => AssetUploadClaimOutcome::InProgress,
                "completed" => {
                    let object_key = row
                        .object_key
                        .ok_or(RepositoryError::InvalidAssetUploadIdempotencyRecord)?;

                    AssetUploadClaimOutcome::Completed {
                        asset: Asset {
                            asset_type: "image".to_owned(),
                            object_key,
                            alt: row.alt,
                        },
                    }
                }
                _ => return Err(RepositoryError::InvalidAssetUploadIdempotencyRecord),
            }
        };

        transaction
            .commit()
            .await
            .map_err(RepositoryError::Database)?;

        Ok(outcome)
    }

    async fn complete_asset_upload(
        &self,
        request: &CompleteAssetUploadRequest,
    ) -> Result<(), RepositoryError> {
        let mut transaction = self.pool.begin().await.map_err(RepositoryError::Database)?;

        let target = sqlx::query_as::<_, AssetUploadCompletionTargetRow>(
            r#"
            SELECT
                rooms.is_published,
                problems.assets
            FROM rooms
            INNER JOIN problems
                ON problems.room_id = rooms.room_id
            WHERE rooms.room_id = ?
              AND problems.problem_id = ?
            FOR UPDATE
            "#,
        )
        .bind(request.room_id)
        .bind(request.problem_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(RepositoryError::Database)?
        .ok_or(RepositoryError::AssetUploadTargetChanged)?;

        if target.is_published {
            return Err(RepositoryError::PublishedRoomImmutable);
        }

        let claim = sqlx::query_as::<_, AssetUploadCompletionRow>(
            r#"
            SELECT claim_token, alt, status
            FROM asset_upload_idempotency
            WHERE request_method = ?
              AND request_path = ?
              AND idempotency_key = ?
            FOR UPDATE
            "#,
        )
        .bind(&request.request_method)
        .bind(&request.request_path)
        .bind(request.idempotency_key)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(RepositoryError::Database)?
        .ok_or(RepositoryError::InvalidAssetUploadIdempotencyRecord)?;

        if claim.claim_token != request.claim_token
            || claim.status != "processing"
            || claim.alt != request.asset.alt
        {
            return Err(RepositoryError::InvalidAssetUploadIdempotencyRecord);
        }

        let mut assets = target.assets.0;
        assets.push(request.asset.clone());

        let problem_update = sqlx::query(
            r#"
            UPDATE problems
            SET assets = ?
            WHERE room_id = ?
              AND problem_id = ?
            "#,
        )
        .bind(sqlx::types::Json(assets))
        .bind(request.room_id)
        .bind(request.problem_id)
        .execute(&mut *transaction)
        .await
        .map_err(RepositoryError::Database)?;

        if problem_update.rows_affected() != 1 {
            return Err(RepositoryError::ProblemAssetsUpdateConflict);
        }

        let idempotency_update = sqlx::query(
            r#"
            UPDATE asset_upload_idempotency
            SET status = 'completed',
                object_key = ?,
                completed_at = ?
            WHERE request_method = ?
              AND request_path = ?
              AND idempotency_key = ?
              AND claim_token = ?
              AND status = 'processing'
            "#,
        )
        .bind(&request.asset.object_key)
        .bind(request.completed_at.to_owned())
        .bind(&request.request_method)
        .bind(&request.request_path)
        .bind(request.idempotency_key)
        .bind(request.claim_token)
        .execute(&mut *transaction)
        .await
        .map_err(RepositoryError::Database)?;

        if idempotency_update.rows_affected() != 1 {
            return Err(RepositoryError::AssetUploadIdempotencyUpdateConflict);
        }

        transaction
            .commit()
            .await
            .map_err(RepositoryError::Database)?;

        Ok(())
    }

    async fn release_asset_upload_claim(
        &self,
        request_method: &str,
        request_path: &str,
        idempotency_key: Uuid,
        claim_token: Uuid,
    ) -> Result<(), RepositoryError> {
        sqlx::query(
            r#"
            DELETE FROM asset_upload_idempotency
            WHERE request_method = ?
              AND request_path = ?
              AND idempotency_key = ?
              AND claim_token = ?
              AND status = 'processing'
            "#,
        )
        .bind(request_method)
        .bind(request_path)
        .bind(idempotency_key)
        .bind(claim_token)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::Database)?;

        Ok(())
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

        let (stored_status, answer_attempt_count) = sqlx::query_as::<_, (String, i32)>(
            r#"
            SELECT
                problem_progress.status,
                problem_progress.answer_attempt_count
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

        if answer_attempt_count < 0 {
            return Err(RepositoryError::InvalidAnswerAttemptCount);
        }

        let next_answer_attempt_count = answer_attempt_count
            .checked_add(1)
            .ok_or(RepositoryError::InvalidAnswerAttemptCount)?;

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

        let counter_update = sqlx::query(
            r#"
            UPDATE problem_progress
            SET answer_attempt_count = ?
            WHERE run_id = ?
              AND problem_id = ?
              AND status = 'available'
              AND answer_attempt_count = ?
            "#,
        )
        .bind(next_answer_attempt_count)
        .bind(submission.run_id)
        .bind(submission.problem_id)
        .bind(answer_attempt_count)
        .execute(&mut *transaction)
        .await
        .map_err(RepositoryError::Database)?;

        if counter_update.rows_affected() != 1 {
            return Err(RepositoryError::ProblemProgressUpdateConflict);
        }
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

        let query_count = u64::try_from(next_answer_attempt_count)
            .map_err(|_| RepositoryError::InvalidQueryCount)?;

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

            let cleared_count = u32::try_from(plan.progress.cleared_count)
                .map_err(|_| RepositoryError::InvalidProgressCount)?;

            let required_count = u32::try_from(plan.progress.required_count)
                .map_err(|_| RepositoryError::InvalidProgressCount)?;

            let elapsed_ms = duration_to_elapsed_ms(plan.elapsed)?;

            let run_status = match plan.run_status {
                RunStatus::Active => AnswerRunStatus::Active,
                RunStatus::Cleared => AnswerRunStatus::Cleared,
            };

            AnswerSubmissionResult::Correct {
                unlocked_problem_ids: plan.unlocked_problem_ids,
                run_status,
                cleared_count,
                required_count,
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

        let insert_result = sqlx::query(
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
        .await;

        if let Err(error) = insert_result {
            let is_unique_violation = error
                .as_database_error()
                .is_some_and(|database_error| database_error.is_unique_violation());

            tx.rollback().await.map_err(RepositoryError::Database)?;

            if is_unique_violation {
                if let Some(active_run) = self.find_active_run(user_id, room_id).await? {
                    return Ok(active_run);
                }
            }

            return Err(RepositoryError::Database(error));
        }

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
            "#,
        )
        .bind(id)
        .fetch_one(&mut *tx)
        .await
        .map_err(RepositoryError::Database)?;

        tx.commit().await.map_err(RepositoryError::Database)?;

        Ok(run)
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

    async fn find_leaderboard_by_room_id(
        &self,
        room_id: Uuid,
    ) -> Result<Vec<LeaderboardRecord>, RepositoryError> {
        let rows = sqlx::query_as::<_, LeaderboardRow>(
            r#"
            WITH run_records AS (
                SELECT
                    runs.run_id,
                    runs.user_id,
                    users.display_name,
                    runs.started_at,
                    CAST(
                        TIMESTAMPDIFF(
                            MICROSECOND,
                            runs.started_at,
                            runs.cleared_at
                        ) DIV 1000
                        AS SIGNED
                    ) AS elapsed_ms,
                    CAST(
                        COALESCE(
                            SUM(
                                CASE
                                    WHEN problems.submission_type = 'operation_sequence'
                                    THEN problem_progress.answer_attempt_count
                                    ELSE 0
                                END
                            ),
                            0
                        )
                        AS SIGNED
                    ) AS query_count,
                    runs.cleared_at
                FROM runs
                INNER JOIN users
                    ON users.user_id = runs.user_id
                LEFT JOIN problem_progress
                    ON problem_progress.run_id = runs.run_id
                LEFT JOIN problems
                    ON problems.problem_id = problem_progress.problem_id
                   AND problems.room_id = runs.room_id
                WHERE runs.room_id = ?
                  AND runs.status = 'cleared'
                GROUP BY
                    runs.run_id,
                    runs.user_id,
                    users.display_name,
                    runs.started_at,
                    runs.cleared_at
            ),
            user_best_candidates AS (
                SELECT
                    run_id,
                    user_id,
                    display_name,
                    started_at,
                    elapsed_ms,
                    query_count,
                    cleared_at,
                    ROW_NUMBER() OVER (
                        PARTITION BY user_id
                        ORDER BY
                            elapsed_ms,
                            query_count,
                            cleared_at,
                            run_id
                    ) AS user_position
                FROM run_records
            ),
            best_runs AS (
                SELECT
                    run_id,
                    user_id,
                    display_name,
                    started_at,
                    elapsed_ms,
                    query_count,
                    cleared_at
                FROM user_best_candidates
                WHERE user_position = 1
            ),
            ranked_best_runs AS (
                SELECT
                    user_id,
                    display_name,
                    started_at,
                    elapsed_ms,
                    query_count,
                    cleared_at,
                    CAST(
                        RANK() OVER (
                            ORDER BY
                                elapsed_ms,
                                query_count,
                                cleared_at
                        )
                        AS SIGNED
                    ) AS leaderboard_rank
                FROM best_runs
            )
            SELECT
                leaderboard_rank AS `rank`,
                user_id,
                display_name,
                started_at,
                query_count,
                cleared_at
            FROM ranked_best_runs
            ORDER BY
                elapsed_ms,
                query_count,
                cleared_at,
                user_id
            "#,
        )
        .bind(room_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::Database)?;

        rows.into_iter().map(LeaderboardRecord::try_from).collect()
    }

    async fn find_user_progress(
        &self,
        user_id: Uuid,
    ) -> Result<UserProgressRecord, RepositoryError> {
        let rows = sqlx::query_as::<_, GenreProgressRow>(
            r#"
            SELECT
                rooms.genre,
                CAST(COUNT(cleared_rooms.room_id) AS SIGNED)
                    AS cleared_room_count,
                CAST(COUNT(*) AS SIGNED)
                    AS total_room_count
            FROM rooms
            LEFT JOIN (
                SELECT DISTINCT room_id
                FROM runs
                WHERE user_id = ?
                  AND status = 'cleared'
            ) AS cleared_rooms
                ON cleared_rooms.room_id = rooms.room_id
            WHERE rooms.is_published = 1
            GROUP BY rooms.genre
            ORDER BY rooms.genre ASC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::Database)?;

        let by_genre = rows
            .into_iter()
            .map(GenreProgressRecord::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        let cleared_room_count = by_genre.iter().try_fold(0_u32, |total, progress| {
            total
                .checked_add(progress.cleared_room_count)
                .ok_or(RepositoryError::InvalidProgressCount)
        })?;

        let total_room_count = by_genre.iter().try_fold(0_u32, |total, progress| {
            total
                .checked_add(progress.total_room_count)
                .ok_or(RepositoryError::InvalidProgressCount)
        })?;

        Ok(UserProgressRecord {
            cleared_room_count,
            total_room_count,
            by_genre,
        })
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

    async fn find_hint_for_run(
        &self,
        run_id: Uuid,
        room_id: Uuid,
        problem_id: Uuid,
        level: i32,
    ) -> Result<Option<HintRecord>, RepositoryError> {
        let mut tx = self.pool.begin().await.map_err(RepositoryError::Database)?;

        let active_run_exists = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT 1
            FROM runs
            WHERE run_id = ? AND room_id = ? AND status = 'active'
            FOR UPDATE
            "#,
        )
        .bind(run_id)
        .bind(room_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(RepositoryError::Database)?;

        if active_run_exists.is_none() {
            return Err(RepositoryError::RunNotFound);
        }

        #[derive(FromRow)]
        struct ProblemHintRow {
            status: String,
            hints: sqlx::types::Json<Vec<crate::problem::Hint>>,
        }

        let row = sqlx::query_as::<_, ProblemHintRow>(
            r#"
            SELECT
                problem_progress.status,
                problems.hints
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
        .fetch_optional(&mut *tx)
        .await
        .map_err(RepositoryError::Database)?;

        let Some(row) = row else {
            return Ok(None);
        };

        if row.status == "locked" {
            return Err(RepositoryError::ProblemLocked);
        }

        let hint_index = match level.checked_sub(1) {
            Some(idx) if idx >= 0 => idx as usize,
            _ => return Ok(None),
        };

        let Some(hint) = row.hints.0.get(hint_index) else {
            return Ok(None);
        };

        sqlx::query(
            r#"
            UPDATE problem_progress
            SET max_hint_level = GREATEST(max_hint_level, ?)
            WHERE run_id = ? AND problem_id = ?
            "#,
        )
        .bind(level)
        .bind(run_id)
        .bind(problem_id)
        .execute(&mut *tx)
        .await
        .map_err(RepositoryError::Database)?;

        tx.commit().await.map_err(RepositoryError::Database)?;

        Ok(Some(HintRecord {
            level,
            body_markdown: hint.body_markdown.clone(),
        }))
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
