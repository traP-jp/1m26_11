use axum::{Json, extract::State};
use openapi_generated::{
    NullValue,
    models::{
        MeDemoAuthenticated, MeDemoUnauthenticated, MeNeoshowcaseAuthenticated,
        MeNeoshowcaseUnauthenticated, MeResponse, User,
    },
};

use crate::{
    AppState,
    auth::current_user::{CurrentUser, OptionalCurrentUser},
    config::AuthMode,
};

const LOGIN_URL: &str = "/_oauth/login?redirect=/";
const LOGOUT_URL: &str = "/_oauth/logout?redirect=/";

pub(crate) async fn get_me(
    State(state): State<AppState>,
    OptionalCurrentUser(user): OptionalCurrentUser,
) -> Json<MeResponse> {
    Json(build_me_response(state.auth_mode, user))
}

fn build_me_response(auth_mode: AuthMode, user: Option<CurrentUser>) -> MeResponse {
    match (auth_mode, user) {
        (AuthMode::Demo, Some(user)) => MeDemoAuthenticated::new(
            true,
            "demo".to_owned(),
            User::new(user.user_id, user.display_name),
            NullValue,
            NullValue,
        )
        .into(),

        (AuthMode::Demo, None) => {
            MeDemoUnauthenticated::new(false, "demo".to_owned(), NullValue, NullValue, NullValue)
                .into()
        }

        (AuthMode::NeoShowcase, Some(user)) => MeNeoshowcaseAuthenticated::new(
            true,
            "neoshowcase".to_owned(),
            User::new(user.user_id, user.display_name),
            NullValue,
            LOGOUT_URL.to_owned(),
        )
        .into(),

        (AuthMode::NeoShowcase, None) => MeNeoshowcaseUnauthenticated::new(
            false,
            "neoshowcase".to_owned(),
            NullValue,
            LOGIN_URL.to_owned(),
            NullValue,
        )
        .into(),
    }
}
