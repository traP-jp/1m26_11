use axum::{
    Json,
    extract::State,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use openapi_generated::models::{GuestLoginRequest, GuestLoginResponse, User};
use uuid::Uuid;

use crate::{OPENAPI_DOCUMENT, AppState, config::AuthMode, error::AppError};

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

pub(crate) async fn login_guest(
    State(state): State<AppState>,
    mut jar: CookieJar,
    Json(payload): Json<GuestLoginRequest>,
) -> Result<(CookieJar, Json<GuestLoginResponse>), AppError> {
    if state.auth_mode != AuthMode::Demo {
        return Err(AppError::not_found("route not found"));
    }

    let user_record = state
        .auth_repository
        .get_or_create_user("demo", &payload.display_name, &payload.display_name)
        .await?;

    let session_id = Uuid::new_v4();
    state
        .auth_repository
        .create_demo_session(session_id, user_record.id)
        .await?;

    let cookie = Cookie::build(("demo_session", session_id.to_string()))
        .path("/")
        .http_only(true)
        .build();
    jar = jar.add(cookie);

    let response = GuestLoginResponse::new(
        true,
        User::new(user_record.id, user_record.display_name),
    );

    Ok((jar, Json(response)))
}

pub(crate) async fn logout_demo(
    State(state): State<AppState>,
    mut jar: CookieJar,
) -> Result<(CookieJar, StatusCode), AppError> {
    if state.auth_mode != AuthMode::Demo {
        return Err(AppError::not_found("route not found"));
    }

    if let Some(session_cookie) = jar.get("demo_session") {
        if let Ok(session_id) = Uuid::parse_str(session_cookie.value()) {
            state.auth_repository.delete_demo_session(session_id).await?;
        }
    }

    let cookie = Cookie::build("demo_session")
        .path("/")
        .build();
    jar = jar.remove(cookie);

    Ok((jar, StatusCode::NO_CONTENT))
}

