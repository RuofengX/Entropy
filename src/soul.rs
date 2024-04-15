use ahash::HashSet;
use anyhow::Ok;
use futures::future::join_all;
use serde::{Deserialize, Serialize};

use crate::{
    db::SaveStorage,
    err::{GuestError, Result, SoulError},
    guest::{Guest, GID},
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

pub struct WonderingSoul<'w, S: SaveStorage> {
    soul: Soul,
    world: &'w World<S>,
}
impl<'w, S: SaveStorage> WonderingSoul<'w, S> {
    pub fn contains_guest(&self, id: GID) -> bool {
        self.soul.guests.contains(&id)
    }

    pub async fn get_guest(&self, id: GID) -> Result<Guest> {
        if self.contains_guest(id) {
            self.world.get_guest(id).await
        } else {
            Err(GuestError::NotExist(id).into())
        }
    }

    pub async fn list_guests(&self) -> Vec<Guest> {
        let rtn = join_all(self.soul.guests.iter().map(|id| self.world.get_guest(*id)));
        rtn.await.into_iter().filter_map(|x| x.ok()).collect()
    }

    pub async fn list_phantom_guest(&self) -> Vec<GID> {
        join_all(
            self.soul
                .guests
                .iter()
                .map(|id| async move { (id, self.world.contains_guest(*id).await) }),
        )
        .await
        .iter()
        .filter_map(|(&id, g)| if !g { Some(id) } else { None })
        .collect()
    }

    pub async fn disconnect_guest(&mut self, id: GID) -> Result<Guest> {
        let guest = self.get_guest(id).await?;
        if self.soul.guests.take(&id).is_some() {
            Ok(guest)
        } else {
            Err(SoulError::GuestNotConnected(id).into())
        }
    }

    pub async fn walk(&self, id: GID, to: Direction) -> Result<NodeID> {
        if !self.contains_guest(id) {
            return Err(SoulError::GuestNotConnected(id).into());
        }
        if self.world.contains_guest(id).await {
            return Err(GuestError::NotExist(id).into());
        };

        self.world
            .modify_guest_with(id, |g| {
                if g.is_energy_enough(g.walk_cost) {
                    g.energy -= g.walk_cost;
                    g.node.walk(to);
                }
            })
            .await
            .map(|x| x.node)
    }
}
