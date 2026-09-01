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
pub enum GetMeProgressResponse {
    /// ログイン中のユーザーの公開room全体に対する進捗
    Status200(models::MeProgressResponse),
    /// 未ログイン
    Status401(models::ErrorResponse),
    /// server内部エラー。具体的なerror.codeは未確定です。
    Status500_Server(models::ErrorResponse),
}

/// Progress
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Progress<E: std::fmt::Debug + Send + Sync + 'static = ()>:
    super::ErrorHandler<E>
{
    /// ログイン中のユーザーの全体進捗を取得する.
    ///
    /// GetMeProgress - GET /api/me/progress
    async fn get_me_progress(
        &self,

        method: &Method,
        host: &Host,
        cookies: &CookieJar,
    ) -> Result<GetMeProgressResponse, E>;
}
