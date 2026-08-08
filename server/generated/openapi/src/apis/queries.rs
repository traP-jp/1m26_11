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
pub enum SubmitQueryResponse {
    /// 操作列の判定結果。不正解もこの200 responseを使用します。
    Status200(models::QueryResponse),
    /// JSON構文不正またはpath parameterのUUID形式不正。具体的なerror.codeは未確定です。
    Status400_JSON(models::ErrorResponse),
    /// 未ログイン
    Status401(models::ErrorResponse),
    /// 対象resourceが存在しないか、現在のAUTH_MODEではendpointが有効ではありません。具体的なerror.codeは未確定です。
    Status404(models::ErrorResponse),
    /// 問題がまだ解放されていません。
    Status409(models::ErrorResponse),
    /// 文字数超過など入力内容が不正です。具体的なerror.codeは未確定です。
    Status422(models::ErrorResponse),
    /// server内部エラー。具体的なerror.codeは未確定です。
    Status500_Server(models::ErrorResponse),
}

/// Queries
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Queries<E: std::fmt::Debug + Send + Sync + 'static = ()>: super::ErrorHandler<E> {
    /// 絞り込み操作列を送信する.
    ///
    /// SubmitQuery - POST /api/rooms/{room_id}/problems/{problem_id}/queries
    async fn submit_query(
        &self,

        method: &Method,
        host: &Host,
        cookies: &CookieJar,
        path_params: &models::SubmitQueryPathParams,
        body: &models::QueryRequest,
    ) -> Result<SubmitQueryResponse, E>;
}
