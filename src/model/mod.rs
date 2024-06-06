use sea_orm::{ActiveModelTrait, DbConn, DbErr, EntityTrait, QuerySelect, Set, TransactionTrait};

use crate::grid::FlatID;

pub mod guest;
pub mod node;
pub mod player;

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
) -> Result<player::Model, DbErr> {
    let p = player::ActiveModel {
        name: Set(name),
        password: Set(password),
        ..Default::default()
    };
    p.insert(db).await
}

pub async fn get_player(
    db: &DbConn,
    id: u32,
    passwd: String,
) -> Result<Option<player::Model>, DbErr> {
    if let Some(player) = player::Entity::find_by_id(id)
        .select_only()
        .column(player::Column::Password)
        .one(db)
        .await?
    {
        if passwd == player.password {
            Ok(Some(player))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}
