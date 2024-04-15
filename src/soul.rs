use ahash::HashSet;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    db::SaveStorage,
    err::{Result, SoulError},
    guest::GID,
    node::{direction::Direction, NodeID},
    world::World,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Soul {
    pub uid: String,
    pub username: String,
    password: String,
    guest_quota: u64,
    guests: HashSet<GID>,
}

pub struct WounderingSoul<'w, S: SaveStorage> {
    soul: Soul,
    world: &'w World<S>,
}
impl<'w, S: SaveStorage> WounderingSoul<'w, S> {
    pub(crate) fn contains_guest(&self, id: GID) -> bool {
        self.soul.guests.contains(&id)
    }

    pub(crate) async fn walk(&self, id: GID, direction: Direction) -> Result<NodeID> {
        if !self.contains_guest(id) {
            return Err(SoulError::GuestNotConnected(id).into());
        }
        if self.world.contains_guest(id).await {
            return Err(SoulError::GuestNotExistInWorld(id).into());
        }

        let rtn = self
            .world
            .modify_guest_with(id, |g| {
                g.node_move(direction)?;
            })
            .await;

        Ok(rtn)
    }
}
