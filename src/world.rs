use std::{
    borrow::BorrowMut, collections::BTreeMap, sync::{atomic::AtomicU64, RwLock, RwLockReadGuard, RwLockWriteGuard}
};

use ahash::AHashMap;
use dashmap::{DashMap, DashSet};
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
    nodes_active: DashMap<NodeID, RwLock<Node>, ahash::RandomState>,
    storage_backend: Box<dyn SaveStorage>,
}

impl World {
    pub fn new(storage_backend: Box<dyn SaveStorage>) -> World {
        World {
            players: AHashMap::new(),
            nodes_active: DashMap::<NodeID, RwLock<Node>, ahash::RandomState>::default(),
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

    pub fn modify_node_with(&self, id: NodeID, f: impl FnOnce(&mut Node) -> ()) -> bool {
        if self.nodes_active.contains_key(&id) {
            if let Some(node_lock) = self.nodes_active.get(&id) {
                let mut node_ctx = node_lock.write().unwrap();
                f(&mut node_ctx);
                return true;
            };
        };

        if self.storage_backend.contains_node(id) {
            let _result = self.load_node_then_modify(id, f);
            debug_assert_eq!(_result, true);
            return true;
        };
        false
    }

    fn load_node_then_modify(&self, id: NodeID, f: impl FnOnce(&mut Node) -> ()) -> bool {
        if let Some(mut node) = self.storage_backend.load_node(id) {
            let nid = node.get_id();
            f(&mut node);
            self.nodes_active.insert(nid, RwLock::new(node));
            true
        } else {
            false
        }
    }
}
