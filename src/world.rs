use dashmap::mapref::one::Ref;
use dashmap::DashMap;
use dbgprint::dbgprintln;
use std::sync::Arc;
use std::sync::RwLock;

use moka::{
    policy::EvictionPolicy,
    sync::{CacheBuilder, SegmentedCache},
};
use serde::{Deserialize, Serialize};

use crate::soul::Soul;
use crate::{
    guest::{Guest, GID},
    node::{Node, NodeData, NodeID},
};

pub trait SaveStorage: std::fmt::Debug + Sync + Send {
    fn contains_node(&self, id: NodeID) -> bool;
    fn save_node(&self, id: NodeID, node: Option<&Node>) -> bool;
    fn load_node(&self, id: NodeID) -> Option<Node>;
    fn count_nodes(&self) -> i32;

    fn save_guest(&self, id: GID, guest: Option<&Guest>) -> bool;
    fn load_guests(&self) -> Vec<Guest>;
    fn count_guests(&self) -> u64;

    fn save_soul(&self, uid: String, soul: Option<Soul>) -> bool;
    fn load_soul(&self, uid: String) -> Option<Soul>;
    fn load_souls(&self) -> Vec<Soul>;

    fn flush(&self) -> ();
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

#[derive(Debug)]
pub struct World {
    guests: DashMap<GID, RwLock<Guest>, ahash::RandomState>,
    nodes_active: SegmentedCache<NodeID, Arc<RwLock<Node>>>,
    storage_backend: Arc<dyn SaveStorage>,
}

impl World {
    pub fn new(storage_backend: Arc<dyn SaveStorage>) -> World {
        World {
            guests: storage_backend
                .load_guests()
                .into_iter()
                .map(|g| (g.id, RwLock::new(g)))
                .collect(),
            nodes_active: CacheBuilder::new(1_000_000)
                .eviction_policy(EvictionPolicy::lru())
                .segments(16)
                .build(),
            storage_backend,
        }
    }
}
impl Drop for World {
    fn drop(&mut self) {
        dbgprintln!("回收world开始");
        self.guests.iter().for_each(|x| {
            self.storage_backend
                .save_guest(*x.key(), Some(&x.value().read().unwrap()));
        });
        self.nodes_active.iter().for_each(|(id, node_ctx)| {
            self.storage_backend
                .save_node(*id.clone(), Some(&node_ctx.read().unwrap()));
        });
        self.storage_backend.flush();
        dbgprintln!("回收world完毕");
    }
}

impl World {
    /// Admin usage
    pub fn count_guest(&self) -> u64 {
        self.storage_backend.count_guests()
    }
    /// Admin usage
    pub fn spawn(&self) -> GID {
        let g_id = GID(self.count_guest());
        let g = Guest::spawn(g_id, NodeID(0, 0));
        self.storage_backend.save_guest(g_id, Some(&g));
        self.guests.insert(g_id, RwLock::new(g));
        g_id
    }

    /// Soul usage
    pub(crate) fn get_guest(&self, id: GID) -> Option<Ref<GID, RwLock<Guest>, ahash::RandomState>> {
        self.guests.get(&id).and_then(|g| Some(g))
    }

    /// Soul usage
    pub(crate) fn detect_node(&self, id: NodeID) -> NodeData {
        if let Some(node) = self.nodes_active.get(&id) {
            node.read().unwrap().data.clone()
        } else {
            self.load_node(id);
            if let Some(node) = self.nodes_active.get(&id) {
                node.read().unwrap().data.clone()
            } else {
                self.generate_node(id)
            }
        }
    }

    /// Soul usage
    pub(crate) fn modify_node_with(&self, id: NodeID, f: impl FnOnce(&mut NodeData) -> ()) -> () {
        if self.nodes_active.contains_key(&id) {
            if let Some(node_lock) = self.nodes_active.get(&id) {
                let mut node_ctx = node_lock.write().unwrap();
                f(&mut node_ctx.data);
            };
        } else if self.storage_backend.contains_node(id) {
            let result = self.load_node_then_modify(id, f);
            debug_assert_eq!(result, true);
        } else {
            self.generate_node_then_modify(id, f);
        };
    }

