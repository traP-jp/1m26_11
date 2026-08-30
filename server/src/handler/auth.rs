use axum::{
    Json,
    extract::{State, rejection::JsonRejection},
    http::StatusCode,
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use openapi_generated::models::{GuestLoginRequest, GuestLoginResponse, User};
use uuid::Uuid;

use crate::{AppState, auth::demo::SESSION_COOKIE_NAME, config::AuthMode, error::AppError};

const DISPLAY_NAME_MAX_LENGTH: usize = 32;

fn normalize_display_name(display_name: &str) -> Result<&str, AppError> {
    let display_name = display_name.trim();

    if display_name.is_empty() {
        return Err(AppError::DisplayNameRequired);
    }

    if display_name.chars().count() > DISPLAY_NAME_MAX_LENGTH {
        return Err(AppError::DisplayNameTooLong);
    }

    Ok(display_name)
}

pub(crate) async fn login_guest(
    State(state): State<AppState>,
    mut jar: CookieJar,
    payload: Result<Json<GuestLoginRequest>, JsonRejection>,
) -> Result<(CookieJar, Json<GuestLoginResponse>), AppError> {
    if state.auth_mode != AuthMode::Demo {
        return Err(AppError::not_found("route not found"));
    }

    let Json(payload) = payload.map_err(|_| AppError::bad_request("invalid request body"))?;

    let display_name = normalize_display_name(&payload.display_name)?;
    let session_id = Uuid::new_v4();

    let user_record = state
        .auth_repository
        .get_or_create_demo_user_and_session(session_id, display_name, display_name)
        .await?;

    let cookie = Cookie::build((SESSION_COOKIE_NAME, session_id.to_string()))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(state.demo_cookie_secure)
        .build();

    jar = jar.add(cookie);

    let response = GuestLoginResponse::new(
        true,
        User::new(user_record.user_id, user_record.display_name),
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

    if let Some(session_cookie) = jar.get(SESSION_COOKIE_NAME) {
        if let Ok(session_id) = Uuid::parse_str(session_cookie.value()) {
            state
                .auth_repository
                .delete_demo_session(session_id)
                .await?;
        }
    }

    let cookie = Cookie::build(SESSION_COOKIE_NAME)
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(state.demo_cookie_secure)
        .build();
    jar = jar.remove(cookie);

    Ok((jar, StatusCode::NO_CONTENT))
}
