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
pub enum CreateProblemResponse {
    /// 問題の作成に成功しました。
    Status201(models::CreateProblemResponse),
    /// room_id、JSON、またはIdempotency-Keyの形式が不正です。 error.codeはINVALID_PATH_PARAMETER、INVALID_JSON、 IDEMPOTENCY_KEY_REQUIRED、INVALID_IDEMPOTENCY_KEYのいずれかです。
    Status400_Room(models::ErrorResponse),
    /// 指定されたroomが存在しません。 error.codeはROOM_NOT_FOUNDです。
    Status404(models::ErrorResponse),
    /// 公開済みroom、問題番号の重複、またはidempotencyの状態により作成できません。 error.codeはPUBLISHED_ROOM_IMMUTABLE、PROBLEM_NUMBER_CONFLICT、 IDEMPOTENCY_KEY_REUSEDのいずれかです。
    Status409(models::ErrorResponse),
    /// 問題内容、回答設定、依存関係、操作列、またはヒントが不正です。 error.codeはINVALID_PROBLEMです。
    Status422(models::ErrorResponse),
    /// DB更新失敗などのserver内部エラーです。 error.codeはINTERNAL_SERVER_ERRORです。
    Status500_DB(models::ErrorResponse),
}

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

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum GetProblemAssetsResponse {
    /// 対象problemに登録された画像と、300秒間有効なpresigned GET URL
    Status200 {
        body: models::ProblemAssetsResponse,
        cache_control: Option<String>,
    },
    /// room_idまたはproblem_idがUUIDではありません。 error.codeはINVALID_PATH_PARAMETERです。
    Status400_Room(models::ErrorResponse),
    /// 未ログイン
    Status401(models::ErrorResponse),
    /// activeなrunがないか、指定されたproblemに取得可能な画像がありません。 error.codeはRUN_NOT_FOUNDまたはIMAGE_NOT_FOUNDです。
    Status404_Active(models::ErrorResponse),
    /// 問題がまだ解放されていません。
    Status409(models::ErrorResponse),
    /// DBアクセスまたはpresigned URL生成に失敗しました。 error.codeはINTERNAL_SERVER_ERRORです。
    Status500_DB(models::ErrorResponse),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum GetProblemHintResponse {
    /// ヒント本文
    Status200(models::ProblemHintResponse),
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

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum GetProblemsResponse {
    /// 挑戦中の部屋の問題一覧
    Status200(models::ProblemsResponse),
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
pub enum UploadProblemAssetResponse {
    /// 画像をuploadし、対象problemへの紐付けが完了しました。
    Status201(models::Asset),
    /// upload requestの形式が不正です。 error.codeはINVALID_PATH_PARAMETER、INVALID_MULTIPART、 IDEMPOTENCY_KEY_REQUIRED、INVALID_IDEMPOTENCY_KEYのいずれかです。
    Status400_UploadRequest(models::ErrorResponse),
    /// roomまたはproblemが存在しないか、problemが指定されたroomに属していません。 error.codeはROOM_OR_PROBLEM_NOT_FOUNDです。
    Status404_Room(models::ErrorResponse),
    /// 公開済みroomまたはidempotencyの状態によりuploadできません。 error.codeはPUBLISHED_ROOM_IMMUTABLE、IDEMPOTENCY_KEY_REUSED、 IDEMPOTENCY_REQUEST_IN_PROGRESSのいずれかです。
    Status409(models::ErrorResponse),
    /// file sizeが5,242,880 bytesを超えています。 error.codeはIMAGE_TOO_LARGEです。
    Status413_FileSize(models::ErrorResponse),
    /// 実file内容がPNG、JPEG、WebPではありません。SVGも許可しません。 error.codeはUNSUPPORTED_IMAGE_TYPEです。
    Status415(models::ErrorResponse),
    /// file内容、画像寸法、またはtrim後のaltが不正です。 error.codeはEMPTY_FILE、INVALID_IMAGE、IMAGE_DIMENSIONS_EXCEEDED、 INVALID_ALTのいずれかです。
    Status422_File(models::ErrorResponse),
    /// DB更新失敗などのserver内部エラーです。 error.codeはINTERNAL_SERVER_ERRORです。
    Status500_DB(models::ErrorResponse),
    /// storage providerが4xx responseを返しました。 error.codeはSTORAGE_PROVIDER_ERRORです。
    Status502_StorageProvider(models::ErrorResponse),
    /// storage providerへの接続失敗、10秒のtimeout、またはproviderの5xxです。 error.codeはSTORAGE_UNAVAILABLEです。
    Status503_StorageProvider(models::ErrorResponse),
}

/// Problems
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Problems<E: std::fmt::Debug + Send + Sync + 'static = ()>:
    super::ErrorHandler<E>
{
    /// 作問用の問題を新規作成する.
    ///
    /// CreateProblem - POST /api/rooms/{room_id}/problems
    async fn create_problem(
        &self,

        method: &Method,
        host: &Host,
        cookies: &CookieJar,
        header_params: &models::CreateProblemHeaderParams,
        path_params: &models::CreateProblemPathParams,
        body: &models::CreateProblemRequest,
    ) -> Result<CreateProblemResponse, E>;

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

    /// 問題に登録された画像の取得URLを発行する.
    ///
    /// GetProblemAssets - GET /api/rooms/{room_id}/problems/{problem_id}/assets
    async fn get_problem_assets(
        &self,

        method: &Method,
        host: &Host,
        cookies: &CookieJar,
        path_params: &models::GetProblemAssetsPathParams,
    ) -> Result<GetProblemAssetsResponse, E>;

    /// 問題のヒントを取得する.
    ///
    /// GetProblemHint - GET /api/rooms/{room_id}/problems/{problem_id}/hints/{level}
    async fn get_problem_hint(
        &self,

        method: &Method,
        host: &Host,
        cookies: &CookieJar,
        path_params: &models::GetProblemHintPathParams,
    ) -> Result<GetProblemHintResponse, E>;

    /// 挑戦中の部屋の問題一覧を取得する.
    ///
    /// GetProblems - GET /api/rooms/{room_id}/problems
    async fn get_problems(
        &self,

        method: &Method,
        host: &Host,
        cookies: &CookieJar,
        path_params: &models::GetProblemsPathParams,
    ) -> Result<GetProblemsResponse, E>;

    /// 作問用画像をアップロードする.
    ///
    /// UploadProblemAsset - POST /api/rooms/{room_id}/problems/{problem_id}/assets
    async fn upload_problem_asset(
        &self,

        method: &Method,
        host: &Host,
        cookies: &CookieJar,
        header_params: &models::UploadProblemAssetHeaderParams,
        path_params: &models::UploadProblemAssetPathParams,
        body: Multipart,
    ) -> Result<UploadProblemAssetResponse, E>;
}
