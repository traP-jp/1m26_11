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
pub enum GetMeResponse {
    /// 現在のログイン状態
    Status200(models::MeResponse),
    /// server内部エラー。具体的なerror.codeは未確定です。
    Status500_Server(models::ErrorResponse),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum LoginGuestResponse {
    /// ローカル用ユーザーを取得または作成し、session Cookieを発行した状態
    Status200 {
        body: models::GuestLoginResponse,
        set_cookie: Option<String>,
    },
    /// JSON構文不正またはpath parameterのUUID形式不正。具体的なerror.codeは未確定です。
    Status400_JSON(models::ErrorResponse),
    /// 対象resourceが存在しないか、現在のAUTH_MODEではendpointが有効ではありません。具体的なerror.codeは未確定です。
    Status404(models::ErrorResponse),
    /// 文字数超過など入力内容が不正です。具体的なerror.codeは未確定です。
    Status422(models::ErrorResponse),
    /// server内部エラー。具体的なerror.codeは未確定です。
    Status500_Server(models::ErrorResponse),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum LogoutLocalResponse {
    /// ログアウト完了。response bodyはありません。
    Status204 { set_cookie: Option<String> },
    /// server内部エラー。具体的なerror.codeは未確定です。
    Status500_Server(models::ErrorResponse),
}

/// Auth
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Auth<E: std::fmt::Debug + Send + Sync + 'static = ()>: super::ErrorHandler<E> {
    /// ログイン状態を取得する.
    ///
    /// GetMe - GET /api/me
    async fn get_me(
        &self,

        method: &Method,
        host: &Host,
        cookies: &CookieJar,
    ) -> Result<GetMeResponse, E>;

    /// ローカル環境で名前を登録する.
    ///
    /// LoginGuest - POST /api/auth/guest
    async fn login_guest(
        &self,

        method: &Method,
        host: &Host,
        cookies: &CookieJar,
        body: &models::GuestLoginRequest,
    ) -> Result<LoginGuestResponse, E>;

    /// ローカル環境からログアウトする.
    ///
    /// LogoutLocal - POST /api/auth/logout
    async fn logout_local(
        &self,

        method: &Method,
        host: &Host,
        cookies: &CookieJar,
    ) -> Result<LogoutLocalResponse, E>;
}
