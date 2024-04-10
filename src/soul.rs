use dashmap::{mapref::one::Ref, DashSet};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, RwLock,
};
use thiserror::Error;

use crate::{
    guest::{Guest, GID},
    node::{self, direction::Direction, NodeID},
    world::World,
};

struct GuestList {
    guest_id: DashSet<GID, ahash::RandomState>,
    world: Arc<World>,
}
impl GuestList {
    fn new(world: Arc<World>) -> Self {
        GuestList {
            guest_id: DashSet::default(),
            world,
        }
    }

    fn len(&self) -> usize {
        self.guest_id.len()
    }

    fn list_all(&self) -> Vec<GID> {
        self.guest_id.iter().map(|x| x.clone()).collect()
    }

    fn get_guest(&self, id: GID) -> Result<Ref<GID, RwLock<Guest>, ahash::RandomState>, SoulError> {
        if self.guest_id.contains(&id) {
            self.world
                .get_guest(id)
                .ok_or(SoulError::GuestNotExistInWorld(id))
        } else {
            Err(SoulError::GuestNotConnected(id))
        }
    }

    fn spawn_guest(&self) -> GID {
        let id = self.world.spawn();
        self.guest_id.insert(id);
        id
    }
}

pub struct Soul {
    pub id: u64,
    pub username: String,
    password: String,
    guest_quota: u64,
    guest: GuestList,
}
impl Soul {
    pub fn new(world: Arc<World>, username: String, password: String, guest_quota: u64) -> Self {
        static SOUL_ID: AtomicU64 = AtomicU64::new(0);
        Self {
            id: SOUL_ID.fetch_add(1, Ordering::AcqRel),
            username,
            password,
            guest_quota,
            guest: GuestList::new(world),
        }
    }

    pub fn challenge_password(&self, password: String) -> bool {
        self.password == password
    }

    pub fn list_guest(&self) -> Vec<GID> {
        self.guest.list_all()
    }

    pub fn spawn_guest(&self) -> Result<GID, SoulError> {
        if self.guest.len() as u64 > self.guest_quota {
            Err(SoulError::GuestQuotaExceeded(self.guest_quota))
        } else {
            Ok(self.guest.spawn_guest())
        }
    }
    pub fn get_guest(&self, id: GID) -> Result<Guest, SoulError> {
        self.guest
            .get_guest(id)
            .and_then(|g| Ok(g.read().unwrap().clone()))
    }

    pub fn move_guest(&mut self, id: GID, to: Direction) -> Result<NodeID, SoulError> {
        let guest = self.guest.get_guest(id)?;
        Ok({
            let mut guest_ctx = guest.write().unwrap();
            let cost = guest_ctx.walk_cost;
            guest_ctx.energy -= cost;
            guest_ctx.node.walk(to)?
        })
    }
}

#[derive(Error, Debug)]
pub enum SoulError {
    #[error("GID::{0:?} is not recorded in soul's memory")]
    GuestNotConnected(GID),
    #[error("guest with GID::{0:?} is recorded in soul's memory, but not found in physical world")]
    GuestNotExistInWorld(GID),
    #[error(transparent)]
    NodeError(#[from] node::NodeError),
    #[error("guest quota::{0} has been exceeded")]
    GuestQuotaExceeded(u64),
}
