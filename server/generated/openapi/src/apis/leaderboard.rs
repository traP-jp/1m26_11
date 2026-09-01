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
pub enum GetRoomLeaderboardResponse {
    /// 部屋別leaderboard
    Status200(models::LeaderboardResponse),
    /// JSON構文不正またはpath parameterのUUID形式不正。具体的なerror.codeは未確定です。
    Status400_JSON(models::ErrorResponse),
    /// 対象resourceが存在しないか、現在のAUTH_MODEではendpointが有効ではありません。具体的なerror.codeは未確定です。
    Status404(models::ErrorResponse),
    /// server内部エラー。具体的なerror.codeは未確定です。
    Status500_Server(models::ErrorResponse),
}

/// Leaderboard
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Leaderboard<E: std::fmt::Debug + Send + Sync + 'static = ()>:
    super::ErrorHandler<E>
{
    /// 部屋別leaderboardを取得する.
    ///
    /// GetRoomLeaderboard - GET /api/rooms/{room_id}/leaderboard
    async fn get_room_leaderboard(
        &self,

        method: &Method,
        host: &Host,
        cookies: &CookieJar,
        path_params: &models::GetRoomLeaderboardPathParams,
    ) -> Result<GetRoomLeaderboardResponse, E>;
}
