use ordered_float::NotNan;
use sea_orm::{entity::prelude::*, IntoActiveModel, Set, TransactionTrait, Unchanged};
use serde::{Deserialize, Serialize};

use crate::{
    err::{ModelError, OperationError},
    grid,
};

use super::node::{self};

#[derive(Clone, Copy, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
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
    pub fn get_efficiency(&self, cell: u8) -> f32 {
        get_carnot_efficiency(self.temperature, cell)
    }

    pub async fn harvest(
        self,
        db: &DbConn,
        cell_i: usize,
    ) -> Result<(self::Model, node::Model), OperationError> {
        let txn = db.begin().await?; // some error magic to this sugar
        let node = node::Model::get_or_init(&txn, self.position).await?;
        let (guest, node) = self.generate(node, cell_i)?;
        let modified_guest = guest.update(&txn).await?;
        let modified_node = node.update(&txn).await?;
        txn.commit().await?;
        Ok((modified_guest, modified_node))
    }

    fn generate(
        self,
        node: node::Model,
        cell_i: usize,
    ) -> Result<(self::ActiveModel, node::ActiveModel), ModelError> {
        let mut data = node.data;
        let cell = data.get_mut(cell_i).ok_or(ModelError::Parse {
            desc: format!(
                "request length({1}) out of range <- node({0})",
                node.id, cell_i
            ),
        })?;

        let mut g = self.into_active_model();

        // Calculate the delta energy first
        let temp = self.temperature;
        let delta = self.temperature.abs_diff(*cell);
        let delta = (self.get_efficiency(*cell) * delta as f32).floor() as u8;

        // no overflow will happen, the efficiency proves that, so no need to check

        // Determine which temperature is hotter and colder.
        // and go change
        if temp > *cell {
            g.temperature = Set(temp - delta);
            *cell += delta;
        } else if temp < *cell {
            g.temperature = Set(temp + delta);
            *cell -= delta;
        } else {
            ()
        };
        g.energy = Set(self.energy + delta as u64);
        let n = node::ActiveModel {
            id: Unchanged(node.id),
            data: Set(data),
        };
        Ok((g, n))
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
