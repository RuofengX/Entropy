use sea_orm::{entity::prelude::*, DatabaseTransaction, IntoActiveModel};
use serde::{Deserialize, Serialize};

use crate::grid;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "node")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: grid::FlatID,
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
    pub fn random(id: grid::FlatID) -> Model {
        Model {
            id,
            data: grid::NodeData::random().to_vec(),
        }
    }
    pub async fn get_or_init(txn: &DatabaseTransaction, id: grid::FlatID) -> Result<Model, DbErr> {
        if let Some(node) = Entity::find_by_id(id).one(txn).await? {
            Ok(node)
        } else {
            Model::random(id).into_active_model().insert(txn).await
        }
    }
}
