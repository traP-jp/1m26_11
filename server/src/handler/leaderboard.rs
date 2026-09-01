use axum::{
    Json,
    extract::{Path, State},
};
use openapi_generated::{
    models::{LeaderboardEntry, LeaderboardMe, LeaderboardResponse, User},
    types::Nullable,
};
use uuid::Uuid;

use crate::{AppState, auth::current_user::OptionalCurrentUser, error::AppError};

pub(crate) async fn get_room_leaderboard(
    State(state): State<AppState>,
    OptionalCurrentUser(current_user): OptionalCurrentUser,
    Path(room_id): Path<String>,
) -> Result<Json<LeaderboardResponse>, AppError> {
    let room_id =
        Uuid::parse_str(&room_id).map_err(|_| AppError::bad_request("invalid room_id"))?;

    let Some(room) = state.auth_repository.find_room_by_id(room_id).await? else {
        return Err(AppError::not_found("room not found"));
    };

    if !room.is_published {
        return Err(AppError::not_found("room not found"));
    }

    let records = state
        .auth_repository
        .find_leaderboard_by_room_id(room_id)
        .await?;

    let me = current_user
        .as_ref()
        .and_then(|user| records.iter().find(|record| record.user_id == user.user_id))
        .map_or(Nullable::Null, |record| {
            Nullable::Present(LeaderboardMe::new(
                record.rank,
                record.elapsed_ms,
                record.query_count,
            ))
        });

    let entries = records
        .into_iter()
        .map(|record| {
            LeaderboardEntry::new(
                record.rank,
                User::new(record.user_id, record.display_name),
                record.elapsed_ms,
                record.query_count,
                record.cleared_at,
            )
        })
        .collect();

    Ok(Json(LeaderboardResponse::new(room_id, entries, me)))
}
