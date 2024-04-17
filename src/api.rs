use std::sync::Arc;

use axum::{
    debug_handler,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use serde_json::Value;

use crate::{db::SledStorage, guest::GID, world::World};

pub(crate) struct ApiError(anyhow::Error);
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", self.0)).into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for ApiError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

type Result<T> = std::result::Result<T, ApiError>;

#[derive(Debug, Deserialize)]
pub(crate) struct SoulUser {
    username: String,
    pw_hash: Vec<u8>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SoulGuestIndex {
    uid: String,
    gid: GID,
}

#[debug_handler]
pub(crate) async fn register_soul(
    Query(soul): Query<SoulUser>,
    State(world): State<Arc<World<SledStorage>>>,
) -> Result<Json<Value>> {
    Ok(Json(serde_json::to_value(
        world.register_soul(soul.username, soul.pw_hash).await?,
    )?))
}

#[debug_handler]
pub(crate) async fn contains_guest(
    Query(guest): Query<SoulGuestIndex>,
    State(world): State<Arc<World<SledStorage>>>,
) -> Result<Json<Value>> {
    let s = world.get_soul(guest.uid).await?;
    Ok(Json(serde_json::to_value(s.contains_guest(guest.gid))?))
}
