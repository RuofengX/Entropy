use ahash::HashSet;
use anyhow::{bail, Ok};
use dbgprint::dbgprintln;
use futures::future::join_all;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};

use crate::{
    alphabet::ENTROPY_CHAR,
    err::{GuestError, NodeError, Result, SoulError},
    guest::{Guest, GID},
    node::{direction::Direction, NODE_SIZE},
    world::World,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Soul {
    pub name: String,
    pub uid: String,
    pub pw_hash: String,
    guest_quota: u64,
    guests: HashSet<GID>,
}
impl Soul {
    pub async fn spawn(world: &World, name: String, pw_hash: String) -> Self {
        let g = world.spawn().await;
        let guests = HashSet::from_iter(vec![g]);
        Self {
            name,
            uid: nanoid!(22, &ENTROPY_CHAR),
            pw_hash,
            guest_quota: 1,
            guests,
        }
    }
    pub fn new(
        name: String,
        uid: String,
        pw_hash: String,
        guest_quota: u64,
        guests: HashSet<GID>,
    ) -> Self {
        Self {
            name,
            uid,
            pw_hash,
            guest_quota,
            guests,
        }
    }
}

pub struct WonderingSoul<'w> {
    soul: Soul,
    world: &'w World,
}
impl<'w> WonderingSoul<'w> {
    pub fn new(world: &'w World, soul: Soul) -> Self {
        Self { soul, world }
    }

    pub fn contains_guest(&self, id: GID) -> bool {
        self.soul.guests.contains(&id)
    }

    pub async fn get_guest(&self, id: GID) -> Result<Option<Guest>> {
        if self.soul.guests.contains(&id) {
            self.world.get_guest(id).await
        } else {
            Err(SoulError::GuestNotConnected(id).into())
        }
    }

    pub async fn list_guests(&self) -> Vec<Guest> {
        let rtn = join_all(self.soul.guests.iter().map(|id| self.world.get_guest(*id)));
        rtn.await
            .into_iter()
            .filter_map(|x| x.ok())
            .filter_map(|x| x)
            .collect()
    }

    pub async fn disconnect_guest(&mut self, id: GID) -> Result<Option<Guest>> {
        let guest = self.get_guest(id).await?;
        if self.soul.guests.take(&id).is_some() {
            Ok(guest)
        } else {
            Err(SoulError::GuestNotConnected(id).into())
        }
    }

    pub async fn walk(&self, id: GID, to: Direction) -> Result<Option<Guest>> {
        if !self.contains_guest(id) {
            return Err(SoulError::GuestNotConnected(id).into());
        }

        // check again whether the guest is exist
        if let Some(g) = self.world.get_guest(id).await? {
            // Pre check energy is good, return error
            if !g.is_energy_enough(g.walk_cost) {
                bail!(GuestError::EnergyNotEnough {
                    op_name: "walk",
                    require: g.walk_cost,
                    left: g.energy
                })
            };

            Ok(self
                .world
                .modify_guest_with(id, |g| {
                    // Also check energy, but ignore error
                    if g.is_energy_enough(g.walk_cost) {
                        g.energy -= g.walk_cost;
                        g.node.walk(to);
                    }
                })
                .await?)
        } else {
            return Err(SoulError::GuestNotExist(id).into());
        }
    }

    pub async fn harvest(&self, id: GID, at: usize) -> Result<Option<Guest>> {
        if at > NODE_SIZE - 1 {
            bail!(NodeError::IndexOutOfRange(at))
        }

        if !self.contains_guest(id) {
            return Err(SoulError::GuestNotConnected(id).into());
        }

        // check first because modify guest wont pop error if guest not exist
        if !self.world.contains_guest(id).await? {
            bail!(SoulError::GuestNotExist(id))
        };

        // check again whether the guest is exist
        Ok(self
            .world
            .modify_guest_with(id, |g| {
                let _ = self.world.modify_node_with_sync(g.node, |n| {
                    let cell = unsafe { n.0.get_unchecked_mut(at) };
                    g.generate_energy(cell);
                });
            })
            .await?)
    }
}
