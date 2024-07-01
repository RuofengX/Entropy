use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, DatabaseTransaction, EntityTrait,
    QueryFilter, Set,
};
use variant::PublicPlayer;

use crate::err::{ModelError, OperationError};
use entropy_base::grid::NodeID;

pub mod guest;
pub mod node;
pub mod player;
pub mod prelude;
pub mod variant;

pub async fn get_node(
    txn: &DatabaseTransaction,
    node_id: NodeID,
) -> Result<node::Model, OperationError> {
    Ok(node::Model::get_or_init(txn, node_id.into()).await?)
}

pub async fn register_player<C: ConnectionTrait>(
    db: &C,
    name: String,
    password: String,
) -> Result<player::Model, OperationError> {
    let p = player::ActiveModel {
        name: Set(name),
        password: Set(password),
        ..Default::default()
    };
    Ok(p.insert(db).await?)
}

pub async fn get_player<C: ConnectionTrait>(
    db: &C,
    id: i32,
    password: String,
) -> Result<Option<player::Model>, OperationError> {
    if let Some(player) = player::Entity::find()
        .filter(
            Condition::all()
                .add(player::Column::Id.eq(id))
                .add(player::Column::Password.eq(password)),
        )
        .one(db)
        .await?
    {
        Ok(Some(player))
    } else {
        Ok(None)
    }
}

pub async fn get_exact_player<C: ConnectionTrait>(
    db: &C,
    id: i32,
    password: String,
) -> Result<player::Model, OperationError> {
    if let Some(player) = player::Entity::find()
        .filter(
            Condition::all()
                .add(player::Column::Id.eq(id))
                .add(player::Column::Password.eq(password)),
        )
        .one(db)
        .await?
    {
        Ok(player)
    } else {
        Err(OperationError::PlayerNotExist(id))
    }
}

pub async fn get_exact_player_public<C: ConnectionTrait>(
    db: &C,
    id: i32,
) -> Result<PublicPlayer, OperationError> {
    if let Some(p) = player::Entity::find_by_id(id)
        .into_partial_model::<PublicPlayer>()
        .one(db)
        .await?
    {
        Ok(p)
    } else {
        Err(OperationError::PlayerNotExist(id))
    }
}

pub async fn list_guest(
    txn: &DatabaseTransaction,
    id: i32,
    password: String,
) -> Result<Vec<guest::Model>, OperationError> {
    Ok(get_player(txn, id, password)
        .await?
        .ok_or(OperationError::PlayerNotExist(id))?
        .list_guest(txn)
        .await?)
}

pub async fn spawn_guest(
    txn: &DatabaseTransaction,
    id: i32,
    password: String,
) -> Result<guest::Model, OperationError> {
    let p = get_exact_player(txn, id, password).await?;
    let rnt = p.spawn_guest(txn).await?;
    Ok(rnt)
}

pub async fn get_guest(
    txn: &DatabaseTransaction,
    id: i32,
    password: String,
    gid: i32,
) -> Result<guest::Model, OperationError> {
    let p = get_exact_player(txn, id, password).await?;
    let g = p.get_guest(txn, gid).await?;
    Ok(g)
}

pub async fn walk(
    txn: &DatabaseTransaction,
    id: i32,
    password: String,
    gid: i32,
    to: (i16, i16),
) -> Result<guest::Model, OperationError> {
    // get guest
    let g = get_exact_player(txn, id, password)
        .await?
        .get_guest(txn, gid)
        .await?;

    // move guest, more easily rollback than node change
    let g_next = g.walk_free(txn, to).await?;

    // exhaust wasted heat
    let n = get_node(txn, NodeID::from_i32(g.pos)).await?; // use old guest position
    let _n = n._walk_exhaust(txn).await?;

    Ok(g_next)
}

pub async fn harvest(
    txn: &DatabaseTransaction,
    id: i32,
    password: String,
    gid: i32,
    at: usize,
) -> Result<guest::Model, OperationError> {
    let g = get_exact_player(txn, id, password)
        .await?
        .get_guest(txn, gid)
        .await?;

    let n = get_node(&txn, NodeID::from_i32(g.pos)).await?;
    let (g, n) = g
        ._harvest_active_model(n.into(), at)
        .map_err(|e| OperationError::Model(e))?;
    let g = g.update(txn).await?;
    n.update(txn).await?;
    Ok(g)
}

pub async fn arrange(
    txn: &DatabaseTransaction,
    id: i32,
    password: String,
    gid: i32,
    transfer_energy: i64,
) -> Result<guest::Model, OperationError> {
    let p = get_exact_player(txn, id, password).await?;
    let g = p.get_guest(txn, gid).await?;

    // consume energy
    let g_count = p.count_guest(txn).await?;
    let g_count = g_count.try_into().map_err(|_| {
        OperationError::Model(ModelError::OutOfLimit {
            desc: format!("owned guest number"),
            limit_type: "u32",
        })
    })?;
    let consume_energy = 2i64.pow(g_count);
    let g = g.consume_energy(txn, consume_energy).await?;
    let new_g = g.arrange_free(txn, transfer_energy).await?;

    Ok(new_g)
}

pub async fn detect(
    txn: &DatabaseTransaction,
    id: i32,
    password: String,
    gid: i32,
) -> Result<Vec<variant::DetectedGuest>, OperationError> {
    let p = get_exact_player(txn, id, password).await?;
    let g = p.get_guest(txn, gid).await?;
    let gs = g.detect(txn).await?;
    Ok(gs)
}

pub async fn heat(
    txn: &DatabaseTransaction,
    id: i32,
    password: String,
    gid: i32,
    at: usize,
    energy: i64,
) -> Result<guest::Model, OperationError> {
    let g = get_exact_player(txn, id, password)
        .await?
        .get_guest(txn, gid)
        .await?;
    let n = get_node(txn, NodeID::from_i32(g.pos)).await?;
    n._heat(txn, at, energy).await?;
    let g = g.consume_energy(txn, energy).await?;
    Ok(g)
}
