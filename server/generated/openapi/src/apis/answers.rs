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
pub enum SubmitAnswerResponse {
    /// 文字列回答の判定結果。不正解もこの200 responseを使用します。
    Status200(models::AnswerResponse),
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

/// Answers
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Answers<E: std::fmt::Debug + Send + Sync + 'static = ()>: super::ErrorHandler<E> {
    /// 文字列回答を送信する.
    ///
    /// SubmitAnswer - POST /api/rooms/{room_id}/problems/{problem_id}/answers
    async fn submit_answer(
        &self,

        method: &Method,
        host: &Host,
        cookies: &CookieJar,
        path_params: &models::SubmitAnswerPathParams,
        body: &models::AnswerRequest,
    ) -> Result<SubmitAnswerResponse, E>;
}
