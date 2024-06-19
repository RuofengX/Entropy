use rand::{rngs::SmallRng, seq::IteratorRandom, SeedableRng};
use sea_orm::{entity::prelude::*, sea_query::OnConflict, DatabaseTransaction, Set};
use serde::{Deserialize, Serialize};

use crate::{
    err::{ModelError, OperationError, RuntimeError},
    grid::{FlatID, NodeData, NodeID},
};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "node")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    #[serde(
        serialize_with = "crate::grid::ser_flat",
        deserialize_with = "crate::grid::de_flat"
    )]
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
    // this method MAY use quary multiple times, so transaction is required
    pub async fn get_or_init(
        txn: &DatabaseTransaction,
        id: NodeID,
    ) -> Result<Model, OperationError> {
        if let Some(node) = Entity::find_by_id(id.into_flat()).one(txn).await? {
            Ok(node)
        } else {
            let n = ActiveModel {
                id: Set(id.into_i32()),
                data: Set(NodeData::random().into()),
            };
            Ok(n.insert(txn).await?)
        }
    }

    pub async fn prepare_origin<C: ConnectionTrait>(db: &C) -> Result<(), RuntimeError> {
        let n = ActiveModel {
            id: Set(NodeID::SITU.into_i32()),
            data: Set(NodeData::random().into()),
        };
        Entity::insert(n)
            .on_conflict(OnConflict::column(Column::Id).do_nothing().to_owned())
            .do_nothing()
            .exec(db)
            .await?;
        Ok(())
    }

    // walk排放费热，不可直接使用
    pub async fn _walk_exhaust<C: ConnectionTrait>(
        mut self,
        db: &C,
    ) -> Result<Model, OperationError> {
        // 循环直到找到一个非 u8::MAX 的字节
        let mut rng = SmallRng::from_entropy();

        while let Some((i, cell)) = self.data.iter().enumerate().choose(&mut rng) {
            if *cell < u8::MAX {
                // 一旦条件满足则直接返回，不涉及下一次循环，所以这里不会再进行借用检查
                self.data[i] += 1;
                let n = ActiveModel {
                    id: Set(self.id),
                    data: Set(self.data),
                };
                return Ok(n.update(db).await?);
            }
        }
        Err(OperationError::NodeTemperatureTooHigh(NodeID::from_i32(
            self.id,
        )))
    }

    // 直接加热，不可直接使用
    pub async fn _heat<C: ConnectionTrait>(
        mut self,
        db: &C,
        i: usize,
        energy: i64,
    ) -> Result<Model, OperationError> {
        let len = self.data.len();
        let cell = self
            .data
            .get_mut(i)
            .ok_or(OperationError::CellIndexOutOfRange {
                node: NodeID::from_i32(self.id),
                require: i,
                max: len,
            })?;
        *cell = cell
            .checked_add(energy.try_into().map_err(|_| ModelError::OutOfLimit {
                desc: "heat energy".to_string(),
                limit_type: "u8",
            })?)
            .ok_or(OperationError::CellTemperatureTooHigh {
                node: NodeID::from_i32(self.id),
                index: i,
            })?;
        let n = ActiveModel {
            id: Set(self.id),
            data: Set(self.data),
        };
        Ok(n.update(db).await?)
    }

    pub async fn _ensure<C: ConnectionTrait>(db: &C, id: FlatID) -> Result<(), OperationError> {
        let n = ActiveModel {
            id: Set(id.into()),
            data: Set(NodeData::random().into()),
        };
        Entity::insert(n)
            .on_conflict(OnConflict::column(Column::Id).do_nothing().to_owned())
            .do_nothing()
            .exec(db)
            .await?;
        Ok(())
    }
}
