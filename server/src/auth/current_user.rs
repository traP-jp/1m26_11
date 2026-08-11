use axum::{extract::FromRequestParts, http::request::Parts};
use uuid::Uuid;

use crate::{AppState, config::AuthMode, error::AppError, repository::AuthUserRecord};

use super::{demo, neoshowcase};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CurrentUser {
    pub(crate) id: Uuid,
    pub(crate) display_name: String,
}

impl From<AuthUserRecord> for CurrentUser {
    fn from(user: AuthUserRecord) -> Self {
        Self {
            id: user.id,
            display_name: user.display_name,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct OptionalCurrentUser(pub(crate) Option<CurrentUser>);

impl FromRequestParts<AppState> for OptionalCurrentUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let user = match state.auth_mode {
            AuthMode::Demo => {
                demo::resolve_current_user(&parts.headers, state.auth_repository.as_ref()).await?
            }
            AuthMode::NeoShowcase => {
                neoshowcase::resolve_current_user(&parts.headers, state.auth_repository.as_ref())
                    .await?
            }
        };

        Ok(Self(user))
    }
}

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let OptionalCurrentUser(user) =
            OptionalCurrentUser::from_request_parts(parts, state).await?;

        user.ok_or(AppError::Unauthorized)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use axum::{
        extract::FromRequestParts,
        http::{Request, header},
    };
    use uuid::Uuid;

    use crate::{
        AppState,
        config::AuthMode,
        error::AppError,
        repository::{AuthRepository, AuthUserRecord, RepositoryError},
    };

    use super::{CurrentUser, OptionalCurrentUser};

    enum ExpectedAuth {
        Demo {
            session_id: Uuid,
            user: AuthUserRecord,
        },
        NeoShowcase {
            provider_subject: String,
            user: AuthUserRecord,
        },
        NoRepositoryCall,
    }

    struct StubAuthRepository {
        expected: ExpectedAuth,
    }

    #[async_trait]
    impl AuthRepository for StubAuthRepository {
        async fn find_user_by_demo_session(
            &self,
            session_id: Uuid,
        ) -> Result<Option<AuthUserRecord>, RepositoryError> {
            match &self.expected {
                ExpectedAuth::Demo {
                    session_id: expected_session_id,
                    user,
                } => {
                    assert_eq!(session_id, *expected_session_id);
                    Ok(Some(user.clone()))
                }
                _ => panic!("demo session lookup was not expected"),
            }
        }

        async fn find_user_by_provider_subject(
            &self,
            _auth_provider: &str,
            _provider_subject: &str,
        ) -> Result<Option<AuthUserRecord>, RepositoryError> {
            panic!("direct provider lookup was not expected");
        }

        async fn get_or_create_user(
            &self,
            auth_provider: &str,
            provider_subject: &str,
            display_name: &str,
        ) -> Result<AuthUserRecord, RepositoryError> {
            match &self.expected {
                ExpectedAuth::NeoShowcase {
                    provider_subject: expected_subject,
                    user,
                } => {
                    assert_eq!(auth_provider, "neoshowcase");
                    assert_eq!(provider_subject, expected_subject.as_str());
                    assert_eq!(display_name, expected_subject.as_str());

                    Ok(user.clone())
                }
                _ => {
                    panic!("NeoShowcase lookup was not expected")
                }
            }
        }
    }

    #[tokio::test]
    async fn optional_user_uses_demo_cookie_in_demo_mode() {
        let session_id = Uuid::new_v4();
        let auth_user = AuthUserRecord {
            id: Uuid::new_v4(),
            display_name: "demo-user".to_owned(),
        };
        let state = AppState::new(
            AuthMode::Demo,
            Arc::new(StubAuthRepository {
                expected: ExpectedAuth::Demo {
                    session_id,
                    user: auth_user.clone(),
                },
            }),
        );

        let request = Request::builder()
            .header(header::COOKIE, format!("demo_session={session_id}"))
            .body(())
            .expect("request should be valid");
        let (mut parts, _) = request.into_parts();

        let OptionalCurrentUser(user) = OptionalCurrentUser::from_request_parts(&mut parts, &state)
            .await
            .expect("authentication should succeed");

        assert_eq!(user, Some(CurrentUser::from(auth_user)));
    }

    #[tokio::test]
    async fn optional_user_uses_forwarded_header_in_neoshowcase_mode() {
        let auth_user = AuthUserRecord {
            id: Uuid::new_v4(),
            display_name: "alice".to_owned(),
        };
        let state = AppState::new(
            AuthMode::NeoShowcase,
            Arc::new(StubAuthRepository {
                expected: ExpectedAuth::NeoShowcase {
                    provider_subject: "alice".to_owned(),
                    user: auth_user.clone(),
                },
            }),
        );

        let request = Request::builder()
            .header("x-forwarded-user", "alice")
            .body(())
            .expect("request should be valid");
        let (mut parts, _) = request.into_parts();

        let OptionalCurrentUser(user) = OptionalCurrentUser::from_request_parts(&mut parts, &state)
            .await
            .expect("authentication should succeed");

        assert_eq!(user, Some(CurrentUser::from(auth_user)));
    }

    #[tokio::test]
    async fn required_user_rejects_unauthenticated_request() {
        let state = AppState::new(
            AuthMode::Demo,
            Arc::new(StubAuthRepository {
                expected: ExpectedAuth::NoRepositoryCall,
            }),
        );

        let request = Request::new(());
        let (mut parts, _) = request.into_parts();

        let error = CurrentUser::from_request_parts(&mut parts, &state)
            .await
            .expect_err("authentication should be required");

        assert!(matches!(error, AppError::Unauthorized));
    }
}
