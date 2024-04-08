use std::sync::{RwLock, RwLockReadGuard};

use ahash::AHashMap;
use serde::{Deserialize, Serialize};

use crate::{
    guest::{Guest, GID},
    node::{Node, NodeID},
};

pub trait SaveStorage {
    fn contains_node(&self, node_id: NodeID) -> bool;
    fn save_node(&self, node: Option<Node>) -> Option<()>;
    fn load_node(&self, node_id: NodeID) -> Option<Node>;
    fn save_guest(&self, guest: Option<Guest>) -> Option<()>;
    fn load_guest(&self, guest_id: GID) -> Option<Guest>;
    fn flush(&mut self) -> ();
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Serialize, Deserialize)]
pub struct WorldID(pub u64);
impl WorldID {
    pub fn get_then_increase(&mut self) -> WorldID {
        let rtn = self.clone();
        self.0 += 1;
        rtn
    }
}

pub struct World {
    players: AHashMap<GID, RwLock<Guest>, ahash::RandomState>,
    nodes_active: RwLock<AHashMap<NodeID, RwLock<Node>, ahash::RandomState>>,
    storage_backend: Box<dyn SaveStorage>,
}

impl World {
    pub fn new(storage_backend: Box<dyn SaveStorage>) -> World {
        World {
            players: AHashMap::new(),
            nodes_active: RwLock::new(AHashMap::new()),
            storage_backend,
        }
    }
}

impl World {
    pub fn spawn(&mut self) -> GID {
        let g = Guest::spawn(NodeID(0, 0));
        let g_id = g.id;
        self.players.insert(g_id, RwLock::new(g));
        g_id
    }
    pub fn get_guest(&self, id: GID) -> Option<&RwLock<Guest>> {
        self.players.get(&id).and_then(|g| Some(g))
    }
    pub fn get_node<'a>(&'a self, id: NodeID) -> Option<RwLockReadGuard<Node>> {
        let n_act = self.nodes_active.read().unwrap();
        if n_act.contains_key(&id) {
            return n_act.get(&id).and_then(|g| Some(g.read().unwrap()));
        } else if self.storage_backend.contains_node(id) {
            return self.load_node_then_get(id);
        } else {
            None
        }
    }
    fn load_node_then_get(&self, id: NodeID) -> Option<RwLockReadGuard<Node>> {
        if let Some(node) = self.storage_backend.load_node(id) {
            let nid = node.get_id();
            let mut n_act = self.nodes_active.write().unwrap();
            n_act.insert(nid, RwLock::new(node));
            Some(n_act.get(&nid).unwrap().read().unwrap())
        } else {
            None
        }
    }
}
