use sea_orm::{DerivePartialModel, FromQueryResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, DerivePartialModel, FromQueryResult)]
#[sea_orm(entity = "super::player::Entity")]
pub struct DetectedPlayer {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, DerivePartialModel, FromQueryResult)]
#[sea_orm(entity = "super::guest::Entity")]
pub struct DetectedGuest {
    pub id: i32,
    pub temperature: i16,
    #[serde(
        serialize_with = "crate::grid::ser_flat",
        deserialize_with = "crate::grid::de_flat"
    )]
    pub pos: i32,
    pub master_id: i32,
}
