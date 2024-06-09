use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, DbConn, DbErr, EntityTrait,
    QueryFilter, Set, TransactionTrait,
};

use crate::err::{OperationError, RuntimeError};

pub mod guest;
pub mod node;
pub mod player;

pub async fn check_database(db: &DbConn) -> Result<(), RuntimeError> {
    Ok(db.ping().await?)
}

pub async fn get_node(db: &DbConn, node_id: i32) -> Result<node::Model, DbErr> {
    db.transaction::<_, node::Model, OperationError>(|txn| {
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
