use axum::{
    Json,
    extract::{Path, State, rejection::JsonRejection, rejection::PathRejection},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::{
    AppState, OPENAPI_DOCUMENT,
    api::{CreateUserRequest, CreateUserResponse},
    error::AppError,
};

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

pub(crate) async fn get_users(State(state): State<AppState>) -> Result<Response, AppError> {
    let users = state.users.get_users().await.map_err(AppError::from)?;
    Ok(Json(users).into_response())
}

pub(crate) async fn create_user(
    State(state): State<AppState>,
    payload: Result<Json<CreateUserRequest>, JsonRejection>,
) -> Result<Response, AppError> {
    let Json(request) = payload.map_err(|rejection| {
        AppError::bad_request(format!("invalid request body: {}", rejection.body_text()))
    })?;
    request.validate().map_err(AppError::bad_request)?;

    let id = state
        .users
        .create_user(&request.name, &request.email)
        .await
        .map_err(AppError::from)?;

    Ok(Json(CreateUserResponse { id }).into_response())
}

pub(crate) async fn get_user(
    State(state): State<AppState>,
    path: Result<Path<Uuid>, PathRejection>,
) -> Result<Response, AppError> {
    let Path(user_id) = path.map_err(|rejection| {
        AppError::bad_request(format!("invalid userID: {}", rejection.body_text()))
    })?;
    let user = state
        .users
        .get_user(user_id)
        .await
        .map_err(AppError::from)?;

    Ok(Json(user).into_response())
}

pub(crate) async fn not_found() -> AppError {
    AppError::not_found("route not found")
}

pub(crate) async fn method_not_allowed() -> AppError {
    AppError::MethodNotAllowed
}
