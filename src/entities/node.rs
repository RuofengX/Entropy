use sea_orm::{entity::prelude::*, Set};
use serde::{Deserialize, Serialize};

use crate::{
    err::{OperationError, RuntimeError},
    grid::{NodeData, NodeID},
};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "node")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::guest::Entity")]
    Guest,
}

impl Related<super::guest::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Guest.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub async fn get_or_init<C: ConnectionTrait>(db: &C, id: i32) -> Result<Model, OperationError> {
        if let Some(node) = Entity::find_by_id(id).one(db).await? {
            Ok(node)
        } else {
            let n = ActiveModel {
                id: Set(id),
                data: Set(NodeData::random().into()),
            };
            Ok(n.insert(db).await?)
        }
    }
    pub async fn prepare_origin<C: ConnectionTrait>(db: &C) -> Result<(), RuntimeError> {
        if let Some(_) = Entity::find_by_id::<NodeID>(NodeID::ORIGIN).one(db).await? {
            return Ok(());
        } else {
            let n = ActiveModel {
                id: Set(NodeID::ORIGIN.into()),
                data: Set(NodeData::random().into()),
            };
            n.insert(db).await?;
            Ok(())
        }
    }
}
