use async_trait::async_trait;
use axum::extract::*;
use axum_extra::extract::CookieJar;
use bytes::Bytes;
use headers::Host;
use http::Method;
use serde::{Deserialize, Serialize};

use crate::{models, types::*};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum GetProblemResponse {
    /// 公開可能な問題データ
    Status200(models::ProblemResponse),
    /// JSON構文不正またはpath parameterのUUID形式不正。具体的なerror.codeは未確定です。
    Status400_JSON(models::ErrorResponse),
    /// 未ログイン
    Status401(models::ErrorResponse),
    /// 対象resourceが存在しないか、現在のAUTH_MODEではendpointが有効ではありません。具体的なerror.codeは未確定です。
    Status404(models::ErrorResponse),
    /// 問題がまだ解放されていません。
    Status409(models::ErrorResponse),
    /// server内部エラー。具体的なerror.codeは未確定です。
    Status500_Server(models::ErrorResponse),
}

/// Problems
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Problems<E: std::fmt::Debug + Send + Sync + 'static = ()>:
    super::ErrorHandler<E>
{
    /// 問題データを取得する.
    ///
    /// GetProblem - GET /api/rooms/{room_id}/problems/{problem_id}
    async fn get_problem(
        &self,

        method: &Method,
        host: &Host,
        cookies: &CookieJar,
        path_params: &models::GetProblemPathParams,
    ) -> Result<GetProblemResponse, E>;
}
