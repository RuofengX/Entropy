use sea_orm::{DbBackend, DbErr, EntityTrait};

use crate::grid::FlatID;

pub mod guest;
pub mod node;
pub mod player;

pub async fn get_node(db: &DbBackend, node_id: FlatID) -> Result<node::Model, DbErr>{
    if let Some(node) = node::Entity::find_by_id(node_id).one(db).await?{
        Ok(node)
    } else {
        node::Entity::insert(model)
    }
};