use sea_orm::{entity::prelude::*, DatabaseTransaction, Set};
use serde::{Deserialize, Serialize};

use crate::{
    err::{OperationError, RuntimeError},
    grid::{FlatID, NodeData, NodeID},
};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "node")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
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
    pub async fn get_or_init(
        txn: &DatabaseTransaction,
        id: FlatID,
    ) -> Result<Model, OperationError> {

        if let Some(node) = Entity::find_by_id::<FlatID>(id)
            .one(txn)
            .await?
        {
            Ok(node)
        } else {
            let n = ActiveModel {
                id: Set(id.into()),
                data: Set(NodeData::random().into()),
            };
            Ok(n.insert(txn).await?)
        }
    }
    pub async fn prepare_origin(txn: &DatabaseTransaction) -> Result<(), RuntimeError> {
        if let Some(_) = Entity::find_by_id(NodeID::SITU.into_i32()).one(txn).await? {
            return Ok(());
        } else {
            let n = ActiveModel {
                id: Set(NodeID::SITU.into_i32()),
                data: Set(NodeData::random().into()),
            };
            n.insert(txn).await?;
            Ok(())
        }
    }
}
