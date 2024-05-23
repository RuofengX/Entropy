use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    Json,
};
use axum_auth::AuthBasic;

use super::{payload, verify::*, Result};
use crate::{
    guest::{Guest, GID},
    node::{NodeData, NodeID},
    soul::Soul,
    world::World,
};

pub mod soul {
    use super::*;

    pub(crate) async fn register(
        State(world): State<Arc<World>>,
        Json(payload::RegisterInfo { name, pw_hash }): Json<payload::RegisterInfo>,
    ) -> Result<Json<Soul>> {
        Ok(Json(world.register_soul(name, pw_hash).await?))
    }

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

    pub(crate) async fn contain(
        AuthBasic((uid, pw_hash)): AuthBasic,
        Query(id): Query<GID>,
        State(world): State<Arc<World>>,
    ) -> Result<Json<bool>> {
        // Verify soul authority
        verify_soul(&world, &uid, pw_hash).await?;

        let wondering_soul = world.get_wondering_soul(&uid).await?.unwrap(); // Soul 注册后不会删除，所以这里不会出现 None
        Ok(Json(wondering_soul.contain_guest(id)))
    }

    pub(crate) async fn get(
        AuthBasic((uid, pw_hash)): AuthBasic,
        Query(id): Query<GID>,
        State(world): State<Arc<World>>,
    ) -> Result<Json<Option<Guest>>> {
        verify_soul(&world, &uid, pw_hash).await?;

        let wondering_soul = world.get_wondering_soul(&uid).await?.unwrap(); // Soul 注册后不会删除，所以这里不会出现 None
        Ok(Json(wondering_soul.get_guest(id).await?))
    }

    pub(crate) async fn walk(
        AuthBasic((uid, pw_hash)): AuthBasic,
        State(world): State<Arc<World>>,
        Json(payload::WalkCommand { id, x, y }): Json<payload::WalkCommand>,
    ) -> Result<Json<Option<Guest>>> {
        verify_soul(&world, &uid, pw_hash).await?;

        let wondering_soul = world.get_wondering_soul(&uid).await?.unwrap();
        Ok(Json(wondering_soul.walk(id, (x as i16, y as i16)).await?))
    }

    pub(crate) async fn harvest(
        AuthBasic((uid, pw_hash)): AuthBasic,
        State(world): State<Arc<World>>,
        Json(payload::HarvestCommand { id, at }): Json<payload::HarvestCommand>,
    ) -> Result<Json<Option<Guest>>> {
        verify_soul(&world, &uid, pw_hash).await?;

        let wondering_soul = world.get_wondering_soul(&uid).await?.unwrap();
        Ok(Json(wondering_soul.harvest(id, at).await?))
    }

    pub(crate) async fn heat(
        AuthBasic((uid, pw_hash)): AuthBasic,
        State(world): State<Arc<World>>,
        Json(payload::HeatCommand { id, at, energy }): Json<payload::HeatCommand>,
    ) -> Result<Json<Option<Guest>>> {
        verify_soul(&world, &uid, pw_hash).await?;

        let wondering_soul = world.get_wondering_soul(&uid).await?.unwrap();
        Ok(Json(wondering_soul.heat(id, at, energy).await?))
    }

    pub(crate) async fn spawn(
        AuthBasic((uid, pw_hash)): AuthBasic,
        State(world): State<Arc<World>>,
        Json(payload::SpawnCommand { id }): Json<payload::SpawnCommand>,
    ) -> Result<Json<Option<Guest>>> {
        let w_soul = get_verified_soul(&world, &uid, pw_hash).await?;
        Ok(Json(w_soul.spawn(id).await?))
    }
}

// Node
pub mod node {
    use super::*;

    pub(crate) async fn get_json(
        State(world): State<Arc<World>>,
        Path((x, y)): Path<(i16, i16)>,
    ) -> Result<Json<NodeData>> {
        Ok(Json(world.detect_node(NodeID(x, y)).await?))
    }

    pub(crate) async fn get_bytes(
        State(world): State<Arc<World>>,
        Path((x, y)): Path<(i16, i16)>,
    ) -> Result<Bytes> {
        Ok(Bytes::from(
            world.detect_node(NodeID(x, y)).await?.0.to_vec(),
        ))
    }
}
