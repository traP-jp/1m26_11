use std::sync::Arc;

use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;
use url::Url;
use uuid::Uuid;

use crate::{
    api::User,
    repository::{RepositoryError, UserRepository},
};

pub const DEFAULT_PHOTO_API_URL: &str = "https://jsonplaceholder.typicode.com/photos";
const USER_ICON_PHOTO_ID: u32 = 1;

#[async_trait]
pub trait PhotoProvider: Send + Sync {
    async fn thumbnail_url(&self, photo_id: u32) -> Result<String, PhotoError>;
}

#[derive(Clone)]
pub struct ReqwestPhotoClient {
    client: Client,
    endpoint: Url,
}

impl ReqwestPhotoClient {
    pub fn new(client: Client, endpoint: &str) -> Result<Self, PhotoClientBuildError> {
        let mut endpoint = Url::parse(endpoint)?;
        if !matches!(endpoint.scheme(), "http" | "https") {
            return Err(PhotoClientBuildError::UnsupportedScheme(
                endpoint.scheme().to_owned(),
            ));
        }

        let path = format!("{}/", endpoint.path().trim_end_matches('/'));
        endpoint.set_path(&path);
        endpoint.set_query(None);
        endpoint.set_fragment(None);

        Ok(Self { client, endpoint })
    }
}

#[async_trait]
impl PhotoProvider for ReqwestPhotoClient {
    async fn thumbnail_url(&self, photo_id: u32) -> Result<String, PhotoError> {
        let url = self.endpoint.join(&photo_id.to_string())?;
        let photo = self
            .client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json::<Photo>()
            .await?;

        Ok(photo.thumbnail_url)
    }
}

#[derive(Debug, Error)]
pub enum PhotoClientBuildError {
    #[error("PHOTO_API_URL is invalid")]
    InvalidUrl(#[from] url::ParseError),
    #[error("PHOTO_API_URL uses unsupported scheme `{0}`")]
    UnsupportedScheme(String),
}

#[derive(Debug, Error)]
pub enum PhotoError {
    #[error("failed to construct photo API request URL")]
    InvalidUrl(#[from] url::ParseError),
    #[error("photo API request failed")]
    Request(#[from] reqwest::Error),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Photo {
    thumbnail_url: String,
}

pub struct UserService {
    repository: Arc<dyn UserRepository>,
    photos: Arc<dyn PhotoProvider>,
}

impl UserService {
    #[must_use]
    pub fn new(repository: Arc<dyn UserRepository>, photos: Arc<dyn PhotoProvider>) -> Self {
        Self { repository, photos }
    }

    pub async fn get_users(&self) -> Result<Vec<User>, ServiceError> {
        let users = self
            .repository
            .get_users()
            .await
            .map_err(ServiceError::Repository)?;
        let icon_url = self
            .photos
            .thumbnail_url(USER_ICON_PHOTO_ID)
            .await
            .map_err(ServiceError::Photo)?;

        Ok(users
            .into_iter()
            .map(|user| User {
                id: user.id,
                name: user.name,
                email: user.email,
                icon_url: icon_url.clone(),
            })
            .collect())
    }

    pub async fn create_user(&self, name: &str, email: &str) -> Result<Uuid, ServiceError> {
        self.repository
            .create_user(name, email)
            .await
            .map_err(ServiceError::Repository)
    }

    pub async fn get_user(&self, user_id: Uuid) -> Result<User, ServiceError> {
        let user = match self.repository.get_user(user_id).await {
            Ok(user) => user,
            Err(RepositoryError::NotFound) => return Err(ServiceError::UserNotFound),
            Err(error) => return Err(ServiceError::Repository(error)),
        };
        let icon_url = self
            .photos
            .thumbnail_url(USER_ICON_PHOTO_ID)
            .await
            .map_err(ServiceError::Photo)?;

        Ok(User {
            id: user.id,
            name: user.name,
            email: user.email,
            icon_url,
        })
    }
}

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("user not found")]
    UserNotFound,
    #[error("repository failed")]
    Repository(#[source] RepositoryError),
    #[error("photo service failed")]
    Photo(#[source] PhotoError),
}
