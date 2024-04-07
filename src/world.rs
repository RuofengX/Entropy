use ahash::AHashMap;

use crate::{
    node::{Node, NodeID},
    player::{Guest, GID},
};

pub trait SaveStorage {
    fn contains_node(&self, world_id: WorldID, node_id: NodeID) -> bool;
    fn save_node(&self, world_id: WorldID, node: Option<Node>) -> Option<()>;
    fn load_node(&self, world_id: WorldID, node_id: NodeID) -> Option<Node>;
    fn save_guest(&self, world_id: WorldID, guest: Option<Guest>) -> Option<()>;
    fn load_guest(&self, world_id: WorldID, guest_id: GID) -> Option<Guest>;
    fn flush(&mut self) -> ();
}

pub struct WorldID(pub u64);

pub struct World {
    id: WorldID,
    players: AHashMap<GID, Guest>,
    node_list: Vec<NodeID>,
    nodes_active: Vec<Node>,
    storage_backend: Box<dyn SaveStorage>,
}
