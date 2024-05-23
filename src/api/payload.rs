use serde::Deserialize;
use crate::guest::GID;

#[derive(Debug, Deserialize)]
pub struct RegisterInfo {
    pub name: String,
    pub pw_hash: String,
}

#[derive(Debug, Deserialize)]
pub struct WalkCommand {
    pub id: GID,
    pub x: i8,
    pub y: i8,
}

#[derive(Debug, Deserialize)]
pub struct HarvestCommand {
    pub id: GID,
    pub at: usize,
}

#[derive(Debug, Deserialize)]
pub struct HeatCommand {
    pub id: GID,
    pub at: usize,
    pub energy: u8, // TODO change this to u64
}

#[derive(Debug, Deserialize)]
pub struct SpawnCommand {
    pub id: GID,
    pub energy: u64,
}
