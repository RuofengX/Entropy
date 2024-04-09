use std::{
    borrow::BorrowMut,
    collections::BTreeMap,
    num::NonZeroUsize,
    ops::DerefMut,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, RwLock, RwLockReadGuard, RwLockWriteGuard,
    },
};

use ahash::AHashMap;
use dashmap::{DashMap, DashSet};
use moka::{
    policy::EvictionPolicy,
    sync::{Cache, CacheBuilder},
};
use serde::{Deserialize, Serialize};

use crate::{
    guest::{Guest, GID},
    node::{Node, NodeData, NodeID},
};

pub trait SaveStorage {
    fn contains_node(&self, node_id: NodeID) -> bool;
    fn save_node(&self, node: Option<Node>) -> Option<()>;
    fn load_node(&self, node_id: NodeID) -> Option<Node>;
    fn save_guest(&self, guest: Option<Guest>) -> Option<()>;
    fn load_guests(&self) -> Vec<Guest>;
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
    nodes_active: Cache<NodeID, Arc<RwLock<Node>>>,
    storage_backend: Box<dyn SaveStorage>,
}

impl World {
    pub fn new(storage_backend: Box<dyn SaveStorage>) -> World {
        World {
            players: AHashMap::new(),
            nodes_active: CacheBuilder::new(1_000_000)
                .eviction_policy(EvictionPolicy::lru())
                .build(),
            storage_backend,
        }
    }
}

impl World {
    /// Admin usage
    pub fn spawn(&mut self) -> GID {
        let g = Guest::spawn(NodeID(0, 0));
        let g_id = g.id;
        self.players.insert(g_id, RwLock::new(g));
        g_id
    }

    /// Soul usage
    pub(crate) fn get_guest(&self, id: GID) -> Option<&RwLock<Guest>> {
        self.players.get(&id).and_then(|g| Some(g))
    }

    /// Soul usage
    pub(crate) fn detect_node(&self, id: NodeID) -> Option<NodeData> {
        self.detect_node_from_cache(id).or_else(|| {
            self.load_node(id);
            if let Some(node) = self.nodes_active.get(&id) {
                Some(node.read().unwrap().data.clone())
            } else {
                None
            }
        })
    }

    /// Soul usage
    pub(crate) fn modify_node_with(&self, id: NodeID, f: impl FnOnce(&mut Node) -> ()) -> bool {
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

    fn check_node_status(&self, id: NodeID) -> NodeStatus {
        if self.nodes_active.contains_key(&id) {
            return NodeStatus::Active;
        } else if self.storage_backend.contains_node(id) {
            return NodeStatus::Archived;
        } else {
            return NodeStatus::NotExist;
        }
    }

    fn load_node(&self, id: NodeID) -> bool {
        if let Some(node) = self.storage_backend.load_node(id) {
            self.nodes_active
                .insert(id, Arc::new(RwLock::new(node)));
            true
        } else {
            false
        }
    }

    fn load_node_then_modify(&self, id: NodeID, f: impl FnOnce(&mut Node) -> ()) -> bool {
        if let Some(node) = self.nodes_active.get(&id) {
            f(&mut node.write().unwrap());
            return true;
        };
        if let Some(mut node) = self.storage_backend.load_node(id) {
            f(&mut node);
            self.nodes_active
                .insert(id, Arc::new(RwLock::new(node)));
            return true;
        };
        return false;
    }

    fn detect_node_from_cache(&self, id: NodeID) -> Option<NodeData> {
        if let Some(node) = self.nodes_active.get(&id) {
            Some(node.read().unwrap().data.clone())
        } else {
            None
        }
    }

}

#[derive(Serialize, Deserialize)]
struct NodeHeat {
    #[serde(skip)]
    nodes_runtime_use: AHashMap<NodeID, AtomicU64>,
    nodes_heat: AHashMap<NodeID, AtomicU64>,
}

#[derive(Debug, PartialEq, Eq)]
enum NodeStatus {
    Active,
    Archived,
    NotExist,
}

mod test{

}