use std::error::Error;

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

use openapi_generated::{
    models::{ErrorResponse, ErrorResponseError},
    types::Object,
};

use crate::{
    problem::{ProblemProjectionError, QueryJudgeError},
    repository::RepositoryError,
};

type BoxError = Box<dyn Error + Send + Sync>;

#[derive(Debug, Error)]
pub(crate) enum AppError {
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "returned by CurrentUser before protected game API handlers are added"
        )
    )]
    #[error("unauthorized")]
    Unauthorized,
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("{0}")]
    NotFound(String),
    #[error("method not allowed")]
    MethodNotAllowed,
    #[error("internal server error")]
    Internal {
        #[source]
        source: BoxError,
    },
    #[error("active run was not found")]
    RunNotFound,
    #[error("problem is locked")]
    ProblemLocked,
    #[error("problem is already cleared")]
    ProblemAlreadyCleared,
    #[error("query validation failed")]
    ValidationError,
}

impl AppError {
    pub(crate) fn bad_request(message: impl Into<String>) -> Self {
        Self::BadRequest(message.into())
    }

    pub(crate) fn conflict(message: impl Into<String>) -> Self {
        Self::Conflict(message.into())
    }

    pub(crate) fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound(message.into())
    }

    pub(crate) fn run_not_found() -> Self {
        Self::RunNotFound
    }

    fn internal(error: impl Error + Send + Sync + 'static) -> Self {
        Self::Internal {
            source: Box::new(error),
        }
    }
}

impl From<RepositoryError> for AppError {
    fn from(error: RepositoryError) -> Self {
        match error {
            RepositoryError::RunNotFound => Self::RunNotFound,
            RepositoryError::ProblemNotFound => Self::not_found("problem not found"),
            RepositoryError::ProblemLocked => Self::ProblemLocked,
            RepositoryError::ProblemAlreadyCleared => Self::ProblemAlreadyCleared,
            RepositoryError::EmptyAnswer
            | RepositoryError::AnswerLengthExceeded
            | RepositoryError::WrongAnswerSubmissionType => Self::ValidationError,
            error => Self::internal(error),
        }
    }
}

impl From<ProblemProjectionError> for AppError {
    fn from(error: ProblemProjectionError) -> Self {
        Self::internal(error)
    }
}

impl From<QueryJudgeError> for AppError {
    fn from(error: QueryJudgeError) -> Self {
        match error {
            error @ QueryJudgeError::InvalidStoredJudgeConfig => Self::internal(error),
            QueryJudgeError::InvalidSource
            | QueryJudgeError::EmptyOperations
            | QueryJudgeError::NonPositiveCount
            | QueryJudgeError::UnknownControl
            | QueryJudgeError::OperationLimitExceeded
            | QueryJudgeError::WrongSubmissionType => Self::ValidationError,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            Self::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                "ログインが必要です".to_owned(),
            ),
            Self::BadRequest(message) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", message.clone()),
            Self::Conflict(message) => (StatusCode::CONFLICT, "CONFLICT", message.clone()),
            Self::NotFound(message) => (StatusCode::NOT_FOUND, "NOT_FOUND", message.clone()),
            Self::MethodNotAllowed => (
                StatusCode::METHOD_NOT_ALLOWED,
                "METHOD_NOT_ALLOWED",
                "method not allowed".to_owned(),
            ),
            Self::Internal { source } => {
                tracing::error!(error = %source, "request failed");

                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_SERVER_ERROR",
                    "internal server error".to_owned(),
                )
            }
            Self::RunNotFound => (
                StatusCode::NOT_FOUND,
                "RUN_NOT_FOUND",
                "挑戦中のrunが見つかりません".to_owned(),
            ),
            Self::ProblemLocked => (
                StatusCode::CONFLICT,
                "PROBLEM_LOCKED",
                "この問題はまだ解放されていません".to_owned(),
            ),
            Self::ProblemAlreadyCleared => (
                StatusCode::CONFLICT,
                "PROBLEM_ALREADY_CLEARED",
                "この問題はすでにクリア済みですわ".to_owned(),
            ),
            Self::ValidationError => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "VALIDATION_ERROR",
                "入力内容が正しくありませんわ".to_owned(),
            ),
        };

        let body = ErrorResponse::new(ErrorResponseError::new(
            code.to_owned(),
            message,
            Object(serde_json::json!({})),
        ));

        (status, Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use axum::{http::StatusCode, response::IntoResponse};
    use http_body_util::BodyExt;

    use super::AppError;

    async fn assert_response_matches_fixture(
        error: AppError,
        expected_status: StatusCode,
        expected_fixture: &str,
    ) {
        let response = error.into_response();

        assert_eq!(response.status(), expected_status);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("response body should be readable")
            .to_bytes();

        let actual: serde_json::Value =
            serde_json::from_slice(&body).expect("response body should be valid JSON");

        let expected: serde_json::Value =
            serde_json::from_str(expected_fixture).expect("OpenAPI fixture should be valid JSON");

        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn unauthorized_response_matches_openapi_fixture() {
        assert_response_matches_fixture(
            AppError::Unauthorized,
            StatusCode::UNAUTHORIZED,
            include_str!("../../openapi/examples/auth/error-unauthorized.json"),
        )
        .await;
    }

    #[tokio::test]
    async fn run_not_found_response_matches_openapi_fixture() {
        assert_response_matches_fixture(
            AppError::RunNotFound,
            StatusCode::NOT_FOUND,
            include_str!("../../openapi/examples/runs/error-run-not-found.json"),
        )
        .await;
    }

    #[tokio::test]
    async fn problem_already_cleared_response_matches_openapi_fixture() {
        assert_response_matches_fixture(
            AppError::ProblemAlreadyCleared,
            StatusCode::CONFLICT,
            include_str!("../../openapi/examples/problems/error-problem-already-cleared.json"),
        )
        .await;
    }

    #[tokio::test]
    async fn validation_error_response_matches_openapi_fixture() {
        assert_response_matches_fixture(
            AppError::ValidationError,
            StatusCode::UNPROCESSABLE_ENTITY,
            include_str!("../../openapi/examples/queries/error-validation.json"),
        )
        .await;
    }
}
