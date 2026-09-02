use axum::{
    Json,
    extract::{Path, State, rejection::JsonRejection},
};
use openapi_generated::models::{
    AnswerRequest, AnswerResponse, CorrectAnswerResponse, IncorrectAnswerResponse, Progress,
    ProgressStatus as ApiProgressStatus, RunStatus as ApiRunStatus,
};
use uuid::Uuid;

use crate::{
    AppState,
    auth::current_user::CurrentUser,
    error::AppError,
    repository::{AnswerRunStatus, AnswerSubmission, AnswerSubmissionResult},
};

pub(crate) async fn submit_answer(
    State(state): State<AppState>,
    user: CurrentUser,
    Path((room_id, problem_id)): Path<(String, String)>,
    payload: Result<Json<AnswerRequest>, JsonRejection>,
) -> Result<Json<AnswerResponse>, AppError> {
    let Json(payload) = payload.map_err(|_| AppError::bad_request("invalid request body"))?;

    let room_id =
        Uuid::parse_str(&room_id).map_err(|_| AppError::bad_request("invalid room_id"))?;

    let problem_id =
        Uuid::parse_str(&problem_id).map_err(|_| AppError::bad_request("invalid problem_id"))?;

    let run = state
        .auth_repository
        .find_active_run(user.user_id, room_id)
        .await?
        .ok_or(AppError::RunNotFound)?;

    let result = state
        .auth_repository
        .record_answer_judgement(AnswerSubmission {
            run_id: run.id,
            problem_id,
            answer: payload.answer,
        })
        .await?;

    let response = match result {
        AnswerSubmissionResult::Incorrect {
            answer_attempt_count,
        } => AnswerResponse::IncorrectAnswerResponse(IncorrectAnswerResponse::new(
            false,
            answer_attempt_count,
            "available".to_owned(),
            "active".to_owned(),
        )),

        AnswerSubmissionResult::Correct {
            unlocked_problem_ids,
            run_status,
            cleared_count,
            required_count,
            elapsed_ms,
        } => {
            let (api_run_status, api_progress_status) = match run_status {
                AnswerRunStatus::Active => (ApiRunStatus::Active, ApiProgressStatus::Active),
                AnswerRunStatus::Cleared => (ApiRunStatus::Cleared, ApiProgressStatus::Cleared),
            };

            let progress = Progress::new(api_progress_status, cleared_count, required_count);

            AnswerResponse::CorrectAnswerResponse(CorrectAnswerResponse::new(
                true,
                "cleared".to_owned(),
                unlocked_problem_ids,
                api_run_status,
                progress,
                elapsed_ms,
            ))
        }
    };

    Ok(Json(response))
}
