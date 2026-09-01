use axum::http::HeaderMap;
use axum_extra::extract::cookie::CookieJar;
use uuid::Uuid;

use crate::repository::{AuthRepository, RepositoryError};

use super::current_user::CurrentUser;

pub(crate) const SESSION_COOKIE_NAME: &str = "demo_session";

pub(crate) async fn resolve_current_user(
    headers: &HeaderMap,
    repository: &dyn AuthRepository,
) -> Result<Option<CurrentUser>, RepositoryError> {
    let cookie_jar = CookieJar::from_headers(headers);

    let Some(session_cookie) = cookie_jar.get(SESSION_COOKIE_NAME) else {
        return Ok(None);
    };

    let Ok(session_id) = Uuid::parse_str(session_cookie.value()) else {
        return Ok(None);
    };

    let user = repository.find_user_by_demo_session(session_id).await?;

    Ok(user.map(CurrentUser::from))
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use axum::http::{HeaderMap, HeaderValue, header};
    use uuid::Uuid;

    use crate::repository::{AuthProvider, AuthRepository, AuthUserRecord, RepositoryError};

    use super::{SESSION_COOKIE_NAME, resolve_current_user};

    struct StubAuthRepository {
        expected_session_id: Option<Uuid>,
        user: Option<AuthUserRecord>,
    }

    #[async_trait]
    impl AuthRepository for StubAuthRepository {
        async fn find_user_by_demo_session(
            &self,
            session_id: Uuid,
        ) -> Result<Option<AuthUserRecord>, RepositoryError> {
            let expected_session_id = self
                .expected_session_id
                .expect("repository should not be called");

            assert_eq!(session_id, expected_session_id);

            Ok(self.user.clone())
        }

        async fn find_user_by_provider_subject(
            &self,
            _auth_provider: AuthProvider,
            _provider_subject: &str,
        ) -> Result<Option<AuthUserRecord>, RepositoryError> {
            panic!("demo resolver must not look up a provider subject");
        }

        async fn get_or_create_user(
            &self,
            _auth_provider: AuthProvider,
            _provider_subject: &str,
            _display_name: &str,
        ) -> Result<AuthUserRecord, RepositoryError> {
            panic!("demo resolver must not create a user");
        }
    }

    #[tokio::test]
    async fn missing_cookie_is_unauthenticated() {
        let repository = StubAuthRepository {
            expected_session_id: None,
            user: None,
        };
        let headers = HeaderMap::new();

        let user = resolve_current_user(&headers, &repository)
            .await
            .expect("repository should not fail");

        assert_eq!(user, None);
    }

    #[tokio::test]
    async fn invalid_session_cookie_is_unauthenticated() {
        let repository = StubAuthRepository {
            expected_session_id: None,
            user: None,
        };
        let headers = headers_with_session_cookie("not-a-uuid");

        let user = resolve_current_user(&headers, &repository)
            .await
            .expect("repository should not fail");

        assert_eq!(user, None);
    }

    #[tokio::test]
    async fn unknown_session_is_unauthenticated() {
        let session_id = Uuid::new_v4();
        let repository = StubAuthRepository {
            expected_session_id: Some(session_id),
            user: None,
        };
        let headers = headers_with_session_cookie(&session_id.to_string());

        let user = resolve_current_user(&headers, &repository)
            .await
            .expect("repository should not fail");

        assert_eq!(user, None);
    }

    #[tokio::test]
    async fn valid_session_resolves_current_user() {
        let session_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let repository = StubAuthRepository {
            expected_session_id: Some(session_id),
            user: Some(AuthUserRecord {
                user_id,
                display_name: "alice".to_owned(),
                auth_provider: AuthProvider::Demo,
            }),
        };
        let headers = headers_with_session_cookie(&session_id.to_string());

        let user = resolve_current_user(&headers, &repository)
            .await
            .expect("repository should not fail")
            .expect("user should be authenticated");

        assert_eq!(user.user_id, user_id);
        assert_eq!(user.display_name, "alice");
    }

    fn headers_with_session_cookie(session_id: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        let cookie = format!("{SESSION_COOKIE_NAME}={session_id}");

        headers.insert(
            header::COOKIE,
            HeaderValue::from_str(&cookie).expect("cookie should be a valid header value"),
        );

        headers
    }
}
