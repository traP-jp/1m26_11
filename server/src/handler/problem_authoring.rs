use axum::{
    Json,
    extract::{Path, State, rejection::JsonRejection},
    http::{HeaderMap, StatusCode},
};
use openapi_generated::models::{CreateProblemRequest, CreateProblemResponse};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    AppState,
    error::AppError,
    problem::{ProblemAuthoringError, validate_problem_draft},
    repository::{CreateProblemRecordOutcome, CreateProblemRecordRequest, RepositoryError},
};

use super::assets::parse_idempotency_key;

const REQUEST_METHOD: &str = "POST";

pub(crate) async fn create_problem(
    State(state): State<AppState>,
    Path(room_id): Path<String>,
    headers: HeaderMap,
    payload: Result<Json<CreateProblemRequest>, JsonRejection>,
) -> Result<(StatusCode, Json<CreateProblemResponse>), AppError> {
    let room_id = Uuid::parse_str(&room_id).map_err(|_| {
        AppError::api(
            StatusCode::BAD_REQUEST,
            "INVALID_PATH_PARAMETER",
            "room_id is invalid",
        )
    })?;

    let idempotency_key = parse_idempotency_key(&headers)?;

    let Json(payload) = payload.map_err(|_| {
        AppError::api(
            StatusCode::BAD_REQUEST,
            "INVALID_JSON",
            "request body is invalid",
        )
    })?;

    let draft = validate_problem_draft(room_id, payload).map_err(problem_validation_error)?;

    let serialized = serde_json::to_vec(&draft).map_err(AppError::internal)?;
    let payload_sha256: [u8; 32] = Sha256::digest(serialized).into();
    let request_path = format!("/api/rooms/{room_id}/problems");

    let outcome = state
        .auth_repository
        .create_problem(&CreateProblemRecordRequest {
            request_method: REQUEST_METHOD.to_owned(),
            request_path,
            idempotency_key,
            payload_sha256,
            draft,
        })
        .await
        .map_err(problem_repository_error)?;

    let problem_id = match outcome {
        CreateProblemRecordOutcome::Created { problem_id }
        | CreateProblemRecordOutcome::Replayed { problem_id } => problem_id,
        CreateProblemRecordOutcome::Reused => {
            return Err(AppError::api(
                StatusCode::CONFLICT,
                "IDEMPOTENCY_KEY_REUSED",
                "Idempotency-Key was reused with different request data",
            ));
        }
    };

    Ok((
        StatusCode::CREATED,
        Json(CreateProblemResponse::new(problem_id)),
    ))
}

fn problem_validation_error(_error: ProblemAuthoringError) -> AppError {
    AppError::api(
        StatusCode::UNPROCESSABLE_ENTITY,
        "INVALID_PROBLEM",
        "problem data is invalid",
    )
}

fn problem_repository_error(error: RepositoryError) -> AppError {
    match error {
        RepositoryError::RoomNotFound => AppError::api(
            StatusCode::NOT_FOUND,
            "ROOM_NOT_FOUND",
            "room was not found",
        ),
        RepositoryError::PublishedRoomImmutable => AppError::api(
            StatusCode::CONFLICT,
            "PUBLISHED_ROOM_IMMUTABLE",
            "published room cannot be modified",
        ),
        RepositoryError::ProblemNumberConflict => AppError::api(
            StatusCode::CONFLICT,
            "PROBLEM_NUMBER_CONFLICT",
            "problem number is already used in the room",
        ),
        RepositoryError::InvalidProblemDependency => AppError::api(
            StatusCode::UNPROCESSABLE_ENTITY,
            "INVALID_PROBLEM",
            "problem dependency is invalid",
        ),
        error => AppError::internal(error),
    }
}
