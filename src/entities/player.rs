use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use super::guest;


#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "player")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u32,
    pub name: String,
    pub password: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::guest::Entity")]
    Guest
}

impl Related<super::guest::Entity> for Entity{
    fn to() -> RelationDef {
        Relation::Guest.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model{
    pub async fn list_guest(&self, db: &DbConn) -> Result<Vec<guest::Model>, DbErr>{
        self.find_related(guest::Entity).all(db).await
    }
}