use sea_orm::entity::prelude::*;

use crate::node;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "node")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: node::FlatID,
    pub data: Vec<u8>,
    pub guests: Vec<u32>,
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
