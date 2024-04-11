use dashmap::DashSet;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;

use crate::{
    guest::{self, Guest, GID},
    node::{direction::Direction, NodeID},
    world::World,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Soul {
    pub uid: String,
    pub username: String,
    password: String,
    guest_quota: u64,
    guests: DashSet<GID, ahash::RandomState>,
}

#[derive(Debug)]
pub struct WonderingSoul {
    soul: Soul,
    world: Arc<World>,
}

impl WonderingSoul {
    pub fn new(
        world: Arc<World>,
        username: String,
        password: String,
        guest_quota: u64,
        guests: Vec<GID>,
    ) -> Self {
        Self {
            soul: Soul {
                uid: nanoid!(),
                username,
                password,
                guest_quota,
                guests: guests.into_iter().collect(),
            },
            world,
        }
    }
    pub fn from_soul(soul: Soul, world: Arc<World>) -> Self {
        Self { soul, world }
    }
}
impl WonderingSoul {
    pub fn challenge_password(&self, password: String) -> bool {
        self.soul.password == password
    }

    pub fn list_guest(&self) -> Vec<GID> {
        self.soul.guests.iter().map(|x| x.key().clone()).collect()
    }

    pub fn spawn_guest(&self) -> Result<GID, SoulError> {
        if self.soul.guests.len() as u64 > self.soul.guest_quota {
            Err(SoulError::GuestQuotaExceeded(self.soul.guest_quota))
        } else {
            let id = self.world.spawn();
            self.soul.guests.insert(id);
            Ok(id)
        }
    }

    pub fn get_guest(&self, id: GID) -> Result<Guest, SoulError> {
        if self.soul.guests.contains(&id) {
            self.world
                .get_guest(id)
                .ok_or(SoulError::GuestNotExistInWorld(id))
                .map(|x| x.value().read().unwrap().clone())
        } else {
            Err(SoulError::GuestNotConnected(id))
        }
    }

    pub fn move_guest(&mut self, id: GID, to: Direction) -> Result<NodeID, SoulError> {
        if self.soul.guests.contains(&id) {
            self.world
                .get_guest(id)
                .ok_or(SoulError::GuestNotExistInWorld(id))
                .and_then(|x| x.value().write().unwrap().walk(to).map_err(|e| e.into()))
        } else {
            Err(SoulError::GuestNotConnected(id))
        }
    }
}

#[derive(Error, Debug)]
pub enum SoulError {
    #[error("GID::{0:?} is not recorded in soul's memory")]
    GuestNotConnected(GID),
    #[error("guest with GID::{0:?} is recorded in soul's memory, but not found in physical world")]
    GuestNotExistInWorld(GID),
    #[error("guest quota::{0} has been exceeded")]
    GuestQuotaExceeded(u64),
    #[error(transparent)]
    GuestError(#[from] guest::GuestError),
}
