use ahash::HashMap;
use std::sync::{Arc, RwLock};
use thiserror::Error;

use crate::{
    guest::{Guest, GID},
    node::{self, direction::Direction, NodeID},
    world::World,
};

struct GuestList(HashMap<GID, Arc<World>>);
impl GuestList {
    fn list_all(&self) -> Vec<GID> {
        self.0.keys().into_iter().map(|x| x.clone()).collect()
    }
    fn get_guest(&self, id: GID) -> Result<&RwLock<Guest>, SoulError> {
        self.0
            .get(&id)
            .ok_or(SoulError::GuestNotExistInWorld)?
            .get_guest(id)
            .ok_or(SoulError::GuestNotExistInWorld)
    }
}

pub struct Soul {
    pub id: u64,
    pub username: String,
    password: String,
    guest: GuestList,
}
impl Soul {
    pub fn list_guest(&self) -> Vec<GID> {
        self.guest.list_all()
    }

    pub fn get_guest(&self, id: GID) -> Result<Guest, SoulError> {
        self.guest
            .get_guest(id)
            .and_then(|g| Ok(g.read().unwrap().clone()))
    }

    pub fn move_guest(&mut self, id: GID, direction: Direction) -> Result<NodeID, SoulError> {
        let guest = self.guest.get_guest(id)?;
        Ok({
            let mut guest_ctx = guest.write().unwrap();
            let cost = guest_ctx.walk_cost;
            guest_ctx.energy -= cost;
            guest_ctx.node.walk(direction)?
        })
    }
}

#[derive(Error, Debug)]
pub enum SoulError {
    #[error("guest is not recorded in soul's memory")]
    GuestNotConnected,
    #[error("guest is recorded in soul's memory, but not found in physical world")]
    GuestNotExistInWorld,
    #[error(transparent)]
    NodeError(#[from] node::NodeError),
}
