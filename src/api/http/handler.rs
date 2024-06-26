use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::Json;
use axum_auth::AuthBasic;
use sea_orm::{
    AccessMode, DatabaseConnection, DatabaseTransaction, DbErr, IsolationLevel, TransactionTrait,
};
use serde::Deserialize;
use tracing::{instrument, Level};

use crate::api::{Attachment, MsgPak};
use crate::entity::variant::{DetectedGuest, PublicPlayer};
use crate::err::{ApiError, OperationError};
use crate::grid::{navi, NodeID, ALLOWED_NAVI};
use crate::{entity, grid};

use super::AppState;
use crate::entity::guest::Model as Guest;
use crate::entity::player::Model as Player;

#[derive(Debug, Deserialize)]
pub struct PlayerAuth {
    id: i32,
    password: String,
}

#[derive(Debug, Deserialize)]
pub struct PlayerRegister {
    name: String,
    password: String,
}

#[derive(Debug, Deserialize)]
pub struct WalkCommand {
    to: navi::Direction,
}
impl WalkCommand {
    pub fn verify(&self) -> Result<(), OperationError> {
        if ALLOWED_NAVI.contains(&self.to) {
            Ok(())
        } else {
            Err(OperationError::DirectionNotAllowed(self.to))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct HeatCommand {
    pub at: usize,
    pub energy: i64,
}

#[derive(Debug, Deserialize)]
pub struct HarvestCommand {
    pub at: usize,
}

#[derive(Debug, Deserialize)]
pub struct ArrangeCommand {
    pub transfer_energy: i64,
}

#[instrument(skip(state, ), err(level = Level::INFO))]
pub async fn ping(State(state): State<AppState>) -> Result<&'static str, ApiError> {
    state.conn.ping().await?;
    Ok("pong")
}

#[instrument(skip(state), ret(level = Level::DEBUG), err(level = Level::INFO))]
pub async fn get_player_public(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<PublicPlayer>, ApiError> {
    Ok(Json(
        entity::get_exact_player_public(&state.conn, id).await?,
    ))
}

#[instrument(skip(state), ret(level = Level::DEBUG), err(level = Level::INFO))]
pub async fn register(
    State(state): State<AppState>,
    Json(PlayerRegister { name, password }): Json<PlayerRegister>,
) -> Result<Json<Player>, ApiError> {
    Ok(Json(
        entity::register_player(&state.conn, name, password).await?,
    ))
}

#[instrument(skip(state, auth), ret(level = Level::DEBUG), err(level = Level::INFO))]
pub async fn verify_player(
    State(state): State<AppState>,
    AuthBasic(auth): AuthBasic,
) -> Result<Json<Player>, ApiError> {
    let PlayerAuth { id, password } = verify_header(auth)?;

    Ok(Json(
        entity::get_exact_player(&state.conn, id, password).await?,
    ))
}

#[instrument(skip(state, auth), ret(level = Level::DEBUG), err(level = Level::INFO))]
pub async fn list_guest(
    State(state): State<AppState>,
    AuthBasic(auth): AuthBasic,
) -> Result<Json<Vec<Guest>>, ApiError> {
    let PlayerAuth { id, password } = verify_header(auth)?;
    let txn = begin_txn(&state.conn).await?;
    let gs = entity::list_guest(&txn, id, password).await?;
    txn.commit().await?;
    Ok(Json(gs))
}

#[instrument(skip(state, auth), ret(level = Level::DEBUG), err(level = Level::INFO))]
pub async fn spawn_guest(
    State(state): State<AppState>,
    AuthBasic(auth): AuthBasic,
) -> Result<Json<Guest>, ApiError> {
    let PlayerAuth { id, password } = verify_header(auth)?;
    let txn = begin_txn(&state.conn).await?;
    let g = entity::spawn_guest(&txn, id, password).await?;
    txn.commit().await?;
    Ok(Json(g))
}

#[instrument(skip(state), err(level = Level::INFO))]
pub async fn get_node(
    State(state): State<AppState>,
    Path((x, y)): Path<(i16, i16)>,
) -> Result<Json<grid::Node>, ApiError> {
    let txn = begin_txn(&state.conn).await?;
    let n = entity::get_node(&txn, NodeID::from_xy(x, y)).await?;
    txn.commit().await?;
    Ok(Json(grid::Node::from(n)))
}

#[instrument(skip(state), err(level = Level::INFO))]
pub async fn get_node_bytes(
    State(state): State<AppState>,
    Path((x, y)): Path<(i16, i16)>,
) -> Result<Attachment<Vec<u8>>, ApiError> {
    let txn = begin_txn(&state.conn).await?;
    let n = entity::get_node(&txn, NodeID::from_xy(x, y)).await?;
    txn.commit().await?;
    Ok(Attachment {
        raw: n.data,
        file_name: format!("{x}-{y}.bin"),
    })
}

#[instrument(skip(state), err(level = Level::INFO))]
pub async fn get_node_msgpak(
    State(state): State<AppState>,
    Path((x, y)): Path<(i16, i16)>,
) -> Result<impl IntoResponse, ApiError> {
    let txn = begin_txn(&state.conn).await?;
    let n = entity::get_node(&txn, NodeID::from_xy(x, y)).await?;
    txn.commit().await?;
    Ok(Attachment {
        raw: MsgPak(n.data),
        file_name: format!("{x}-{y}.msgpak"),
    })
}

#[instrument(skip(state, auth), ret(level = Level::DEBUG), err(level = Level::INFO))]
pub async fn get_guest(
    State(state): State<AppState>,
    Path(gid): Path<i32>,
    AuthBasic(auth): AuthBasic,
) -> Result<Json<Guest>, ApiError> {
    let PlayerAuth { id, password } = verify_header(auth)?;
    let txn = begin_txn(&state.conn).await?;
    let g = entity::get_guest(&txn, id, password, gid).await?;
    txn.commit().await?;
    Ok(Json(g))
}

#[instrument(skip(state, auth), ret(level = Level::DEBUG), err(level = Level::INFO))]
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
    let txn = begin_txn(&state.conn).await?;
    let g = entity::walk(&txn, id, password, gid, cmd.to).await?;
    txn.commit().await?;

    //return
    Ok(Json(g))
}

#[instrument(skip(state, auth), ret(level = Level::DEBUG), err(level = Level::INFO))]
pub async fn harvest(
    State(state): State<AppState>,
    AuthBasic(auth): AuthBasic,
    Path(gid): Path<i32>,
    Json(cmd): Json<HarvestCommand>,
) -> Result<Json<Guest>, ApiError> {
    // verify
    let PlayerAuth { id, password } = verify_header(auth)?;

    // transaction
    let txn = begin_txn(&state.conn).await?;
    let g = entity::harvest(&txn, id, password, gid, cmd.at).await?;
    txn.commit().await?;

    // return
    Ok(Json(g))
}

#[instrument(skip(state, auth), ret(level = Level::DEBUG), err(level = Level::INFO))]
pub async fn arrange(
    State(state): State<AppState>,
    AuthBasic(auth): AuthBasic,
    Path(gid): Path<i32>,
    Json(cmd): Json<ArrangeCommand>,
) -> Result<Json<Guest>, ApiError> {
    // verify
    let PlayerAuth { id, password } = verify_header(auth)?;

    // transaction
    let txn = begin_txn(&state.conn).await?;
    let new_g = entity::arrange(&txn, id, password, gid, cmd.transfer_energy).await?;
    txn.commit().await?;

    // return
    Ok(Json(new_g))
}

#[instrument(skip(state, auth), ret(level = Level::DEBUG), err(level = Level::INFO))]
pub async fn detect(
    State(state): State<AppState>,
    AuthBasic(auth): AuthBasic,
    Path(gid): Path<i32>,
) -> Result<Json<Vec<DetectedGuest>>, ApiError> {
    // verify
    let PlayerAuth { id, password } = verify_header(auth)?;

    // transaction
    let txn = begin_txn(&state.conn).await?;
    let gs = entity::detect(&txn, id, password, gid).await?;
    txn.commit().await?;

    // return
    Ok(Json(gs))
}

#[instrument(skip(state, auth), ret(level = Level::DEBUG), err(level = Level::INFO))]
pub async fn heat(
    State(state): State<AppState>,
    AuthBasic(auth): AuthBasic,
    Path(gid): Path<i32>,
    Json(cmd): Json<HeatCommand>,
) -> Result<Json<Guest>, ApiError> {
    // verify
    let PlayerAuth { id, password } = verify_header(auth)?;

    // transaction
    let txn = begin_txn(&state.conn).await?;
    let g = entity::heat(&txn, id, password, gid, cmd.at, cmd.energy).await?;
    txn.commit().await?;

    // return
    Ok(Json(g))
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

async fn begin_txn(db: &DatabaseConnection) -> Result<DatabaseTransaction, DbErr> {
    db.begin_with_config(
        Some(IsolationLevel::RepeatableRead), // set isolate level
        Some(AccessMode::ReadWrite),
    )
    .await
}
