mod me;

pub(crate) use me::get_me;

use axum::{
    Json,
    extract::{Path, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use chrono::Utc;
use openapi_generated::models::{ActiveRunResponse, GuestLoginRequest, GuestLoginResponse, User};
use uuid::Uuid;

use crate::{
    AppState, OPENAPI_DOCUMENT, auth::current_user::CurrentUser, config::AuthMode, error::AppError,
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

pub(crate) async fn start_or_resume_run(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(room_id): Path<String>,
) -> Result<Json<ActiveRunResponse>, AppError> {
    let room_id =
        Uuid::parse_str(&room_id).map_err(|_| AppError::bad_request("invalid room_id"))?;

    let room = state.auth_repository.find_room_by_id(room_id).await?;
    if room.is_none() {
        return Err(AppError::not_found("room not found"));
    }

    let run = state
        .auth_repository
        .find_active_run(user.user_id, room_id)
        .await?;

    let run_record = match run {
        Some(active_run) => active_run,
        None => {
            let cleared_run = state
                .auth_repository
                .find_cleared_run(user.user_id, room_id)
                .await?;
            if cleared_run.is_some() {
                return Err(AppError::conflict("room already cleared"));
            }

            let new_run_id = Uuid::new_v4();
            let started_at = Utc::now();
            state
                .auth_repository
                .create_run(new_run_id, user.user_id, room_id, started_at)
                .await?
        }
    };

    let cleared_problem_ids = state
        .auth_repository
        .find_cleared_problem_ids(run_record.id)
        .await?;

    let elapsed: chrono::Duration = Utc::now() - run_record.started_at;
    let elapsed_ms = elapsed.num_milliseconds().max(0) as u64;

    let response = ActiveRunResponse::new(
        "active".to_owned(),
        run_record.started_at,
        elapsed_ms,
        cleared_problem_ids,
    );

    Ok(Json(response))
}
