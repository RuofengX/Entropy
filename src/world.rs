use ahash::AHashMap;

use crate::{
    node::{Node, NodeID},
    player::{Guest, EID},
};

pub trait SaveStorage: Default {
    fn contains_node(&self, node_id: NodeID) -> bool;
    fn save_node(&self, node: Option<Node>) -> Option<()>;
    fn load_node(&self, node_id: NodeID) -> Option<Node>;
    fn save_guest(&self, guest: Option<Guest>) -> Option<()>;
    fn load_guest(&self, id: EID) -> Option<Guest>;
    fn flush(&mut self) -> ();
}

#[derive(Debug)]
pub struct World<S: SaveStorage> {
    players: AHashMap<EID, Guest>,
    node_list: Vec<NodeID>,
    nodes_active: Vec<Node>,
    storage_backend: S,
}
