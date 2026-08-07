use std::sync::Arc;

use axum::{Router, routing::get};
use sqlx::MySqlPool;
use tower_http::trace::TraceLayer;

use service::UserService;

pub mod api;
pub mod config;
mod error;
mod handler;
pub mod repository;
pub mod service;

pub const OPENAPI_DOCUMENT: &str = include_str!(concat!(env!("OUT_DIR"), "/openapi-v1.yaml"));

#[derive(Clone)]
pub struct AppState {
    pub(crate) users: Arc<UserService>,
}

impl AppState {
    #[must_use]
    pub fn new(
        repository: Arc<dyn repository::UserRepository>,
        photos: Arc<dyn service::PhotoProvider>,
    ) -> Self {
        Self {
            users: Arc::new(UserService::new(repository, photos)),
        }
    }
}

pub fn app(state: AppState) -> Router {
    Router::new()
        .route(
            "/api/v1/ping",
            get(handler::ping).fallback(handler::method_not_allowed),
        )
        .route(
            "/openapi.yaml",
            get(handler::openapi).fallback(handler::method_not_allowed),
        )
        .route(
            "/api/v1/users",
            get(handler::get_users)
                .post(handler::create_user)
                .fallback(handler::method_not_allowed),
        )
        .route(
            "/api/v1/users/{user_id}",
            get(handler::get_user).fallback(handler::method_not_allowed),
        )
        .fallback(handler::not_found)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

pub async fn migrate(pool: &MySqlPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}
