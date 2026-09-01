use axum::{Json, extract::State};
use openapi_generated::models::{GenreProgress, MeProgressResponse};

use crate::{AppState, auth::current_user::CurrentUser, error::AppError};

pub(crate) async fn get_me_progress(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> Result<Json<MeProgressResponse>, AppError> {
    let progress = state
        .auth_repository
        .find_user_progress(current_user.user_id)
        .await?;

    let by_genre = progress
        .by_genre
        .into_iter()
        .map(|progress| {
            GenreProgress::new(
                progress.genre,
                progress.cleared_room_count,
                progress.total_room_count,
            )
        })
        .collect();

    Ok(Json(MeProgressResponse::new(
        progress.cleared_room_count,
        progress.total_room_count,
        by_genre,
    )))
}
