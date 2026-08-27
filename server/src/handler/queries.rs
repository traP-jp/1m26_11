use axum::{
    Json,
    extract::{Path, State, rejection::JsonRejection},
};
use openapi_generated::models::{
    CorrectQueryResponse, IncorrectQueryResponse, Operation as ApiOperation, QueryRequest,
    QueryResponse,
};
use uuid::Uuid;

use crate::{
    AppState,
    auth::current_user::CurrentUser,
    error::AppError,
    problem::{Operation, decode_stored_judge_config, judge_query},
    repository::QuerySubmission,
};

pub(crate) async fn submit_query(
    State(state): State<AppState>,
    user: CurrentUser,
    Path((room_id, problem_id)): Path<(String, String)>,
    payload: Result<Json<QueryRequest>, JsonRejection>,
) -> Result<Json<QueryResponse>, AppError> {
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

    let problem = state
        .auth_repository
        .find_problem_for_run(run.id, room_id, problem_id)
        .await?
        .ok_or_else(|| AppError::not_found("problem not found"))?;

    if problem.status == "locked" {
        return Err(AppError::ProblemLocked);
    }

    if problem.status == "cleared" {
        return Err(AppError::ProblemAlreadyCleared);
    }

    let QueryRequest {
        source,
        operations: request_operations,
    } = payload;

    let operations = request_operations
        .into_iter()
        .map(|operation| Operation {
            control: operation.control,
            count: operation.count,
        })
        .collect::<Vec<_>>();

    let judge_config = decode_stored_judge_config(
        &problem.submission_type,
        &problem.judge_config.0,
        &problem.input_schema.0,
    )?;

    let judgement = judge_query(&source, &operations, &problem.input_schema.0, &judge_config)?;

    let query_id = Uuid::new_v4();
    let correct = judgement.correct;
    let remaining_pattern_count = judgement.remaining_pattern_count;
    let normalized_operations = judgement.normalized_operations;

    let result = state
        .auth_repository
        .record_query_judgement(QuerySubmission {
            query_id,
            run_id: run.id,
            problem_id,
            source,
            operations,
            normalized_operations: normalized_operations.clone(),
            remaining_pattern_count,
            is_correct: correct,
        })
        .await?;

    let normalized_operations = normalized_operations
        .into_iter()
        .map(|operation| ApiOperation::new(operation.control, operation.count))
        .collect();

    let response = if correct {
        QueryResponse::CorrectQueryResponse(CorrectQueryResponse::new(
            query_id,
            true,
            normalized_operations,
            remaining_pattern_count,
            result.query_count,
            result.problem_status,
        ))
    } else {
        QueryResponse::IncorrectQueryResponse(IncorrectQueryResponse::new(
            query_id,
            false,
            normalized_operations,
            remaining_pattern_count,
            result.query_count,
            result.problem_status,
        ))
    };

    Ok(Json(response))
}
