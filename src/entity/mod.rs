use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, DatabaseTransaction, EntityTrait,
    QueryFilter, Set,
};

use crate::{err::OperationError, grid::NodeID};

pub mod guest;
pub mod node;
pub mod player;
pub mod prelude;

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
