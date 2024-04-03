use ahash::AHashMap;
use serde::{Deserialize, Serialize};

use crate::{
    node::{Node, NodeID},
    player::{Player, EID},
};

pub trait NodeStorage: Default {
    fn is_contains_node(&self, node_id: NodeID) -> bool;
    fn save_node(&self, node: Node) -> Option<()>;
    fn save_nodes(&self, nodes: impl Iterator<Item = Node>) -> Option<()>;
    fn load_node(&self, node_id: NodeID) -> Option<Node>;
    fn load_nodes(&self, node_ids: impl Iterator<Item = NodeID>) -> impl Iterator<Item = Node>;
    fn load_nearby_nodes(&self, node_id: NodeID, radius: i16) -> impl Iterator<Item = Node>;
    fn save_player(&self, player: Player) -> Option<()>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct World<B: NodeStorage> {
    players: AHashMap<EID, Player>,
    node_list: Vec<NodeID>,

    #[serde(skip)]
    nodes_active: Vec<Node>,
    #[serde(skip)]
    storage_backend: B,
}
