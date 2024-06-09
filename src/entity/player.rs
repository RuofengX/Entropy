use crate::{err::OperationError, grid::NodeID};

use super::guest;
use sea_orm::{entity::prelude::*, QuerySelect};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "player")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i32,
    pub name: String,
    pub password: String,
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
    pub async fn count_guest<C: ConnectionTrait>(&self, db: &C) -> Result<u64, OperationError> {
        Ok(self.find_related(guest::Entity).count(db).await?)
    }

    pub async fn list_guest<C: ConnectionTrait>(
        &self,
        db: &C,
    ) -> Result<Vec<guest::Model>, OperationError> {
        Ok(self.find_related(guest::Entity).all(db).await?)
    }

    pub async fn list_guest_id<C: ConnectionTrait>(
        &self,
        db: &C,
    ) -> Result<Vec<i32>, OperationError> {
        Ok(self
            .find_related(guest::Entity)
            .column(guest::Column::Id)
            .into_tuple()
            .all(db)
            .await?)
    }

    pub async fn spawn_guest<C: ConnectionTrait>(
        &self,
        db: &C,
    ) -> Result<guest::Model, OperationError> {
        if self.count_guest(db).await? == 0 {
            Ok(guest::Model::spawn(db, NodeID::SITU.into(), self.id).await?)
        } else {
            Err(OperationError::AlreadyHasGuest)
        }
    }

    pub async fn get_guest<C: ConnectionTrait>(
        &self,
        db: &C,
        gid: i32,
    ) -> Result<guest::Model, OperationError> {
        let g = guest::Entity::find_by_id(gid).one(db).await?;
        if let Some(g) = g {
            if g.master_id == self.id {
                return Ok(g);
            }
        };
        Err(OperationError::GuestNotExist(gid))
    }
}
