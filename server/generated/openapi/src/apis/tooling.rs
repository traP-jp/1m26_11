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
pub enum GetOpenApiResponse {
    /// このserver buildが使用するOpenAPI文書
    Status200
    (String)
}




/// Tooling
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Tooling<E: std::fmt::Debug + Send + Sync + 'static = ()>: super::ErrorHandler<E> {
    /// OpenAPI文書を取得する.
    ///
    /// GetOpenApi - GET /openapi.yaml
    async fn get_open_api(
    &self,
    
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
    ) -> Result<GetOpenApiResponse, E>;
}
