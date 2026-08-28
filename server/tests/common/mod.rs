#![allow(dead_code)]

use std::{
    env,
    str::FromStr,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use axum::{Router, body::Body, http::Request};
use chrono::{DateTime, Utc};
use http_body_util::BodyExt;
use serde_json::json;
use server::{
    AppState, app,
    config::AuthMode,
    problem::{Asset, AssetUrlResolveError, AssetUrlResolver, InputSchema},
    repository::{
        AnswerRunStatus, AnswerSubmission, AnswerSubmissionResult, AuthRepository, AuthUserRecord,
        HintRecord, ProblemDetailRecord, QuerySubmission, QuerySubmissionResult, RepositoryError,
        RoomRecord, RunRecord,
    },
};
use sqlx::{
    MySqlPool,
    mysql::{MySqlConnectOptions, MySqlPoolOptions},
    types::Json,
};
use tower::ServiceExt;
use uuid::Uuid;

pub const MOCK_SESSION_ID: &str = "55555555-5555-4555-8555-555555555555";
pub const MOCK_RESUME_ROOM_ID: &str = "11111111-1111-4111-8111-111111111111";
pub const MOCK_NEW_ROOM_ID: &str = "33333333-3333-4333-8333-333333333333";
pub const MOCK_CLEARED_ROOM_ID: &str = "44444444-4444-4444-8444-444444444444";
pub const MOCK_CLEARED_PROBLEM_ID: &str = "22222222-2222-4222-8222-222222222221";
pub const MOCK_LOCKED_PROBLEM_ID: &str = "22222222-2222-4222-8222-222222222222";
pub const MOCK_CLEARED_DETAIL_PROBLEM_ID: &str = "22222222-2222-4222-8222-222222222223";
pub const MOCK_DATABASE_ERROR_PROBLEM_ID: &str = "22222222-2222-4222-8222-222222222224";
pub const MOCK_STRING_PROBLEM_ID: &str = "22222222-2222-4222-8222-222222222225";

pub fn problem_detail_record(id: Uuid, status: &str) -> ProblemDetailRecord {
    ProblemDetailRecord {
        id,
        number: 1,
        problem_type: "small".to_owned(),
        title: "生年月日".to_owned(),
        body_markdown: "問題文です".to_owned(),
        submission_type: "operation_sequence".to_owned(),
        assets: Json(vec![Asset {
            asset_type: "image".to_owned(),
            object_key: "private/problem-assets/birthday.png".to_owned(),
            alt: "問題資料".to_owned(),
        }]),
        input_schema: Json(
            serde_json::from_value::<InputSchema>(json!({
                "query": {
                    "type": "operation_sequence",
                    "allowed_controls": ["down", "right", "up"],
                    "max_operations": 100
                },
                "answer": {
                    "type": "string",
                    "max_length": 50
                }
            }))
            .expect("problem input schema should be valid"),
        ),
        judge_config: Json(json!({
            "type": "operation_sequence",
            "correct_operations": [
                {
                    "control": "down",
                    "count": 16
                },
                {
                    "control": "right",
                    "count": 2
                },
                {
                    "control": "up",
                    "count": 1
                }
            ],
            "candidates": [
                {
                    "candidate_id": "correct",
                    "operations": [
                        {
                            "control": "down",
                            "count": 16
                        },
                        {
                            "control": "right",
                            "count": 2
                        },
                        {
                            "control": "up",
                            "count": 1
                        }
                    ]
                }
            ]
        })),
        status: status.to_owned(),
        hint_count: 2,
    }
}

pub struct StubAssetUrlResolver;

impl AssetUrlResolver for StubAssetUrlResolver {
    fn resolve(&self, object_key: &str) -> Result<String, AssetUrlResolveError> {
        assert_eq!(
            object_key, "private/problem-assets/birthday.png",
            "expected object key should be passed to the resolver",
        );

        Ok("/assets/problems/birthday.png".to_owned())
    }
}

pub struct StubAuthRepository;

#[async_trait]
impl AuthRepository for StubAuthRepository {
    async fn find_user_by_demo_session(
        &self,
        _session_id: Uuid,
    ) -> Result<Option<AuthUserRecord>, RepositoryError> {
        Ok(Some(AuthUserRecord {
            user_id: Uuid::from_str(MOCK_SESSION_ID).unwrap(),
            display_name: "test-user".to_owned(),
        }))
    }

    async fn find_user_by_provider_subject(
        &self,
        _auth_provider: &str,
        _provider_subject: &str,
    ) -> Result<Option<AuthUserRecord>, RepositoryError> {
        Ok(None)
    }

    async fn get_or_create_user(
        &self,
        _auth_provider: &str,
        _provider_subject: &str,
        display_name: &str,
    ) -> Result<AuthUserRecord, RepositoryError> {
        Ok(AuthUserRecord {
            user_id: Uuid::new_v4(),
            display_name: display_name.to_owned(),
        })
    }

    async fn create_demo_session(
        &self,
        _session_id: Uuid,
        _user_id: Uuid,
    ) -> Result<(), RepositoryError> {
        Ok(())
    }

    async fn delete_demo_session(&self, _session_id: Uuid) -> Result<(), RepositoryError> {
        Ok(())
    }

    async fn find_room_by_id(&self, room_id: Uuid) -> Result<Option<RoomRecord>, RepositoryError> {
        if room_id == Uuid::nil() {
            Ok(None)
        } else {
            Ok(Some(RoomRecord {
                id: room_id,
                number: 1,
                name: "Test Room".to_owned(),
                genre: "Test".to_owned(),
                description: "Test description".to_owned(),
                is_published: true,
                created_at: Utc::now(),
            }))
        }
    }

    async fn find_active_run(
        &self,
        _user_id: Uuid,
        _room_id: Uuid,
    ) -> Result<Option<RunRecord>, RepositoryError> {
        let resume_room_id = Uuid::from_str(MOCK_RESUME_ROOM_ID).unwrap();
        if _room_id == resume_room_id {
            Ok(Some(RunRecord {
                id: resume_room_id,
                user_id: _user_id,
                room_id: _room_id,
                status: "active".to_owned(),
                started_at: Utc::now() - chrono::Duration::seconds(65),
                cleared_at: None,
            }))
        } else {
            Ok(None)
        }
    }

    async fn create_run(
        &self,
        id: Uuid,
        user_id: Uuid,
        room_id: Uuid,
        started_at: DateTime<Utc>,
    ) -> Result<RunRecord, RepositoryError> {
        Ok(RunRecord {
            id,
            user_id,
            room_id,
            status: "active".to_owned(),
            started_at,
            cleared_at: None,
        })
    }

    async fn find_cleared_run(
        &self,
        user_id: Uuid,
        room_id: Uuid,
    ) -> Result<Option<RunRecord>, RepositoryError> {
        let cleared_room_id = Uuid::from_str(MOCK_CLEARED_ROOM_ID).unwrap();
        if room_id == cleared_room_id {
            Ok(Some(RunRecord {
                id: Uuid::new_v4(),
                user_id,
                room_id,
                status: "cleared".to_owned(),
                started_at: Utc::now() - chrono::Duration::seconds(100),
                cleared_at: Some(Utc::now()),
            }))
        } else {
            Ok(None)
        }
    }

    async fn find_cleared_problem_ids(&self, run_id: Uuid) -> Result<Vec<Uuid>, RepositoryError> {
        let resume_room_id = Uuid::from_str(MOCK_RESUME_ROOM_ID).unwrap();
        if run_id == resume_room_id {
            Ok(vec![Uuid::from_str(MOCK_CLEARED_PROBLEM_ID).unwrap()])
        } else {
            Ok(vec![])
        }
    }

    async fn find_problem_for_run(
        &self,
        run_id: Uuid,
        room_id: Uuid,
        problem_id: Uuid,
    ) -> Result<Option<ProblemDetailRecord>, RepositoryError> {
        let active_run_id = Uuid::from_str(MOCK_RESUME_ROOM_ID).unwrap();

        if run_id != active_run_id || room_id != active_run_id {
            return Ok(None);
        }

        let available_id = Uuid::from_str(MOCK_CLEARED_PROBLEM_ID).unwrap();
        let locked_id = Uuid::from_str(MOCK_LOCKED_PROBLEM_ID).unwrap();
        let cleared_id = Uuid::from_str(MOCK_CLEARED_DETAIL_PROBLEM_ID).unwrap();
        let database_error_id = Uuid::from_str(MOCK_DATABASE_ERROR_PROBLEM_ID).unwrap();

        if problem_id == database_error_id {
            return Err(RepositoryError::Database(sqlx::Error::Protocol(
                "simulated private database failure".to_owned(),
            )));
        }

        if problem_id == available_id {
            Ok(Some(problem_detail_record(problem_id, "available")))
        } else if problem_id == locked_id {
            Ok(Some(problem_detail_record(problem_id, "locked")))
        } else if problem_id == cleared_id {
            Ok(Some(problem_detail_record(problem_id, "cleared")))
        } else {
            Ok(None)
        }
    }

    async fn find_hint_for_run(
        &self,
        run_id: Uuid,
        room_id: Uuid,
        problem_id: Uuid,
        level: i32,
    ) -> Result<Option<HintRecord>, RepositoryError> {
        let active_run_id = Uuid::from_str(MOCK_RESUME_ROOM_ID).unwrap();

        if run_id != active_run_id || room_id != active_run_id {
            return Ok(None);
        }

        let available_id = Uuid::from_str(MOCK_CLEARED_PROBLEM_ID).unwrap();
        let locked_id = Uuid::from_str(MOCK_LOCKED_PROBLEM_ID).unwrap();
        let cleared_id = Uuid::from_str(MOCK_CLEARED_DETAIL_PROBLEM_ID).unwrap();
        let database_error_id = Uuid::from_str(MOCK_DATABASE_ERROR_PROBLEM_ID).unwrap();

        if problem_id == database_error_id {
            return Err(RepositoryError::Database(sqlx::Error::Protocol(
                "simulated private database failure".to_owned(),
            )));
        }

        if problem_id == locked_id {
            return Err(RepositoryError::ProblemLocked);
        }

        if problem_id == available_id || problem_id == cleared_id {
            if level == 1 {
                Ok(Some(HintRecord {
                    level: 1,
                    body_markdown: "最初の操作に注目してください".to_owned(),
                }))
            } else if level == 2 {
                Ok(Some(HintRecord {
                    level: 2,
                    body_markdown: "2番目のヒントです".to_owned(),
                }))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
    async fn record_query_judgement(
        &self,
        submission: QuerySubmission,
    ) -> Result<QuerySubmissionResult, RepositoryError> {
        if submission.is_correct {
            Ok(QuerySubmissionResult {
                query_count: 5,
                problem_status: "cleared".to_owned(),
            })
        } else {
            Ok(QuerySubmissionResult {
                query_count: 4,
                problem_status: "available".to_owned(),
            })
        }
    }
    async fn record_answer_judgement(
        &self,
        submission: AnswerSubmission,
    ) -> Result<AnswerSubmissionResult, RepositoryError> {
        let active_run_id = Uuid::from_str(MOCK_RESUME_ROOM_ID).unwrap();
        let string_problem_id = Uuid::from_str(MOCK_STRING_PROBLEM_ID).unwrap();
        let operation_problem_id = Uuid::from_str(MOCK_CLEARED_PROBLEM_ID).unwrap();
        let locked_problem_id = Uuid::from_str(MOCK_LOCKED_PROBLEM_ID).unwrap();
        let cleared_problem_id = Uuid::from_str(MOCK_CLEARED_DETAIL_PROBLEM_ID).unwrap();
        let database_error_problem_id = Uuid::from_str(MOCK_DATABASE_ERROR_PROBLEM_ID).unwrap();

        if submission.run_id != active_run_id {
            return Err(RepositoryError::RunNotFound);
        }

        if submission.problem_id == locked_problem_id {
            return Err(RepositoryError::ProblemLocked);
        }

        if submission.problem_id == cleared_problem_id {
            return Err(RepositoryError::ProblemAlreadyCleared);
        }

        if submission.problem_id == database_error_problem_id {
            return Err(RepositoryError::Database(sqlx::Error::Protocol(
                "simulated private database failure".to_owned(),
            )));
        }

        if submission.problem_id == operation_problem_id {
            return Err(RepositoryError::WrongAnswerSubmissionType);
        }

        if submission.problem_id != string_problem_id {
            return Err(RepositoryError::ProblemNotFound);
        }

        if submission.answer.chars().count() > 50 {
            return Err(RepositoryError::AnswerLengthExceeded);
        }

        if submission.answer.trim().is_empty() {
            return Err(RepositoryError::EmptyAnswer);
        }

        if submission.answer == "19520715" {
            Ok(AnswerSubmissionResult::Correct {
                unlocked_problem_ids: vec![locked_problem_id],
                run_status: AnswerRunStatus::Active,
                cleared_problem_count: 1,
                total_problem_count: 4,
                elapsed_ms: 48_321,
            })
        } else {
            Ok(AnswerSubmissionResult::Incorrect {
                answer_attempt_count: 2,
            })
        }
    }
}

#[derive(Default)]
pub struct DemoSessionCalls {
    pub created: Vec<(Uuid, Uuid)>,
    pub deleted: Vec<Uuid>,
}

pub struct RecordingAuthRepository {
    pub user_id: Uuid,
    pub demo_session_calls: Mutex<DemoSessionCalls>,
}

impl RecordingAuthRepository {
    pub fn new(user_id: Uuid) -> Self {
        Self {
            user_id,
            demo_session_calls: Mutex::new(DemoSessionCalls::default()),
        }
    }
}

#[async_trait]
impl AuthRepository for RecordingAuthRepository {
    async fn find_user_by_demo_session(
        &self,
        _session_id: Uuid,
    ) -> Result<Option<AuthUserRecord>, RepositoryError> {
        Ok(None)
    }

    async fn find_user_by_provider_subject(
        &self,
        _auth_provider: &str,
        _provider_subject: &str,
    ) -> Result<Option<AuthUserRecord>, RepositoryError> {
        Ok(None)
    }

    async fn get_or_create_user(
        &self,
        _auth_provider: &str,
        _provider_subject: &str,
        display_name: &str,
    ) -> Result<AuthUserRecord, RepositoryError> {
        Ok(AuthUserRecord {
            user_id: self.user_id,
            display_name: display_name.to_owned(),
        })
    }

    async fn create_demo_session(
        &self,
        session_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), RepositoryError> {
        self.demo_session_calls
            .lock()
            .expect("demo session call log should not be poisoned")
            .created
            .push((session_id, user_id));

        Ok(())
    }

    async fn delete_demo_session(&self, session_id: Uuid) -> Result<(), RepositoryError> {
        self.demo_session_calls
            .lock()
            .expect("demo session call log should not be poisoned")
            .deleted
            .push(session_id);

        Ok(())
    }
}

pub fn test_app() -> Router {
    app(AppState::new(AuthMode::Demo, Arc::new(StubAuthRepository)))
}

pub fn problem_test_app() -> Router {
    app(AppState::new(AuthMode::Demo, Arc::new(StubAuthRepository))
        .with_asset_url_resolver(Arc::new(StubAssetUrlResolver)))
}

pub async fn request(app: &Router, request: Request<Body>) -> axum::response::Response {
    app.clone().oneshot(request).await.unwrap()
}

pub async fn body_json<T: serde::de::DeserializeOwned>(response: axum::response::Response) -> T {
    serde_json::from_slice(&body_bytes(response).await).unwrap()
}

pub async fn body_bytes(response: axum::response::Response) -> Vec<u8> {
    response
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes()
        .to_vec()
}

pub async fn connect_test_database() -> MySqlPool {
    let database_url = env::var("TEST_DATABASE_URL")
        .expect("TEST_DATABASE_URL must point to a disposable test database");

    let options =
        MySqlConnectOptions::from_str(&database_url).expect("TEST_DATABASE_URL should be valid");

    MySqlPoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .expect("test database should be reachable")
}

pub async fn primary_key_columns(pool: &MySqlPool, table_name: &str) -> Vec<String> {
    sqlx::query_scalar(
        r#"
        SELECT column_name
        FROM information_schema.key_column_usage
        WHERE table_schema = DATABASE()
          AND table_name = ?
          AND constraint_name = 'PRIMARY'
        ORDER BY ordinal_position
        "#,
    )
    .bind(table_name)
    .fetch_all(pool)
    .await
    .expect("primary key columns should be readable")
}

pub async fn index_columns(pool: &MySqlPool, table_name: &str, index_name: &str) -> Vec<String> {
    sqlx::query_scalar(
        r#"
        SELECT column_name
        FROM information_schema.statistics
        WHERE table_schema = DATABASE()
          AND table_name = ?
          AND index_name = ?
        ORDER BY seq_in_index
        "#,
    )
    .bind(table_name)
    .bind(index_name)
    .fetch_all(pool)
    .await
    .expect("index columns should be readable")
}

pub async fn foreign_key_delete_rule(pool: &MySqlPool, constraint_name: &str) -> String {
    sqlx::query_scalar(
        r#"
        SELECT delete_rule
        FROM information_schema.referential_constraints
        WHERE constraint_schema = DATABASE()
          AND constraint_name = ?
        "#,
    )
    .bind(constraint_name)
    .fetch_one(pool)
    .await
    .expect("foreign key delete rule should be readable")
}
