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
pub enum GetCurrentRunResponse {
    /// 現在のactiveな挑戦状態
    Status200(models::ActiveRunResponse),
    /// JSON構文不正またはpath parameterのUUID形式不正。具体的なerror.codeは未確定です。
    Status400_JSON(models::ErrorResponse),
    /// 未ログイン
    Status401(models::ErrorResponse),
    /// activeなrunが存在しません。
    Status404_Active(models::ErrorResponse),
    /// server内部エラー。具体的なerror.codeは未確定です。
    Status500_Server(models::ErrorResponse),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum StartOrResumeRunResponse {
    /// activeな挑戦状態
    Status200_Active(models::ActiveRunResponse),
    /// JSON構文不正またはpath parameterのUUID形式不正。具体的なerror.codeは未確定です。
    Status400_JSON(models::ErrorResponse),
    /// 未ログイン
    Status401(models::ErrorResponse),
    /// 対象resourceが存在しないか、現在のAUTH_MODEではendpointが有効ではありません。具体的なerror.codeは未確定です。
    Status404(models::ErrorResponse),
    /// server内部エラー。具体的なerror.codeは未確定です。
    Status500_Server(models::ErrorResponse),
}

/// Runs
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Runs<E: std::fmt::Debug + Send + Sync + 'static = ()>: super::ErrorHandler<E> {
    /// 現在の挑戦状態を取得する.
    ///
    /// GetCurrentRun - GET /api/rooms/{room_id}/runs/current
    async fn get_current_run(
        &self,

        method: &Method,
        host: &Host,
        cookies: &CookieJar,
        path_params: &models::GetCurrentRunPathParams,
    ) -> Result<GetCurrentRunResponse, E>;

    /// 挑戦を開始または再開する.
    ///
    /// StartOrResumeRun - POST /api/rooms/{room_id}/runs
    async fn start_or_resume_run(
        &self,

        method: &Method,
        host: &Host,
        cookies: &CookieJar,
        path_params: &models::StartOrResumeRunPathParams,
    ) -> Result<StartOrResumeRunResponse, E>;
}
