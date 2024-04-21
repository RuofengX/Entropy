use std::sync::Arc;

use axum::{
    debug_handler,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use axum_auth::AuthBasic;
use serde::Deserialize;
use thiserror::Error;

use crate::{guest::GID, soul::Soul, world::World};

type Result<T> = std::result::Result<T, ApiError>;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("auth error, wrong uid::{0} or password")]
    AuthError(String),

    #[error("no password found in request header")]
    EmptyPassword,

    #[error("error when serialize/deserialize json data")]
    JsonParseError(#[from] serde_json::Error),

    /// all the other error is from backend
    #[error(transparent)]
    BackendError(#[from] anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("{self}")).into_response()
    }
}

#[derive(Debug, Deserialize)]
pub struct RegisterInfo {
    name: String,
    pw_hash: String,
}

#[derive(Debug, Deserialize)]
pub struct GuestIndex {
    gid: GID,
}

#[debug_handler]
pub(crate) async fn register_soul(
    Query(RegisterInfo { name, pw_hash }): Query<RegisterInfo>,
    State(world): State<Arc<World>>,
) -> Result<Json<Soul>> {
    Ok(Json(world.register_soul(name, pw_hash).await?))
}

//FIXME: SAVE OR GET SOUL NOT WORKING
#[debug_handler]
pub(crate) async fn contain_guest(
    AuthBasic((uid, pw_hash)): AuthBasic,
    Query(GuestIndex { gid }): Query<GuestIndex>,
    State(world): State<Arc<World>>,
) -> Result<Json<Option<bool>>> {
    // Verify soul authority
    let pw_hash = pw_hash.ok_or(ApiError::EmptyPassword)?;
    if !world.verify_soul(&uid, &pw_hash).await? {
        return Err(ApiError::AuthError(uid));
    }

    Ok(
        if let Some(wondering_soul) = world.get_wondering_soul(&uid).await? {
            Json(Some(wondering_soul.contains_guest(gid)))
        } else {
            Json(None)
        },
    )
}
