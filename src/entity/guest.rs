use axum::async_trait;
use ordered_float::NotNan;
use sea_orm::{
    entity::prelude::*, ActiveValue::NotSet, Condition, DatabaseTransaction, IntoActiveModel, Set,
    Unchanged,
};
use serde::{Deserialize, Serialize};

use crate::{
    err::{ModelError, OperationError},
    grid::{navi, FlatID, Node, NodeID},
};

use super::{node, variant::DetectedGuest};

#[derive(Clone, Copy, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "guest")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub energy: i64,
    #[sea_orm(index)]
    #[serde(
        serialize_with = "crate::grid::ser_flat",
        deserialize_with = "crate::grid::de_flat"
    )]
    pub pos: i32,
    pub temperature: i16, // should be i8, but sea_orm always error
    #[sea_orm(index)]
    pub master_id: i32,
}

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
        from = "Column::Pos",
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

#[async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(self, db: &C, _insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        let pos = self.pos.as_ref();
        node::Model::_ensure(db, FlatID::from(pos.clone()))
            .await
            .map_err(|e| DbErr::Custom(e.to_string()))?;
        Ok(self)
    }
}

pub fn get_carnot_efficiency(one: i8, other: i8) -> f32 {
    let one = one as i16 - i8::MIN as i16;
    let other = other as i16 - i8::MIN as i16;
    let one = unsafe { NotNan::new_unchecked(one as f32) };
    let other = unsafe { NotNan::new_unchecked(other as f32) };
    let (h, c) = if one > other {
        (*one, *other)
    } else {
        (*other, *one)
    };
    1f32 - c / h
}

impl Model {
    pub async fn spawn<C: ConnectionTrait>(
        db: &C,
        pos: NodeID,
        master_id: i32,
    ) -> Result<Model, OperationError> {
        let g = ActiveModel {
            id: NotSet,
            energy: Set(0),
            pos: Set(pos.into_i32()),
            temperature: Set(0),
            master_id: Set(master_id),
        };
        Ok(g.insert(db).await?)
    }

    pub async fn walk<C: ConnectionTrait>(
        &self,
        db: &C,
        to: navi::Direction,
    ) -> Result<Model, OperationError> {
        self.verify_energy(1)?;

        let at = FlatID::from(self.pos).into_node_id().navi_to(to);
        let mut g = self.into_active_model();
        g.pos = Set(at.into_i32());
        g.energy = Set(self.energy - 1);
        Ok(g.update(db).await?)
    }

    /// Introduce a new guest from an existing guest,
    /// transfer energy from the old to new.
    ///
    /// Return the new guest model.
    ///
    /// This method will not take any energy cost, it's FREE
    pub async fn arrange_free(
        &self,
        txn: &DatabaseTransaction,
        transfer_energy: i64,
    ) -> Result<Model, OperationError> {
        self.consume_energy(txn, transfer_energy).await?;

        let to = ActiveModel {
            energy: Set(transfer_energy),
            pos: Set(self.pos),
            temperature: Set(0),
            master_id: Set(self.master_id),
            ..Default::default()
        };

        let to = to.insert(txn).await?;

        Ok(to)
    }

    pub async fn detect<C: ConnectionTrait>(&self, db: &C) -> Result<Vec<DetectedGuest>, OperationError> {
        let gs = Entity::find()
            .filter(
                Condition::all()
                    .add(Column::Id.ne(self.id))
                    .add(Column::Pos.eq(self.pos)),
            )
            .into_partial_model::<DetectedGuest>()
            .all(db)
            .await?;
        Ok(gs)
    }

    /// Consume energy of self energy and update database.
    pub async fn consume_energy<C: ConnectionTrait>(
        &self,
        db: &C,
        energy: i64,
    ) -> Result<Model, OperationError> {
        self.verify_energy(energy)?;

        let mut g = self.into_active_model();
        g.energy = Set(self.energy - energy);
        let g: Model = g.update(db).await?;

        Ok(g)
    }

    pub fn get_efficiency(&self, cell: i8) -> f32 {
        get_carnot_efficiency(self.temperature as i8, cell)
    }

    // generate two middle model, handler use these model to do things left
    pub fn _generate_active_model(
        self,
        node: Node,
        cell_i: usize,
    ) -> Result<(self::ActiveModel, node::ActiveModel), ModelError> {
        let mut data = node.data.clone();
        let mut cell = node.data.get(cell_i).ok_or(ModelError::Parse {
            desc: format!(
                "request length({1}) out of range <- node({0:?})",
                node.id, cell_i
            ),
        })?;

        let mut g = self.into_active_model();

        // Calculate the delta energy first
        let temp = self.temperature as i8;
        let delta = temp.abs_diff(cell);
        let delta = (self.get_efficiency(cell) * delta as f32).div_euclid(2.0) as u8;

        // no overflow will happen, the efficiency proves that, so no need to check

        // Determine which temperature is hotter and colder.
        // and go change
        if temp > cell {
            g.temperature = Set(temp.saturating_sub_unsigned(delta) as i16);
            cell = cell.saturating_add_unsigned(delta);
        } else if temp < cell {
            g.temperature = Set(temp.saturating_add_unsigned(delta) as i16);
            cell = cell.saturating_sub_unsigned(delta);
        } else {
            ()
        };
        g.energy = Set(self.energy + delta as i64);
        data.set(cell_i, cell);
        let n = node::ActiveModel {
            id: Unchanged(node.id.into_i32()),
            data: Set(data.into()),
        };
        Ok((g, n))
    }

    /// Check if Guest has enough energy.
    ///
    /// Return Ok(()) if energy is enough
    /// Return Err if energy is not enough
    fn verify_energy(&self, require: i64) -> Result<(), OperationError> {
        if self.energy >= require {
            Ok(())
        } else {
            Err(OperationError::EnergyNotEnough {
                require: require,
                reserve: self.energy,
            })
        }
    }
}
