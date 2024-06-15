use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::Json;
use axum_auth::AuthBasic;
use sea_orm::{ActiveModelTrait, TransactionTrait};
use serde::Deserialize;

use crate::err::{ApiError, ModelError, OperationError};
use crate::grid::{navi, FlatID, Node, NodeID, INDEXED_NAVI};
use crate::{entity, grid};

use super::AppState;
use crate::entity::guest::Model as Guest;
use crate::entity::player::Model as Player;

#[derive(Debug, Deserialize)]
pub struct PlayerAuth {
    id: i32,
    password: String,
}

fn verify_header(auth: (String, Option<String>)) -> Result<PlayerAuth, ApiError> {
    let (id, password) = auth;
    let password = password.ok_or(ApiError::AuthHeader)?;
    if let Ok(id) = id.parse::<i32>() {
        Ok(PlayerAuth { id, password })
    } else {
        Err(ApiError::AuthHeader)
    }
}

#[derive(Debug, Deserialize)]
pub struct PlayerRegister {
    name: String,
    password: String,
}

pub async fn register(
    State(state): State<AppState>,
    Json(PlayerRegister { name, password }): Json<PlayerRegister>,
) -> Result<Json<Player>, ApiError> {
    Ok(Json(
        entity::register_player(&state.conn, name, password).await?,
    ))
}

pub async fn get_player(
    State(state): State<AppState>,
    AuthBasic(auth): AuthBasic,
) -> Result<Json<Player>, ApiError> {
    let PlayerAuth { id, password } = verify_header(auth)?;

    Ok(Json(
        entity::get_player(&state.conn, id, password)
            .await?
            .ok_or(ApiError::AuthError(id))?,
    ))
}

pub async fn list_guest(
    State(state): State<AppState>,
    AuthBasic(auth): AuthBasic,
) -> Result<Json<Vec<Guest>>, ApiError> {
    let PlayerAuth { id, password } = verify_header(auth)?;

    Ok(Json(
        entity::get_player(&state.conn, id, password)
            .await?
            .ok_or(ApiError::AuthError(id))?
            .list_guest(&state.conn)
            .await?,
    ))
}

pub async fn spawn_guest(
    State(state): State<AppState>,
    AuthBasic(auth): AuthBasic,
) -> Result<Json<Guest>, ApiError> {
    let PlayerAuth { id, password } = verify_header(auth)?;

    let txn = state.conn.begin().await?;
    let p = entity::get_exact_player(&txn, id, password).await?;
    let rtn = p.spawn_guest(&txn).await?;
    txn.commit().await?;
    Ok(Json(rtn))
}

pub async fn get_node(
    State(state): State<AppState>,
    Path((x, y)): Path<(i16, i16)>,
) -> Result<Json<grid::Node>, ApiError> {
    let txn = state.conn.begin().await?;
    let n = entity::get_node(&txn, NodeID::from_xy(x, y)).await?;
    txn.commit().await?;
    Ok(Json(grid::Node::from(n)))
}

pub async fn get_node_bytes(
    State(state): State<AppState>,
    Path((x, y)): Path<(i16, i16)>,
) -> Result<Bytes, ApiError> {
    let txn = state.conn.begin().await?;
    let rtn = entity::get_node(&txn, NodeID::from_xy(x, y)).await?;
    txn.commit().await?;
    Ok(Bytes::from(rtn.data))
}

pub async fn get_guest(
    State(state): State<AppState>,
    Path(gid): Path<i32>,
    AuthBasic(auth): AuthBasic,
) -> Result<Json<Guest>, ApiError> {
    let PlayerAuth { id, password } = verify_header(auth)?;

    let txn = state.conn.begin().await?;
    let p = entity::get_exact_player(&txn, id, password).await?;
    let g = p.get_guest(&txn, gid).await?;
    txn.commit().await?;
    Ok(Json(g))
}

#[derive(Debug, Deserialize)]
pub struct WalkCommand {
    to: navi::Direction,
}
impl WalkCommand {
    pub fn verify(&self) -> Result<(), OperationError> {
        if INDEXED_NAVI.contains(&self.to) {
            Ok(())
        } else {
            Err(OperationError::DirectionNotAllowed(self.to))
        }
    }
}

pub async fn walk(
    State(state): State<AppState>,
    AuthBasic(auth): AuthBasic,
    Path(gid): Path<i32>,
    Json(cmd): Json<WalkCommand>,
) -> Result<Json<Guest>, ApiError> {
    // verify
    cmd.verify()?;
    let PlayerAuth { id, password } = verify_header(auth)?;

    // transaction
    let txn = state.conn.begin().await?;
    let p = entity::get_exact_player(&txn, id, password).await?;
    let g = p.get_guest(&txn, gid).await?;
    let g = g.walk(&txn, cmd.to).await?;
    txn.commit().await?;

    //return
    Ok(Json(g))
}

#[derive(Debug, Deserialize)]
pub struct HarvestCommand {
    pub at: usize,
}

pub async fn harvest(
    State(state): State<AppState>,
    AuthBasic(auth): AuthBasic,
    Path(gid): Path<i32>,
    Json(cmd): Json<HarvestCommand>,
) -> Result<Json<Guest>, ApiError> {
    // verify
    let PlayerAuth { id, password } = verify_header(auth)?;

    // transaction
    let txn = state.conn.begin().await?;
    let p = entity::get_exact_player(&txn, id, password).await?;
    let g = p.get_guest(&txn, gid).await?;
    let pos = FlatID::from(g.pos);
    let n = entity::get_node(&txn, pos.into()).await?;

    let node: Node = Node::from_model(n);
    let (g, n) = g
        .generate(node, cmd.at)
        .map_err(|e| ApiError::Operation(OperationError::Model(e)))?;
    let g = g.update(&txn).await?;
    n.update(&txn).await?;

    txn.commit().await?;

    Ok(Json(g))
}

#[derive(Debug, Deserialize)]
pub struct ArrangeCommand {
    pub transfer_energy: i64,
}

pub async fn arrange(
    State(state): State<AppState>,
    AuthBasic(auth): AuthBasic,
    Path(gid): Path<i32>,
    Json(cmd): Json<ArrangeCommand>,
) -> Result<Json<Guest>, ApiError> {
    // verify
    let PlayerAuth { id, password } = verify_header(auth)?;

    // transaction
    let txn = state.conn.begin().await?;
    let p = entity::get_exact_player(&txn, id, password).await?;
    let g = p.get_guest(&txn, gid).await?;

    // consume energy
    let g_count = p.count_guest(&txn).await?;
    if g_count >= u32::MAX as u64 {
        return Err(OperationError::Model(ModelError::OutOfLimit {
            desc: format!("owned guest number"),
            limit_type: "u32",
        })
        .into());
    };
    let g_count = g_count.try_into().map_err(|_| {
        OperationError::Model(ModelError::OutOfLimit {
            desc: format!("owned guest number"),
            limit_type: "u32",
        })
    })?;
    let consume_energy = 2i64.pow(g_count);
    let g = g.consume_energy(&txn, consume_energy).await?;
    let new_g = g.arrange_free(&txn, cmd.transfer_energy).await?;

    txn.commit().await?;

    Ok(Json(new_g))
}
