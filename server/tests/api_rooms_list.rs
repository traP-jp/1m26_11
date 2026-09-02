mod common;

use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use common::{MOCK_RESUME_ROOM_ID, MOCK_SESSION_ID, body_bytes, body_json, request, test_app};
use server::{
    AppState, app,
    config::AuthMode,
    repository::{
        AuthProvider, AuthRepository, AuthUserRecord, RepositoryError, RoomBestRecordRecord,
        RoomSummaryRecord,
    },
};
use uuid::Uuid;

#[tokio::test]
async fn rooms_unauthenticated_matches_openapi_fixture() {
    let app = test_app();

    let req = Request::get("/api/rooms").body(Body::empty()).unwrap();

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");

    let actual: serde_json::Value = body_json(response).await;
    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../openapi/examples/rooms/response-unauthenticated.json"
    ))
    .expect("OpenAPI response fixture should be valid JSON");

    assert_eq!(actual, expected);
}

#[tokio::test]
async fn rooms_authenticated_active_matches_openapi_fixture() {
    let app = test_app();

    let req = Request::get("/api/rooms")
        .header(header::COOKIE, format!("demo_session={MOCK_SESSION_ID}"))
        .body(Body::empty())
        .unwrap();

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");

    let actual: serde_json::Value = body_json(response).await;
    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../openapi/examples/rooms/response-authenticated-active.json"
    ))
    .expect("OpenAPI response fixture should be valid JSON");

    assert_eq!(actual, expected);
}

#[tokio::test]
async fn rooms_authenticated_cleared_matches_openapi_fixture() {
    struct ClearedUserAuthRepo;

    #[async_trait]
    impl AuthRepository for ClearedUserAuthRepo {
        async fn find_user_by_demo_session(
            &self,
            _session_id: Uuid,
        ) -> Result<Option<AuthUserRecord>, RepositoryError> {
            Ok(Some(AuthUserRecord {
                user_id: Uuid::parse_str("99999999-9999-4999-8999-999999999999").unwrap(),
                auth_provider: AuthProvider::Demo,
                display_name: "cleared-user".to_owned(),
            }))
        }

        async fn find_user_by_provider_subject(
            &self,
            _auth_provider: AuthProvider,
            _provider_subject: &str,
        ) -> Result<Option<AuthUserRecord>, RepositoryError> {
            Ok(None)
        }

        async fn get_or_create_user(
            &self,
            auth_provider: AuthProvider,
            _provider_subject: &str,
            display_name: &str,
        ) -> Result<AuthUserRecord, RepositoryError> {
            Ok(AuthUserRecord {
                user_id: Uuid::parse_str("99999999-9999-4999-8999-999999999999").unwrap(),
                auth_provider,
                display_name: display_name.to_owned(),
            })
        }

        async fn find_published_rooms_with_progress(
            &self,
            _user_id: Option<Uuid>,
        ) -> Result<Vec<RoomSummaryRecord>, RepositoryError> {
            Ok(vec![RoomSummaryRecord {
                room_id: Uuid::parse_str(MOCK_RESUME_ROOM_ID).unwrap(),
                number: 12,
                name: "general".to_owned(),
                genre: "OSINT".to_owned(),
                description: "人物を特定して脱出せよ".to_owned(),
                problem_count: 5,
                progress_status: "cleared".to_owned(),
                cleared_count: 4,
                required_count: 4,
                best_record: Some(RoomBestRecordRecord {
                    elapsed_ms: 119820,
                    rank: 14,
                    query_count: 37,
                }),
            }])
        }
    }

    let app = app(AppState::new(AuthMode::Demo, Arc::new(ClearedUserAuthRepo)));

    let req = Request::get("/api/rooms")
        .header(header::COOKIE, format!("demo_session={MOCK_SESSION_ID}"))
        .body(Body::empty())
        .unwrap();

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");

    let actual: serde_json::Value = body_json(response).await;
    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../openapi/examples/rooms/response-authenticated-cleared.json"
    ))
    .expect("OpenAPI response fixture should be valid JSON");

    assert_eq!(actual, expected);
}