    fn generate_node(&self, id: NodeID) -> NodeData {
        dbgprintln!("生成node::{:?}", id);
        let node = Node::generate_new();
        let rtn = node.data.clone();
        // self.storage_backend.save_node(id, Some(&node));
        self.nodes_active.insert(id, Arc::new(RwLock::new(node)));
        rtn
    }

    fn generate_node_then_modify(&self, id: NodeID, f: impl FnOnce(&mut NodeData)) {
        let mut node = Node::generate_new();
        f(&mut node.data);
        self.storage_backend.save_node(id, Some(&node));
        self.nodes_active.insert(id, Arc::new(RwLock::new(node)));
    }

    fn load_node(&self, id: NodeID) -> bool {
        if let Some(node) = self.storage_backend.load_node(id) {
            self.nodes_active.insert(id, Arc::new(RwLock::new(node)));
            true
        } else {
            false
        }
    }

    fn load_node_then_modify(&self, id: NodeID, f: impl FnOnce(&mut NodeData) -> ()) -> bool {
        if let Some(node) = self.nodes_active.get(&id) {
            f(&mut node.write().unwrap().data);
            return true;
        };
        if let Some(mut node) = self.storage_backend.load_node(id) {
            f(&mut node.data);
            self.nodes_active.insert(id, Arc::new(RwLock::new(node)));
            return true;
        };
        return false;
    }
}

#[derive(Debug)]
pub struct SledBackend {
    db: sled::Db,
    guests: typed_sled::Tree<GID, Guest>,
    nodes: typed_sled::Tree<NodeID, Node>,
    souls: typed_sled::Tree<String, Soul>,
}
impl SledBackend {
    pub fn new(temporary: bool) -> Self {
        let db = if !temporary {
            // 持久化配置
            sled::Config::new()
                .path("./saves")
                .mode(sled::Mode::HighThroughput)
                .open()
                .expect("创建sled数据库时出错")
        } else {
            // 临时配置
            sled::Config::new()
                .temporary(temporary)
                .open()
                .expect("创建sled数据库时出错")
        };
        SledBackend {
            guests: typed_sled::Tree::open(&db, "guests"),
            nodes: typed_sled::Tree::open(&db, "nodes"),
            souls: typed_sled::Tree::open(&db, "souls"),
            db,
        }
    }
}
impl SaveStorage for SledBackend {
    fn contains_node(&self, node_id: NodeID) -> bool {
        dbgprintln!("查询node::{:?}是否存在", node_id);
        self.nodes.contains_key(&node_id).is_ok()
    }

    fn save_node(&self, id: NodeID, node: Option<&Node>) -> bool {
        if let Some(node) = node {
            dbgprintln!("保存node::{:?}", id);
            self.nodes.insert(&id, node).is_ok()
        } else {
            dbgprintln!("删除node::{:?}", id);
            self.nodes.remove(&id).is_ok()
        }
    }

    fn load_node(&self, id: NodeID) -> Option<Node> {
        dbgprintln!("加载node::{:?}", id);
        self.nodes.get(&id).expect("读取sled数据库错误")
    }

    fn count_nodes(&self) -> i32 {
        self.nodes.len() as i32
    }

    fn save_guest(&self, id: GID, guest: Option<&Guest>) -> bool {
        if let Some(guest) = guest {
            dbgprintln!("保存guest::{:?}", id);
            self.guests.insert(&id, guest).expect("写入sled数据库错误");
            true
        } else {
            dbgprintln!("删除guest::{:?}", id);
            self.guests
                .remove(&id)
                .expect("写入sled数据库错误")
                .is_some()
        }
    }

