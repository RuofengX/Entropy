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

pub mod error {
    use super::*;

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
}

type Result<T> = std::result::Result<T, error::ApiError>;

async fn verify_soul(world: &World, uid: &String, pw_hash: Option<String>) -> Result<()> {
    // Verify soul authority
    let pw_hash = pw_hash.ok_or(error::ApiError::EmptyPassword)?;
    if !world.verify_soul(uid, &pw_hash).await? {
        Err(error::ApiError::AuthError(uid.clone()).into())
    } else {
        Ok(())
    }
}

pub mod soul {
    use super::*;
    #[derive(Debug, Deserialize)]
    pub struct RegisterInfo {
        name: String,
        pw_hash: String,
    }

    #[debug_handler]
    pub(crate) async fn register(
        State(world): State<Arc<World>>,
        Json(RegisterInfo { name, pw_hash }): Json<RegisterInfo>,
    ) -> Result<Json<Soul>> {
        Ok(Json(world.register_soul(name, pw_hash).await?))
    }

    #[debug_handler]
    pub(crate) async fn get(
        AuthBasic((uid, pw_hash)): AuthBasic,
        State(world): State<Arc<World>>,
    ) -> Result<Json<Soul>> {
        verify_soul(&world, &uid, pw_hash).await?;
        Ok(Json(world.get_soul(&uid).await?.unwrap()))
    }
}

pub mod guest {
    use super::*;

    #[derive(Debug, Deserialize)]
    pub struct IndexQuery {
        id: GID,
    }

    #[debug_handler]
    pub(crate) async fn contain(
        AuthBasic((uid, pw_hash)): AuthBasic,
        Query(IndexQuery { id }): Query<IndexQuery>,
        State(world): State<Arc<World>>,
    ) -> Result<Json<Option<bool>>> {
        // Verify soul authority
        verify_soul(&world, &uid, pw_hash).await?;

        let wondering_soul = world.get_wondering_soul(&uid).await?.unwrap(); // Soul 注册后不会删除，所以这里不会出现 None
        Ok(Json(Some(wondering_soul.contain_guest(id))))
    }

    #[debug_handler]
    pub(crate) async fn get(
        AuthBasic((uid, pw_hash)): AuthBasic,
        Query(IndexQuery { id }): Query<IndexQuery>,
        State(world): State<Arc<World>>,
    ) -> Result<Json<Option<Guest>>> {
        verify_soul(&world, &uid, pw_hash).await?;

        let wondering_soul = world.get_wondering_soul(&uid).await?.unwrap(); // Soul 注册后不会删除，所以这里不会出现 None
        Ok(Json(wondering_soul.get_guest(id).await?))
    }

    #[derive(Debug, Deserialize)]
    pub struct WalkBody {
        id: GID,
        to: direction::Direction,
    }

    #[debug_handler]
    pub(crate) async fn walk(
        AuthBasic((uid, pw_hash)): AuthBasic,
        State(world): State<Arc<World>>,
        Json(WalkBody { id, to }): Json<WalkBody>,
    ) -> Result<Json<Option<Guest>>> {
        verify_soul(&world, &uid, pw_hash).await?;

        let wondering_soul = world.get_wondering_soul(&uid).await?.unwrap();
        Ok(Json(wondering_soul.walk(id, to).await?))
    }

    #[derive(Debug, Deserialize)]
    pub struct HarvestBody {
        id: GID,
        at: usize,
    }

    #[debug_handler]
    pub(crate) async fn harvest(
        AuthBasic((uid, pw_hash)): AuthBasic,
        State(world): State<Arc<World>>,
        Json(HarvestBody { id, at }): Json<HarvestBody>,
    ) -> Result<Json<Option<Guest>>> {
        verify_soul(&world, &uid, pw_hash).await?;

        let wondering_soul = world.get_wondering_soul(&uid).await?.unwrap();
        Ok(Json(wondering_soul.harvest(id, at).await?))
    }

    #[derive(Debug, Deserialize)]
    pub struct HeatBody {
        id: GID,
        at: usize,
        energy: u8,
    }
    #[debug_handler]
    pub(crate) async fn heat(
        AuthBasic((uid, pw_hash)): AuthBasic,
        State(world): State<Arc<World>>,
        Json(HeatBody { id, at, energy }): Json<HeatBody>,
    ) -> Result<Json<Option<Guest>>> {
        verify_soul(&world, &uid, pw_hash).await?;

        let wondering_soul = world.get_wondering_soul(&uid).await?.unwrap();
        Ok(Json(wondering_soul.heat(id, at, energy).await?))
    }
}

// Node
pub mod node {
    use super::*;
    #[debug_handler]
    pub(crate) async fn get_json(
        State(world): State<Arc<World>>,
        Path((x, y)): Path<(i16, i16)>,
    ) -> Result<Json<NodeData>> {
        Ok(Json(world.detect_node(NodeID(x, y)).await?))
    }

    #[debug_handler]
    pub(crate) async fn get_bytes(
        State(world): State<Arc<World>>,
        Path((x, y)): Path<(i16, i16)>,
    ) -> Result<Bytes> {
        Ok(Bytes::from(
            world.detect_node(NodeID(x, y)).await?.0.to_vec(),
        ))
    }
}
