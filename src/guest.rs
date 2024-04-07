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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Guest {
    pub id: GID,
    node: NodeID,
    temperature: u8,
    energy: NotNan<f32>,
    walk_cost: NotNan<f32>,
    engine_efficiency: NotNan<f32>,
}

impl Guest {
    pub fn spawn(node: NodeID) -> Guest {
        static ID_COUNTER: RwLock<GID> = RwLock::new(GID(0));
        let mut wtx = ID_COUNTER.write().unwrap();
        Guest {
            id: wtx.get_then_increase(),
            node,
            temperature: 128,
            energy: NotNan::new(0.0).unwrap(),
            engine_efficiency: NotNan::new(0.8).unwrap(),
            walk_cost: NotNan::new(0.8).unwrap(),
        }
    }
    pub fn walk(&mut self, direction: direction::Direction) {
        self.node.0 += direction.0;
        self.node.1 += direction.1;
        self.energy -= self.walk_cost;
    }
}
