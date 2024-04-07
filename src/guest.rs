use std::sync::RwLock;

use serde::{Deserialize, Serialize};

use crate::{
    node::{direction, NodeID},
    world::WorldID,
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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Guest {
    pub id: GID,
    node: NodeID,
    temperature: u8,
    energy: u8,
}

impl Guest {
    pub fn spawn(node: NodeID) -> Guest {
        static ID_COUNTER: RwLock<GID> = RwLock::new(GID(0));
        let mut wtx = ID_COUNTER.write().unwrap();
        Guest {
            id: wtx.get_then_increase(),
            node,
            temperature: 128,
            energy: 0,
        }
    }
    pub fn walk(&mut self, direction: direction::Direction) {
        self.node.0 += direction.0;
        self.node.1 += direction.1;
    }
}
