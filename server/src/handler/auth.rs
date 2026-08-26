use axum::{Json, extract::State, http::StatusCode};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use openapi_generated::models::{GuestLoginRequest, GuestLoginResponse, User};
use uuid::Uuid;

use crate::{AppState, config::AuthMode, error::AppError};

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
        .create_demo_session(session_id, user_record.user_id)
        .await?;

    let cookie = Cookie::build(("demo_session", session_id.to_string()))
        .path("/")
        .http_only(true)
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

    if let Some(session_cookie) = jar.get("demo_session") {
        if let Ok(session_id) = Uuid::parse_str(session_cookie.value()) {
            state
                .auth_repository
                .delete_demo_session(session_id)
                .await?;
        }
    }

    let cookie = Cookie::build("demo_session").path("/").build();
    jar = jar.remove(cookie);

    Ok((jar, StatusCode::NO_CONTENT))
}
