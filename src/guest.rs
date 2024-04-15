use ordered_float::NotNan;
use serde::{Deserialize, Serialize};

use crate::{
    err::{GuestError, Result},
    node::{direction::Direction, NodeID},
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Serialize, Deserialize)]
pub struct GID(pub u64);
impl GID {
    pub fn get_then_increase(&mut self) -> GID {
        let rtn = self.clone();
        self.0 += 1;
        rtn
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Guest {
    pub id: GID,
    pub node: NodeID,
    pub energy: NotNan<f32>,
    pub walk_cost: NotNan<f32>,
    temperature: u8,
    engine_efficiency: NotNan<f32>,
}

impl Guest {
    /// used by World::Spawn
    pub(crate) fn new(id: GID, node: NodeID) -> Guest {
        Guest {
            id,
            node,
            temperature: 128,
            energy: NotNan::new(0.0).unwrap(),
            engine_efficiency: NotNan::new(0.8).unwrap(),
            walk_cost: NotNan::new(0.8).unwrap(),
        }
    }

    pub(crate) fn node_move(&mut self, to: Direction) -> Result<NodeID> {
        let now = self.energy;
        let cost = self.walk_cost;
        if now < cost {
            return Err(GuestError::EnergyNotEnough("Walk", cost, now).into());
        } else {
            Ok(self.node.walk(to)?)
        }
    }
}
