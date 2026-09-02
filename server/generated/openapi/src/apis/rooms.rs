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
pub enum GetRoomsResponse {
    /// 公開部屋一覧の取得成功
    Status200(models::RoomsResponse),
    /// server内部エラー。具体的なerror.codeは未確定です。
    Status500_Server(models::ErrorResponse),
}

/// Rooms
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Rooms<E: std::fmt::Debug + Send + Sync + 'static = ()>: super::ErrorHandler<E> {
    /// 公開部屋一覧を取得する.
    ///
    /// GetRooms - GET /api/rooms
    async fn get_rooms(
        &self,

        method: &Method,
        host: &Host,
        cookies: &CookieJar,
    ) -> Result<GetRoomsResponse, E>;
}
