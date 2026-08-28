mod answers;
mod auth;
mod me;
mod queries;
mod rooms;

pub(crate) use answers::submit_answer;
pub(crate) use auth::{login_guest, logout_demo};
pub(crate) use me::get_me;
pub(crate) use queries::submit_query;
pub(crate) use rooms::{get_current_run, get_problem, get_problem_hint, start_or_resume_run};

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
