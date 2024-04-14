use dashmap::DashSet;
use futures::{future, TryFutureExt};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;

use crate::{
    db::SaveStorage,
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
pub struct WonderingSoul<S: SaveStorage> {
    soul: Soul,
    world: Arc<World<S>>,
}

impl<S: SaveStorage> WonderingSoul<S> {
    pub fn new(
        world: Arc<World<S>>,
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
    pub fn from_soul(soul: Soul, world: Arc<World<S>>) -> Self {
        Self { soul, world }
    }
}
impl<S: SaveStorage> WonderingSoul<S> {
    pub fn challenge_password(&self, password: String) -> bool {
        self.soul.password == password
    }

    pub fn list_guest(&self) -> Vec<GID> {
        self.soul.guests.iter().map(|x| x.key().clone()).collect()
    }

    pub async fn spawn_guest(&self) -> Result<GID, SoulError> {
        if self.soul.guests.len() as u64 > self.soul.guest_quota {
            Err(SoulError::GuestQuotaExceeded(self.soul.guest_quota))
        } else {
            let id = self.world.spawn().await;
            self.soul.guests.insert(id);
            Ok(id)
        }
    }

    pub async fn get_guest(&self, id: GID) -> Result<Guest, SoulError> {
        if self.soul.guests.contains(&id) {
            self.world
                .get_guest(id)
                .await
                .ok_or(SoulError::GuestNotExistInWorld(id))
                .map(|x| x.clone())
        } else {
            Err(SoulError::GuestNotConnected(id))
        }
    }

    pub async fn move_guest(&mut self, id: GID, to: Direction) -> Result<NodeID, SoulError> {
        if self.soul.guests.contains(&id) {
            future::ready(
                self.world
                    .get_guest(id)
                    .await
                    .ok_or(SoulError::GuestNotExistInWorld(id)),
            )
            .and_then(|x| async move { x.walk(to).map_err(|e| e.into()) })
            .await
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
