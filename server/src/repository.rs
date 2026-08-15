use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, MySqlPool};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("database operation failed")]
    Database(#[source] sqlx::Error),
    #[error("user was not found after get-or-create")]
    UserNotFoundAfterUpsert,
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

#[derive(Clone, Debug, Eq, PartialEq, FromRow)]
pub struct AuthUserRecord {
    pub user_id: Uuid,
    pub display_name: String,
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

#[async_trait]
pub trait AuthRepository: Send + Sync {
    async fn find_user_by_demo_session(
        &self,
        session_id: Uuid,
    ) -> Result<Option<AuthUserRecord>, RepositoryError>;

    async fn find_user_by_provider_subject(
        &self,
        auth_provider: &str,
        provider_subject: &str,
    ) -> Result<Option<AuthUserRecord>, RepositoryError>;

    async fn get_or_create_user(
        &self,
        auth_provider: &str,
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
        sqlx::query_as::<_, AuthUserRecord>(
            r#"
            SELECT users.id AS user_id, users.display_name
            FROM demo_sessions
            INNER JOIN users ON users.id = demo_sessions.user_id
            WHERE demo_sessions.id = ?
              AND users.auth_provider = 'demo'
            LIMIT 1
            "#,
        )
        .bind(session_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::Database)
    }

    async fn find_user_by_provider_subject(
        &self,
        auth_provider: &str,
        provider_subject: &str,
    ) -> Result<Option<AuthUserRecord>, RepositoryError> {
        sqlx::query_as::<_, AuthUserRecord>(
            r#"
            SELECT id AS user_id, display_name
            FROM users
            WHERE auth_provider = ?
              AND provider_subject = ?
            LIMIT 1
            "#,
        )
        .bind(auth_provider)
        .bind(provider_subject)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::Database)
    }

    async fn get_or_create_user(
        &self,
        auth_provider: &str,
        provider_subject: &str,
        display_name: &str,
    ) -> Result<AuthUserRecord, RepositoryError> {
        let user_id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO users (
                id,
                auth_provider,
                provider_subject,
                display_name
            )
            VALUES (?, ?, ?, ?)
            ON DUPLICATE KEY UPDATE id = id
            "#,
        )
        .bind(user_id)
        .bind(auth_provider)
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
            INSERT INTO demo_sessions (id, user_id)
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
            WHERE id = ?
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
            SELECT id, number, name, genre, description, is_published, created_at
            FROM rooms
            WHERE id = ?
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
            SELECT id, user_id, room_id, status, started_at, cleared_at
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
            INSERT INTO runs (id, user_id, room_id, status, started_at, cleared_at)
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
            SELECT id, room_id, number, problem_type, title, body_markdown, submission_type,
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
            SELECT id, user_id, room_id, status, started_at, cleared_at
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
            SELECT id, room_id, number, problem_type, title, body_markdown, submission_type,
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
