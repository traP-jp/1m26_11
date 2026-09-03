use std::{sync::Arc, time::Duration};

use axum::{
    Router,
    extract::{DefaultBodyLimit, MatchedPath},
    routing::{get, post},
};
use config::AuthMode;
use problem::{AssetUrlResolver, UnconfiguredAssetUrlResolver};
use repository::AuthRepository;
use sqlx::MySqlPool;
use tower_http::trace::TraceLayer;
use tracing::Span;
use uuid::Uuid;

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
mod image_upload;
pub use image_upload::{
    ImageStorage, ImageStorageError, ImageStorageUpload, ImageUrlSigner, ImageUrlSigningError,
    S3ImageStorage,
};
pub mod problem;
pub mod repository;

pub const OPENAPI_DOCUMENT: &str = include_str!(concat!(env!("OUT_DIR"), "/openapi-v1.yaml"));

#[derive(Clone)]
pub struct AppState {
    pub(crate) auth_mode: AuthMode,
    pub(crate) demo_cookie_secure: bool,
    pub(crate) auth_repository: Arc<dyn AuthRepository>,
    pub(crate) asset_url_resolver: Arc<dyn AssetUrlResolver>,
    pub(crate) image_storage: Option<Arc<dyn ImageStorage>>,
    pub(crate) image_url_signer: Option<Arc<dyn ImageUrlSigner>>,
}

impl AppState {
    #[must_use]
    pub fn new(auth_mode: AuthMode, auth_repository: Arc<dyn AuthRepository>) -> Self {
        Self {
            auth_mode,
            demo_cookie_secure: true,
            auth_repository,
            asset_url_resolver: Arc::new(UnconfiguredAssetUrlResolver),
            image_storage: None,
            image_url_signer: None,
        }
    }

    #[must_use]
    pub fn with_demo_cookie_secure(mut self, secure: bool) -> Self {
        self.demo_cookie_secure = secure;
        self
    }

    #[must_use]
    pub fn with_asset_url_resolver(
        mut self,
        asset_url_resolver: Arc<dyn AssetUrlResolver>,
    ) -> Self {
        self.asset_url_resolver = asset_url_resolver;
        self
    }

    #[must_use]
    pub fn with_image_storage(mut self, image_storage: Arc<dyn ImageStorage>) -> Self {
        self.image_storage = Some(image_storage);
        self
    }

    #[must_use]
    pub fn with_image_url_signer(mut self, image_url_signer: Arc<dyn ImageUrlSigner>) -> Self {
        self.image_url_signer = Some(image_url_signer);
        self
    }
}

pub fn app(state: AppState) -> Router {
    let image_download_route_enabled = state.image_url_signer.is_some();
    let image_upload_route_enabled =
        state.auth_mode == AuthMode::Demo && state.image_storage.is_some();

    let mut router = Router::new()
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
            "/api/me/progress",
            get(handler::get_me_progress).fallback(handler::method_not_allowed),
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
            "/api/rooms",
            get(handler::get_rooms).fallback(handler::method_not_allowed),
        )
        .route(
            "/api/rooms/{room_id}/runs",
            post(handler::start_or_resume_run).fallback(handler::method_not_allowed),
        )
        .route(
            "/api/rooms/{room_id}/problems/{problem_id}",
            get(handler::get_problem).fallback(handler::method_not_allowed),
        )
        .route(
            "/api/rooms/{room_id}/problems/{problem_id}/hints/{level}",
            get(handler::get_problem_hint).fallback(handler::method_not_allowed),
        )
        .route(
            "/api/rooms/{room_id}/runs/current",
            get(handler::get_current_run).fallback(handler::method_not_allowed),
        )
        .route(
            "/api/rooms/{room_id}/leaderboard",
            get(handler::get_room_leaderboard).fallback(handler::method_not_allowed),
        )
        .route(
            "/api/rooms/{room_id}/problems/{problem_id}/queries",
            post(handler::submit_query).fallback(handler::method_not_allowed),
        )
        .route(
            "/api/rooms/{room_id}/problems/{problem_id}/answers",
            post(handler::submit_answer).fallback(handler::method_not_allowed),
        );
    if image_download_route_enabled && image_upload_route_enabled {
        router = router.route(
            "/api/rooms/{room_id}/problems/{problem_id}/assets",
            get(handler::get_problem_assets)
                .post(handler::upload_problem_asset)
                .fallback(handler::method_not_allowed)
                .layer(DefaultBodyLimit::max(
                    image_upload::MAX_IMAGE_FILE_BYTES + 64 * 1024,
                )),
        );
    } else if image_download_route_enabled {
        router = router.route(
            "/api/rooms/{room_id}/problems/{problem_id}/assets",
            get(handler::get_problem_assets).fallback(handler::method_not_allowed),
        );
    } else if image_upload_route_enabled {
        router = router.route(
            "/api/rooms/{room_id}/problems/{problem_id}/assets",
            post(handler::upload_problem_asset)
                .fallback(handler::method_not_allowed)
                .layer(DefaultBodyLimit::max(
                    image_upload::MAX_IMAGE_FILE_BYTES + 64 * 1024,
                )),
        );
    }

    router
        .fallback(handler::not_found)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &axum::http::Request<_>| {
                    let request_id = Uuid::new_v4();
                    let matched_route = request
                        .extensions()
                        .get::<MatchedPath>()
                        .map(MatchedPath::as_str)
                        .unwrap_or("<unmatched>");

                    tracing::info_span!(
                        "http_request",
                        request_id = %request_id,
                        method = %request.method(),
                        matched_route = %matched_route,
                    )
                })
                .on_response(
                    |response: &axum::http::Response<_>, duration: Duration, span: &Span| {
                        tracing::info!(
                            parent: span,
                            status = response.status().as_u16(),
                            duration_ms = duration.as_secs_f64() * 1_000.0,
                            "request completed"
                        );
                    },
                ),
        )
        .with_state(state)
}

pub async fn migrate(pool: &MySqlPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}
