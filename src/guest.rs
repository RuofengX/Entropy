use std::sync::RwLock;

use ordered_float::NotNan;
use serde::{Deserialize, Serialize};

use crate::node::{direction, NodeID};

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
    node: NodeID,
    temperature: u8,
    energy: NotNan<f32>,
    walk_cost: NotNan<f32>,
    engine_efficiency: NotNan<f32>,
}

impl Guest {
    pub fn spawn(id:GID, node: NodeID) -> Guest {
        Guest {
            id,
            node,
            temperature: 128,
            energy: NotNan::new(0.0).unwrap(),
            engine_efficiency: NotNan::new(0.8).unwrap(),
            walk_cost: NotNan::new(0.8).unwrap(),
        }
    }
}
