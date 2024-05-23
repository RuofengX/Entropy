use std::collections::HashSet;

use anyhow::{bail, Ok};
use futures::future::join_all;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};

use crate::{
    alphabet::ENTROPY_CHAR,
    err::{GuestError, NodeError, Result, SoulError},
    guest::{Guest, GID},
    node::{navi::Direction, NodeID, NODE_SIZE},
    world::World,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Soul {
    pub name: String,
    pub uid: String,
    pub pw_hash: String,
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
            guests,
        }
    }
    pub fn new(name: String, uid: String, pw_hash: String, guests: HashSet<GID>) -> Self {
        Self {
            name,
            uid,
            pw_hash,
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

    pub fn contain_guest(&self, id: GID) -> bool {
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

    pub async fn walk(&self, id: GID, to: Direction) -> Result<Option<Guest>> {
        self.check_guest(id).await?;

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
                        g.node.transform(to);
                    }
                })
                .await?)
        } else {
            return Err(SoulError::GuestNotExist(id).into());
        }
    }

    pub async fn harvest(&self, id: GID, at: usize) -> Result<Option<Guest>> {
        Self::check_node_index(at)?;
        self.check_guest(id).await?;

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

    pub async fn heat(&self, id: GID, at: usize, energy: u8) -> Result<Option<Guest>> {
        Self::check_node_index(at)?;
        self.check_guest_energy(id, "heat", energy as u64).await?;

        self.world
            .modify_guest_with(id, |g| {
                if !g.is_energy_enough(energy as u64) {
                    return;
                };
                let _ = self.world.modify_node_with_sync(g.node, |n| {
                    let cell = n.0[at];
                    let (new_cell, is_overflow) = cell.overflowing_add(energy);
                    g.energy -= energy as u64;
                    if !is_overflow {
                        n.0[at] = new_cell;
                    } else {
                        let overflow = new_cell + 1;
                        g.energy += overflow as u64;
                        n.0[at] = u8::MAX;
                    }
                });
            })
            .await
    }

    pub async fn spawn(&self, id: GID, energy: u64) -> Result<Option<Guest>> {
        self.check_guest_energy(id, "spawn", energy as u64).await?;

        let mut node: NodeID = Default::default();

        self.world
            .modify_guest_with(id, |g| {
                g.energy -= energy;
                node = g.node;
            })
            .await?;

        let new_gid = self.world.spawn_at(node).await;

        self.world
            .modify_guest_with(new_gid, |g| {
                g.energy += energy;
            })
            .await?;

        self.world.get_guest(new_gid).await
    }

    fn check_node_index(at: usize) -> Result<()> {
        if at > NODE_SIZE - 1 {
            bail!(NodeError::IndexOutOfRange(at))
        } else {
            Ok(())
        }
    }

    async fn check_guest(&self, id: GID) -> Result<()> {
        if !self.soul.guests.contains(&id) {
            bail!(SoulError::GuestNotConnected(id));
        };
        if !self.world.contains_guest(id).await? {
            bail!(SoulError::GuestNotExist(id));
        };
        Ok(())
    }

    async fn check_guest_energy(&self, id: GID, op_name: &'static str, require: u64) -> Result<()> {
        if !self.soul.guests.contains(&id) {
            bail!(SoulError::GuestNotConnected(id));
        };
        if let Some(g) = self.world.get_guest(id).await? {
            if require > g.energy {
                bail!(GuestError::EnergyNotEnough {
                    op_name,
                    require,
                    left: g.energy
                });
            } else {
                Ok(())
            }
        } else {
            bail!(SoulError::GuestNotExist(id));
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        db::Storage,
        node::{navi, NodeID},
    };

    use super::*;

    async fn init_world(name: &'static str) -> World {
        let sled = Storage::new(format!("{}.sled", name).into(), true).unwrap();
        World::new(sled)
    }

    async fn get_wondering_soul<'w>(w: &'w World) -> WonderingSoul<'w> {
        let soul = w
            .register_soul("test".to_string(), "".to_string())
            .await
            .unwrap();
        w.get_wondering_soul(&soul.uid).await.unwrap().unwrap()
    }

    #[tokio::test]
    async fn test_get_guest() {
        let w = init_world("get").await;
        let s = get_wondering_soul(&w).await;

        let gid = s.soul.guests.iter().next().unwrap();

        let g = s.get_guest(*gid).await.unwrap().unwrap();
        assert_eq!(g.id, *gid);
    }

    #[tokio::test]
    async fn test_list_guest() {
        let w = init_world("list").await;
        let s = get_wondering_soul(&w).await;

        let gids: Vec<GID> = s.soul.guests.iter().cloned().collect();

        let gids_2: Vec<GID> = s.list_guests().await.into_iter().map(|g| g.id).collect();
        assert_eq!(gids, gids_2);
    }

    #[tokio::test]
    async fn test_walk() {
        let w = init_world("walk").await;
        let s = get_wondering_soul(&w).await;
        let gid: GID = s.soul.guests.iter().cloned().next().unwrap();

        let g = w.get_guest(gid).await.unwrap().unwrap();
        let mut pos = g.node;

        let _ = s.walk(gid, navi::UP_RIGHT).await;

        let g = w.get_guest(gid).await.unwrap().unwrap();
        let pos_2 = g.node;

        assert_eq!(pos.transform(navi::UP_RIGHT), pos_2);
    }

    #[tokio::test]
    async fn test_walk_edge() {
        let w = init_world("walk_edge").await;
        let s = get_wondering_soul(&w).await;
        let gid: GID = s.soul.guests.iter().cloned().next().unwrap();

        // make the guest at right-middle polar node
        let _ = w
            .modify_guest_with(gid, |g| {
                g.node = NodeID::POLAR_RIGHT_MIDDLE;
            })
            .await;

        let _ = s.walk(gid, navi::RIGHT).await;

        let g = w.get_guest(gid).await.unwrap().unwrap();
        let pos_2 = g.node;

        // warps to the left
        assert_eq!(pos_2, NodeID::POLAR_LEFT_MIDDLE);
    }

    //TODO MORE TEST NEEDED
}
