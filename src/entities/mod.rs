use sea_orm::{ActiveModelTrait, DbConn, DbErr, EntityTrait, QuerySelect, Set, TransactionTrait};

use crate::{
    err::{OperationError, RuntimeError},
    grid::FlatID,
};

pub mod guest;
pub mod node;
pub mod player;

pub async fn check_database(db: &DbConn) -> Result<(), RuntimeError> {
    Ok(db.ping().await?)
}

pub async fn get_node(db: &DbConn, node_id: FlatID) -> Result<node::Model, DbErr> {
    db.transaction::<_, node::Model, DbErr>(|txn| {
        Box::pin(async move { node::Model::get_or_init(txn, node_id).await })
    })
    .await
    .map_err(|e| DbErr::Custom(e.to_string()))
}

pub async fn register_player(
    db: &DbConn,
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

pub async fn get_player(
    db: &DbConn,
    id: i32,
    password: String,
) -> Result<Option<player::Model>, OperationError> {
    if let Some(player) = player::Entity::find_by_id(id)
        .select_only()
        .column(player::Column::Password)
        .one(db)
        .await?
    {
        if password == player.password {
            Ok(Some(player))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}