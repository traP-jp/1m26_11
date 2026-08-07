use email_address::EmailAddress;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const MAX_USER_FIELD_LENGTH: usize = 255;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
}

impl CreateUserRequest {
    pub(crate) fn validate(&self) -> Result<(), &'static str> {
        if self.name.trim().is_empty() {
            return Err("name must not be blank");
        }
        if self.name.chars().count() > MAX_USER_FIELD_LENGTH {
            return Err("name must be at most 255 characters");
        }
        if self.email.chars().count() > MAX_USER_FIELD_LENGTH {
            return Err("email must be at most 255 characters");
        }
        if !EmailAddress::is_valid(&self.email) {
            return Err("email must be a valid email address");
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateUserResponse {
    pub id: Uuid,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub icon_url: String,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ErrorResponse {
    pub message: String,
}
