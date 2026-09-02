use axum::{
    Json,
    extract::{Path, State},
};
use chrono::Utc;
use openapi_generated::{
    models::{
        ActiveRunResponse, BestRecord, ProblemHintResponse, ProblemResponse, Progress,
        ProgressStatus, RoomItem, RoomsResponse,
    },
    types::Nullable,
};
use uuid::Uuid;

use crate::{
    AppState,
    auth::current_user::{CurrentUser, OptionalCurrentUser},
    error::AppError,
    problem::build_problem_response,
    repository::RepositoryError,
};

pub(crate) async fn get_rooms(
    State(state): State<AppState>,
    OptionalCurrentUser(user): OptionalCurrentUser,
) -> Result<Json<RoomsResponse>, AppError> {
    let user_id = user.map(|u| u.user_id);
    let room_summaries = state
        .auth_repository
        .find_published_rooms_with_progress(user_id)
        .await?;

    let items = room_summaries
        .into_iter()
        .map(|summary| {
            let progress_status = match summary.progress_status.as_str() {
                "not_started" => ProgressStatus::NotStarted,
                "active" => ProgressStatus::Active,
                "cleared" => ProgressStatus::Cleared,
                other => {
                    return Err(RepositoryError::InvalidRunStatus {
                        status: format!("unexpected progress status: {other}"),
                    }
                    .into());
                }
            };

            let progress = Progress::new(
                progress_status,
                summary.cleared_count,
                summary.required_count,
            );

            let best_record = match summary.best_record {
                Some(record) => Nullable::Present(BestRecord::new(
                    record.elapsed_ms,
                    record.rank,
                    record.query_count,
                )),
                None => Nullable::Null,
            };

            Ok(RoomItem::new(
                summary.room_id,
                summary.number as u32,
                summary.name,
                summary.genre,
                summary.description,
                summary.problem_count,
                progress,
                best_record,
            ))
        })
        .collect::<Result<Vec<_>, AppError>>()?;

    Ok(Json(RoomsResponse::new(items)))
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

pub(crate) async fn get_problem(
    State(state): State<AppState>,
    user: CurrentUser,
    Path((room_id, problem_id)): Path<(String, String)>,
) -> Result<Json<ProblemResponse>, AppError> {
    let room_id =
        Uuid::parse_str(&room_id).map_err(|_| AppError::bad_request("invalid room_id"))?;

    let problem_id =
        Uuid::parse_str(&problem_id).map_err(|_| AppError::bad_request("invalid problem_id"))?;

    let run = state
        .auth_repository
        .find_active_run(user.user_id, room_id)
        .await?
        .ok_or(AppError::RunNotFound)?;

    let problem = state
        .auth_repository
        .find_problem_for_run(run.id, room_id, problem_id)
        .await?
        .ok_or_else(|| AppError::not_found("problem not found"))?;

    if problem.status == "locked" {
        return Err(AppError::ProblemLocked);
    }

    let response = build_problem_response(problem, state.asset_url_resolver.as_ref())?;

    Ok(Json(response))
}

pub(crate) async fn get_problem_hint(
    State(state): State<AppState>,
    user: CurrentUser,
    Path((room_id, problem_id, level)): Path<(String, String, String)>,
) -> Result<Json<ProblemHintResponse>, AppError> {
    let room_id =
        Uuid::parse_str(&room_id).map_err(|_| AppError::bad_request("invalid room_id"))?;

    let problem_id =
        Uuid::parse_str(&problem_id).map_err(|_| AppError::bad_request("invalid problem_id"))?;

    let level = level
        .parse::<i32>()
        .map_err(|_| AppError::bad_request("invalid hint level"))?;

    if level <= 0 {
        return Err(AppError::bad_request("invalid hint level"));
    }

    let run = state
        .auth_repository
        .find_active_run(user.user_id, room_id)
        .await?
        .ok_or(AppError::RunNotFound)?;

    let hint = state
        .auth_repository
        .find_hint_for_run(run.id, room_id, problem_id, level)
        .await?
        .ok_or_else(|| AppError::not_found("hint not found"))?;

    let response = ProblemHintResponse::new(hint.level, hint.body_markdown);

    Ok(Json(response))
}

pub(crate) async fn get_current_run(
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
        None => return Err(AppError::run_not_found()),
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