#[tokio::test]
async fn rooms_empty_matches_openapi_fixture() {
    struct EmptyRoomsRepo;

    #[async_trait]
    impl AuthRepository for EmptyRoomsRepo {
        async fn find_user_by_demo_session(
            &self,
            _session_id: Uuid,
        ) -> Result<Option<AuthUserRecord>, RepositoryError> {
            Ok(None)
        }

        async fn find_user_by_provider_subject(
            &self,
            _auth_provider: AuthProvider,
            _provider_subject: &str,
        ) -> Result<Option<AuthUserRecord>, RepositoryError> {
            Ok(None)
        }

        async fn get_or_create_user(
            &self,
            auth_provider: AuthProvider,
            _provider_subject: &str,
            display_name: &str,
        ) -> Result<AuthUserRecord, RepositoryError> {
            Ok(AuthUserRecord {
                user_id: Uuid::new_v4(),
                auth_provider,
                display_name: display_name.to_owned(),
            })
        }

        async fn find_published_rooms_with_progress(
            &self,
            _user_id: Option<Uuid>,
        ) -> Result<Vec<RoomSummaryRecord>, RepositoryError> {
            Ok(vec![])
        }
    }

    let app = app(AppState::new(AuthMode::Demo, Arc::new(EmptyRoomsRepo)));

    let req = Request::get("/api/rooms").body(Body::empty()).unwrap();

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");

    let actual: serde_json::Value = body_json(response).await;
    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../openapi/examples/rooms/response-empty.json"
    ))
    .expect("OpenAPI response fixture should be valid JSON");

    assert_eq!(actual, expected);
}

#[tokio::test]
async fn rooms_database_error_returns_500_without_details() {
    struct ErrorRoomsRepo;

    #[async_trait]
    impl AuthRepository for ErrorRoomsRepo {
        async fn find_user_by_demo_session(
            &self,
            _session_id: Uuid,
        ) -> Result<Option<AuthUserRecord>, RepositoryError> {
            Ok(None)
        }

        async fn find_user_by_provider_subject(
            &self,
            _auth_provider: AuthProvider,
            _provider_subject: &str,
        ) -> Result<Option<AuthUserRecord>, RepositoryError> {
            Ok(None)
        }

        async fn get_or_create_user(
            &self,
            auth_provider: AuthProvider,
            _provider_subject: &str,
            display_name: &str,
        ) -> Result<AuthUserRecord, RepositoryError> {
            Ok(AuthUserRecord {
                user_id: Uuid::new_v4(),
                auth_provider,
                display_name: display_name.to_owned(),
            })
        }

        async fn find_published_rooms_with_progress(
            &self,
            _user_id: Option<Uuid>,
        ) -> Result<Vec<RoomSummaryRecord>, RepositoryError> {
            Err(RepositoryError::Database(sqlx::Error::PoolClosed))
        }
    }

    let app = app(AppState::new(AuthMode::Demo, Arc::new(ErrorRoomsRepo)));

    let req = Request::get("/api/rooms").body(Body::empty()).unwrap();

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let body = body_bytes(response).await;
    let body_text = std::str::from_utf8(&body).expect("response body should be UTF-8");

    assert!(
        !body_text.contains("PoolClosed"),
        "database error details must not be exposed"
    );

    let body: serde_json::Value =
        serde_json::from_slice(&body).expect("response body should be valid JSON");

    assert_eq!(body["error"]["code"], "INTERNAL_SERVER_ERROR");
    assert_eq!(body["error"]["message"], "internal server error");
}

#[tokio::test]
async fn rooms_invalid_progress_status_returns_500() {
    struct InvalidStatusRoomsRepo;

    #[async_trait]
    impl AuthRepository for InvalidStatusRoomsRepo {
        async fn find_user_by_demo_session(
            &self,
            _session_id: Uuid,
        ) -> Result<Option<AuthUserRecord>, RepositoryError> {
            Ok(None)
        }

        async fn find_user_by_provider_subject(
            &self,
            _auth_provider: AuthProvider,
            _provider_subject: &str,
        ) -> Result<Option<AuthUserRecord>, RepositoryError> {
            Ok(None)
        }

        async fn get_or_create_user(
            &self,
            auth_provider: AuthProvider,
            _provider_subject: &str,
            display_name: &str,
        ) -> Result<AuthUserRecord, RepositoryError> {
            Ok(AuthUserRecord {
                user_id: Uuid::new_v4(),
                auth_provider,
                display_name: display_name.to_owned(),
            })
        }

        async fn find_published_rooms_with_progress(
            &self,
            _user_id: Option<Uuid>,
        ) -> Result<Vec<RoomSummaryRecord>, RepositoryError> {
            Ok(vec![RoomSummaryRecord {
                room_id: Uuid::new_v4(),
                number: 1,
                name: "test".to_owned(),
                genre: "test".to_owned(),
                description: "test".to_owned(),
                problem_count: 1,
                progress_status: "invalid_status".to_owned(),
                cleared_count: 0,
                required_count: 1,
                best_record: None,
            }])
        }
    }

    let app = app(AppState::new(
        AuthMode::Demo,
        Arc::new(InvalidStatusRoomsRepo),
    ));

    let req = Request::get("/api/rooms").body(Body::empty()).unwrap();

    let response = request(&app, req).await;

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}
