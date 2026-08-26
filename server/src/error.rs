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

use crate::{problem::ProblemProjectionError, repository::RepositoryError};

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
    #[error("run not found")]
    RunNotFound,
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
        Self::internal(error)
    }
}

impl From<ProblemProjectionError> for AppError {
    fn from(error: ProblemProjectionError) -> Self {
        Self::internal(error)
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
            Self::RunNotFound => (
                StatusCode::NOT_FOUND,
                "RUN_NOT_FOUND",
                "挑戦中のrunが見つかりません".to_owned(),
            ),
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

    #[tokio::test]
    async fn unauthorized_response_matches_openapi_fixture() {
        let response = AppError::Unauthorized.into_response();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("response body should be readable")
            .to_bytes();

        let actual: serde_json::Value =
            serde_json::from_slice(&body).expect("response body should be valid JSON");

        let expected: serde_json::Value = serde_json::from_str(include_str!(
            "../../openapi/examples/auth/error-unauthorized.json"
        ))
        .expect("OpenAPI fixture should be valid JSON");

        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn run_not_found_response_matches_openapi_fixture() {
        let response = AppError::RunNotFound.into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("response body should be readable")
            .to_bytes();

        let actual: serde_json::Value =
            serde_json::from_slice(&body).expect("response body should be valid JSON");

        let expected: serde_json::Value = serde_json::from_str(include_str!(
            "../../openapi/examples/runs/error-run-not-found.json"
        ))
        .expect("OpenAPI fixture should be valid JSON");

        assert_eq!(actual, expected);
    }
}
