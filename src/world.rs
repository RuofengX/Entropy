use ahash::AHashMap;
use serde::{Deserialize, Serialize};

use crate::{
    guest::{Guest, GID},
    node::{Node, NodeID},
};

pub trait SaveStorage {
    fn contains_node(&self, world_id: WorldID, node_id: NodeID) -> bool;
    fn save_node(&self, world_id: WorldID, node: Option<Node>) -> Option<()>;
    fn load_node(&self, world_id: WorldID, node_id: NodeID) -> Option<Node>;
    fn save_guest(&self, world_id: WorldID, guest: Option<Guest>) -> Option<()>;
    fn load_guest(&self, world_id: WorldID, guest_id: GID) -> Option<Guest>;
    fn flush(&mut self) -> ();
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Serialize, Deserialize)]
pub struct WorldID(pub u64);

pub struct World {
    id: WorldID,
    players: AHashMap<GID, Guest, ahash::RandomState>,
    nodes_active: AHashMap<NodeID, Node, ahash::RandomState>,
    storage_backend: Box<dyn SaveStorage>,
}

impl World {
    pub fn spawn(&mut self) -> GID {
        let g = Guest::spawn(self.id, NodeID(0, 0));
        let g_id = g.id;
        self.players.insert(g_id, g);
        g_id
    }
}
