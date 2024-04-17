use ahash::HashSet;
use anyhow::{bail, Ok};
use futures::future::join_all;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};

use crate::{
    err::{GuestError, Result, SoulError},
    guest::{Guest, GID},
    node::direction::Direction,
    world::World,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Soul {
    pub uid: String,
    pub name: String,
    pub pw_hash: Vec<u8>,
    guest_quota: u64,
    guests: HashSet<GID>,
}
impl Soul {
    pub async fn new(world: &World, name: String, pw_hash: Vec<u8>) -> Self {
        let g = world.spawn().await;
        let guests = HashSet::from_iter(vec![g]);
        Self {
            uid: nanoid!(),
            name,
            pw_hash,
            guest_quota: 1,
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
        self.world.get_guest(id).await
    }

    pub async fn list_guests(&self) -> Vec<Guest> {
        let rtn = join_all(self.soul.guests.iter().map(|id| self.world.get_guest(*id)));
        rtn.await
            .into_iter()
            .filter_map(|x| x.ok())
            .filter_map(|x| x)
            .collect()
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
        if !self.world.contains_guest(id).await {};

        // Pre check energy is good, return error
        if let Some(g) = self.world.get_guest(id).await? {
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

            // check again whether the guest is exist
        } else {
            return Ok(None);
        }
    }
}
