use async_trait::async_trait;
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
    pub id: Uuid,
    pub display_name: String,
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
}

#[async_trait]
impl AuthRepository for SqlxUserRepository {
    async fn find_user_by_demo_session(
        &self,
        session_id: Uuid,
    ) -> Result<Option<AuthUserRecord>, RepositoryError> {
        sqlx::query_as::<_, AuthUserRecord>(
            r#"
            SELECT users.id, users.display_name
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
            SELECT id, display_name
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
}
