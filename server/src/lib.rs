use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};
use config::AuthMode;
use repository::AuthRepository;
use sqlx::MySqlPool;
use tower_http::trace::TraceLayer;

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "authentication extractors will be used by game API handlers"
    )
)]
pub(crate) mod auth;
pub mod config;
mod error;
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "query and answer handlers will use the shared clear flow"
    )
)]
pub(crate) mod game_progress;
mod handler;
pub mod problem;
pub mod repository;

pub const OPENAPI_DOCUMENT: &str = include_str!(concat!(env!("OUT_DIR"), "/openapi-v1.yaml"));

#[derive(Clone)]
pub struct AppState {
    pub(crate) auth_mode: AuthMode,
    pub(crate) auth_repository: Arc<dyn AuthRepository>,
}

impl AppState {
    #[must_use]
    pub fn new(auth_mode: AuthMode, auth_repository: Arc<dyn AuthRepository>) -> Self {
        Self {
            auth_mode,
            auth_repository,
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
            "/api/me",
            get(handler::get_me).fallback(handler::method_not_allowed),
        )
        .route(
            "/api/auth/guest",
            post(handler::login_guest).fallback(handler::method_not_allowed),
        )
        .route(
            "/api/auth/logout",
            post(handler::logout_demo).fallback(handler::method_not_allowed),
        )
        .route(
            "/api/rooms/{room_id}/runs",
            post(handler::start_or_resume_run).fallback(handler::method_not_allowed),
        )
        .fallback(handler::not_found)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

pub async fn migrate(pool: &MySqlPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}
