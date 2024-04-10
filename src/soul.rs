use ahash::HashMap;
use thiserror::Error;
use std::sync::Arc;

use crate::{guest::GID, node::{self, direction, NodeID}, world::World};

struct GuestList(HashMap<GID, Arc<World>>);

pub struct Soul {
    pub id: u64,
    pub username: String,
    password: String,
    owned_guest: HashMap<GID, Arc<World>>,
}
impl Soul {
    pub fn walk(&mut self, id: GID, direction: direction::Direction) -> Result<NodeID, SoulError> {
        let guest = self
            .owned_guest
            .get(&id)
            .unwrap()
            .get_guest(id)
            .ok_or(SoulError::GuestNotExist)?;
        let rtn = {
            let mut guest_ctx = guest.write().unwrap();
            let cost = guest_ctx.walk_cost;
            guest_ctx.energy -= cost;
            guest_ctx.node.walk(direction)?
        };
        Ok(rtn)
    }
}

#[derive(Error, Debug)]
pub enum SoulError {
    #[error("guest is recorded in soul's memory, but not found in physical world.")]
    GuestNotExist,
    #[error(transparent)]
    NodeError(#[from] node::NodeError)
}
