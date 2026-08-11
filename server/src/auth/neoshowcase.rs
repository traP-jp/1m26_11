use axum::http::HeaderMap;

use crate::repository::{AuthRepository, RepositoryError};

use super::current_user::CurrentUser;

const X_FORWARDED_USER_HEADER: &str = "x-forwarded-user";
const AUTH_PROVIDER: &str = "neoshowcase";

pub(crate) async fn resolve_current_user(
    headers: &HeaderMap,
    repository: &dyn AuthRepository,
) -> Result<Option<CurrentUser>, RepositoryError> {
    let Some(header_value) = headers.get(X_FORWARDED_USER_HEADER) else {
        return Ok(None);
    };

    let Ok(provider_subject) = header_value.to_str() else {
        return Ok(None);
    };

    if provider_subject.is_empty() {
        return Ok(None);
    }

    let user = repository
        .get_or_create_user(AUTH_PROVIDER, provider_subject, provider_subject)
        .await?;

    Ok(Some(user.into()))
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use axum::http::{HeaderMap, HeaderValue};
    use uuid::Uuid;

    use crate::repository::{AuthRepository, AuthUserRecord, RepositoryError};

    use super::{AUTH_PROVIDER, resolve_current_user};

    struct StubAuthRepository {
        user_id: Uuid,
    }

    #[async_trait]
    impl AuthRepository for StubAuthRepository {
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
            auth_provider: &str,
            provider_subject: &str,
            display_name: &str,
        ) -> Result<AuthUserRecord, RepositoryError> {
            assert_eq!(auth_provider, AUTH_PROVIDER);
            assert_eq!(provider_subject, "alice");
            assert_eq!(display_name, "alice");

            Ok(AuthUserRecord {
                id: self.user_id,
                display_name: display_name.to_owned(),
            })
        }
    }

    #[tokio::test]
    async fn missing_header_is_unauthenticated() {
        let repository = StubAuthRepository {
            user_id: Uuid::new_v4(),
        };
        let headers = HeaderMap::new();

        let user = resolve_current_user(&headers, &repository)
            .await
            .expect("repository should not fail");

        assert_eq!(user, None);
    }

    #[tokio::test]
    async fn forwarded_user_resolves_current_user() {
        let user_id = Uuid::new_v4();
        let repository = StubAuthRepository { user_id };
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-user", HeaderValue::from_static("alice"));

        let user = resolve_current_user(&headers, &repository)
            .await
            .expect("repository should not fail")
            .expect("user should be authenticated");

        assert_eq!(user.id, user_id);
        assert_eq!(user.display_name, "alice");
    }
}
