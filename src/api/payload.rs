use crate::guest::GID;
use serde::Deserialize;

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
}

pub mod ws {
    use super::*;

    #[derive(Debug, Deserialize)]
    pub struct DetectCommand{
        pub fmt: String, // ["json", "bytes"]
        pub x: i16,
        pub y: i16,
    }

}
