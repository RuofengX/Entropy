use ordered_float::NotNan;
use sea_orm::{entity::prelude::*, IntoActiveModel, Set, TransactionError, TransactionTrait};

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
    pub fn get_efficiency(&self, cell: u8) -> f32 {
        get_carnot_efficiency(self.temperature, cell)
    }

    pub async fn harvest(&self, db: &DbConn, cell_i: usize) -> Result<Self, DbErr> {
        db.transaction::<_, Model, DbErr>(|txn| {
            Box::pin(async move {
                let mut n = node::Model::get_or_init(txn, self.position)
                    .await?
                    .into_active_model();
                let (g, n) = self.generate(n.data.into_value()[cell_i]);
                let rtn = g.save(txn).await?;
                n.save(txn).await?;
                Ok(rtn)
            })
        }).await.map_err(|e|)
    }
    fn generate(&self, node: &node::Model, cell_i: usize) -> (ActiveModel, node::ActiveModel) {
        // Calculate the delta energy first
        let temp = self.temperature;
        let mut data = node.data.clone();
        let cell = &mut data[cell_i];
        let delta = self.temperature.abs_diff(*cell);
        let delta = (self.get_efficiency(*cell) * delta as f32).floor() as u8;

        // no overflow will happen, the efficiency proves that, so no need to check

        // Determine which temperature is hotter and colder.
        // and go change
        let mut g = self.into_active_model();
        let mut n = node.into_active_model();
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
        n.data = Set(node.data.clone());
        (g, n)
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
