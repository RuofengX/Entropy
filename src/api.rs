use std::sync::Arc;

use axum::{
    body::Bytes,
    debug_handler,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use axum_auth::AuthBasic;
use serde::Deserialize;
use thiserror::Error;

use crate::{
    guest::{Guest, GID},
    node::{direction, NodeData, NodeID},
    soul::Soul,
    world::World,
};

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
pub struct GuestIndexQuery {
    id: GID,
}

#[debug_handler]
pub(crate) async fn register_soul(
    State(world): State<Arc<World>>,
    Json(RegisterInfo { name, pw_hash }): Json<RegisterInfo>,
) -> Result<Json<Soul>> {
    Ok(Json(world.register_soul(name, pw_hash).await?))
}

async fn verify_soul(world: &World, uid: &String, pw_hash: Option<String>) -> Result<()> {
    // Verify soul authority
    let pw_hash = pw_hash.ok_or(ApiError::EmptyPassword)?;
    if !world.verify_soul(uid, &pw_hash).await? {
        Err(ApiError::AuthError(uid.clone()).into())
    } else {
        Ok(())
    }
}

#[debug_handler]
pub(crate) async fn contain_guest(
    AuthBasic((uid, pw_hash)): AuthBasic,
    Query(GuestIndexQuery { id }): Query<GuestIndexQuery>,
    State(world): State<Arc<World>>,
) -> Result<Json<Option<bool>>> {
    // Verify soul authority
    verify_soul(&world, &uid, pw_hash).await?;

    let wondering_soul = world.get_wondering_soul(&uid).await?.unwrap(); // Soul 注册后不会删除，所以这里不会出现 None
    Ok(Json(Some(wondering_soul.contains_guest(id))))
}

#[debug_handler]
pub(crate) async fn get_guest(
    AuthBasic((uid, pw_hash)): AuthBasic,
    Query(GuestIndexQuery { id }): Query<GuestIndexQuery>,
    State(world): State<Arc<World>>,
) -> Result<Json<Option<Guest>>> {
    verify_soul(&world, &uid, pw_hash).await?;

    let wondering_soul = world.get_wondering_soul(&uid).await?.unwrap(); // Soul 注册后不会删除，所以这里不会出现 None
    Ok(Json(wondering_soul.get_guest(id).await?))
}

#[derive(Debug, Deserialize)]
pub struct GuestWalkBody {
    id: GID,
    to: direction::Direction,
}

#[debug_handler]
pub(crate) async fn guest_walk(
    AuthBasic((uid, pw_hash)): AuthBasic,
    State(world): State<Arc<World>>,
    Json(GuestWalkBody { id, to }): Json<GuestWalkBody>,
) -> Result<Json<Option<Guest>>> {
    verify_soul(&world, &uid, pw_hash).await?;

    let wondering_soul = world.get_wondering_soul(&uid).await?.unwrap();
    Ok(Json(wondering_soul.walk(id, to).await?))
}

#[derive(Debug, Deserialize)]
pub struct GuestHarvestBody {
    id: GID,
    at: usize,
}

#[debug_handler]
pub(crate) async fn guest_harvest(
    AuthBasic((uid, pw_hash)): AuthBasic,
    State(world): State<Arc<World>>,
    Json(GuestHarvestBody { id, at }): Json<GuestHarvestBody>,
) -> Result<Json<Option<Guest>>> {
    verify_soul(&world, &uid, pw_hash).await?;

    let wondering_soul = world.get_wondering_soul(&uid).await?.unwrap();
    Ok(Json(wondering_soul.harvest(id, at).await?))
}

#[debug_handler]
pub(crate) async fn get_node_json(
    State(world): State<Arc<World>>,
    Path((x, y)): Path<(i16, i16)>,
) -> Result<Json<NodeData>> {
    Ok(Json(world.detect_node(NodeID(x, y)).await?))
}

#[debug_handler]
pub(crate) async fn get_node_bytes(
    State(world): State<Arc<World>>,
    Path((x, y)): Path<(i16, i16)>,
) -> Result<Bytes> {
    Ok(Bytes::from(
        world.detect_node(NodeID(x, y)).await?.0.to_vec(),
    ))
}
