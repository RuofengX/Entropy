use ordered_float::NotNan;
use sea_orm::{entity::prelude::*, TransactionTrait};

use crate::grid;

use super::node;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "guest")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u32,
    pub energy: u64,
    #[sea_orm(index)]
    pub position: grid::FlatID,
    pub temperature: u8,
    #[sea_orm(index)]
    pub master_id: u32,
}

impl Model {
    fn get_efficiency(&self, cell: u8) -> f32 {
        get_carnot_efficiency(self.temperature, cell)
    }
}
impl ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::player::Entity",
        from = "Column::MasterId",
        to = "super::player::Column::Id"
    )]
    Player,
    #[sea_orm(
        belongs_to = "super::node::Entity",
        from = "Column::Position",
        to = "super::node::Column::Id"
    )]
    Node,
}

impl Related<super::player::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Player.def()
    }
}

impl Related<super::node::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Node.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

pub fn get_carnot_efficiency(one: u8, other: u8) -> f32 {
    let one = unsafe { NotNan::new_unchecked(one as f32) };
    let other = unsafe { NotNan::new_unchecked(other as f32) };
    let (h, c) = if one > other {
        (*one, *other)
    } else {
        (*other, *one)
    };
    1f32 - c / h
}
