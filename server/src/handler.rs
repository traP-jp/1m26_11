mod answers;
mod assets;
mod auth;
mod leaderboard;
mod me;
mod progress;
mod queries;
mod rooms;

pub(crate) use answers::submit_answer;
pub(crate) use assets::upload_problem_asset;
pub(crate) use auth::{login_guest, logout_demo};
pub(crate) use leaderboard::get_room_leaderboard;
pub(crate) use me::get_me;
pub(crate) use progress::get_me_progress;
pub(crate) use queries::submit_query;
pub(crate) use rooms::{
    get_current_run, get_problem, get_problem_hint, get_rooms, start_or_resume_run,
};

use axum::{
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};

use crate::{OPENAPI_DOCUMENT, error::AppError};

pub(crate) async fn ping() -> Response {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        "pong",
    )
        .into_response()
}

pub(crate) async fn openapi() -> Response {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/yaml")],
        OPENAPI_DOCUMENT,
    )
        .into_response()
}

pub(crate) async fn not_found() -> AppError {
    AppError::not_found("route not found")
}

pub(crate) async fn method_not_allowed() -> AppError {
    AppError::MethodNotAllowed
}
