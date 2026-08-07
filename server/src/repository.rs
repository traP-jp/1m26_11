use async_trait::async_trait;
use sqlx::{FromRow, MySqlPool};
use thiserror::Error;
use uuid::Uuid;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserRecord {
    pub id: Uuid,
    pub name: String,
    pub email: String,
}

#[derive(FromRow)]
struct DatabaseUser {
    id: String,
    name: String,
    email: String,
}

impl TryFrom<DatabaseUser> for UserRecord {
    type Error = uuid::Error;

    fn try_from(user: DatabaseUser) -> Result<Self, Self::Error> {
        Ok(Self {
            id: Uuid::parse_str(&user.id)?,
            name: user.name,
            email: user.email,
        })
    }
}

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("user not found")]
    NotFound,
    #[error("database operation failed")]
    Database(#[source] sqlx::Error),
    #[error("database contains an invalid user ID")]
    InvalidUserId(#[source] uuid::Error),
}

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn get_users(&self) -> Result<Vec<UserRecord>, RepositoryError>;
    async fn create_user(&self, name: &str, email: &str) -> Result<Uuid, RepositoryError>;
    async fn get_user(&self, user_id: Uuid) -> Result<UserRecord, RepositoryError>;
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

#[async_trait]
impl UserRepository for SqlxUserRepository {
    async fn get_users(&self) -> Result<Vec<UserRecord>, RepositoryError> {
        sqlx::query_as::<_, DatabaseUser>("SELECT id, name, email FROM users ORDER BY id")
            .fetch_all(&self.pool)
            .await
            .map_err(RepositoryError::Database)?
            .into_iter()
            .map(UserRecord::try_from)
            .collect::<Result<_, _>>()
            .map_err(RepositoryError::InvalidUserId)
    }

    async fn create_user(&self, name: &str, email: &str) -> Result<Uuid, RepositoryError> {
        let user_id = Uuid::new_v4();
        sqlx::query("INSERT INTO users (id, name, email) VALUES (?, ?, ?)")
            .bind(user_id.to_string())
            .bind(name)
            .bind(email)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::Database)?;

        Ok(user_id)
    }

    async fn get_user(&self, user_id: Uuid) -> Result<UserRecord, RepositoryError> {
        let user = sqlx::query_as::<_, DatabaseUser>(
            "SELECT id, name, email FROM users WHERE id = ? LIMIT 1",
        )
        .bind(user_id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::Database)?
        .ok_or(RepositoryError::NotFound)?;

        UserRecord::try_from(user).map_err(RepositoryError::InvalidUserId)
    }
}