    fn load_guests(&self) -> Vec<Guest> {
        dbgprintln!("加载全体guests");
        self.guests
            .iter()
            .into_iter()
            .filter_map(|x| match x {
                Ok((_, guest)) => Some(guest),
                Err(e) => {
                    dbgprintln!("读取guest时错误::{:?}", e);
                    None
                }
            })
            .collect()
    }

    fn count_guests(&self) -> u64 {
        self.guests.len() as u64
    }

    fn save_soul(&self, uid: String, soul: Option<Soul>) -> bool {
        if let Some(soul) = soul {
            dbgprintln!("保存soul::{:?}", uid);
            self.souls
                .insert(&uid, &soul)
                .expect("写入sled数据库错误")
                .is_some()
        } else {
            dbgprintln!("删除soul::{:?}", uid);
            self.souls
                .remove(&uid)
                .expect("写入sled数据库错误")
                .is_some()
        }
    }

    fn load_soul(&self, uid: String) -> Option<Soul> {
        dbgprintln!("加载soul::{:?}", uid);
        self.souls.get(&uid).expect("读取sled数据库错误")
    }

    fn load_souls(&self) -> Vec<Soul> {
        dbgprintln!("加载全体souls");
        self.souls
            .iter()
            .values()
            .map(|x| x.expect("读取sled数据库错误"))
            .collect()
    }

    fn flush(&self) -> () {
        match self.db.flush() {
            Ok(c) => {
                dbgprintln!("保存了{:?}字节", c);
            }
            Err(e) => {
                panic!("保存sled数据库时错误::{}", e)
            }
        }
    }
}

mod test {
    #![allow(unused_imports)]
    use super::{SledBackend, World};
    use crate::guest::GID;
    use crate::node::NodeData;
    use crate::node::NodeID;
    use std::sync::Arc;

    #[test]
    fn test_sled() {
        let back = Arc::new(SledBackend::new(true));
        let w = World::new(back.clone());
        assert_eq!(w.spawn(), GID(0));
        assert_eq!(w.spawn(), GID(1));
        assert_eq!(w.spawn(), GID(2));
        let g1 = w.get_guest(GID(1)).unwrap().read().unwrap().clone();
        drop(w);

        let w = World::new(back.clone());
        assert_eq!(w.spawn(), GID(3));
        assert_eq!(w.spawn(), GID(4));
        assert_eq!(w.spawn(), GID(5));

        assert_eq!(w.get_guest(GID(1)).unwrap().read().unwrap().clone(), g1);
    }

    #[test]
    fn test_node() {
        // Create world
        let back = Arc::new(SledBackend::new(true));
        let w = World::new(back.clone());

        //  assert nodes_active length
        assert_eq!(w.nodes_active.iter().collect::<Vec<_>>().len(), 0);
        w.detect_node(NodeID(1, 1));
        assert_eq!(w.nodes_active.iter().collect::<Vec<_>>().len(), 1);
        // assert nodes' data is same after dettach database
        let data1 = w.detect_node(NodeID(114, 514));
        drop(w);
        let w = World::new(back.clone());
        let data2 = w.detect_node(NodeID(114, 514));
        assert_eq!(data1, data2);

        // assert data change is also saved
        fn temperature_minus_one(data: &mut NodeData) {
            data.0.iter_mut().for_each(|x| *x = x.saturating_sub(1));
        }

        let tep1 = w.detect_node(NodeID(114, 514)).0[0];
        w.modify_node_with(NodeID(114, 514), temperature_minus_one);
        drop(w);
        let w = World::new(back.clone());
        let tep2 = w.detect_node(NodeID(114, 514)).0[0];

        assert_eq!(tep1.saturating_sub(1), tep2);
    }

    #[test]
    fn save_lot_nodes() {
        let back = Arc::new(SledBackend::new(false));
        let w = World::new(back.clone());
        for i in 0..1001 {
            w.detect_node(NodeID(i, i));
        }
        drop(w);
        let w = World::new(back.clone());
        assert_eq!(w.storage_backend.count_nodes(), 1001);
    }
}
