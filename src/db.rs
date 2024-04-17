use std::path::PathBuf;

use anyhow::Ok;
use async_trait::async_trait;
use thiserror::Error;
use typed_sled::Tree;

use crate::{
    err::{GuestError, NodeError, Result, SoulError},
    guest::{Guest, GID},
    node::{Node, NodeID},
    soul::Soul,
};

#[async_trait]
pub trait SaveStorage: std::fmt::Debug + Sync + Send + Unpin {
    // NODES
    async fn contains_node(&self, id: NodeID) -> Result<bool>;
    async fn count_nodes(&self) -> Result<u32>;

    async fn get_node(&self, id: NodeID) -> Result<Option<Node>>;
    async fn get_node_or_init(&self, id: NodeID) -> Result<Node>; // Auto init node if not exist
    async fn save_node(&self, id: NodeID, node: Option<&Node>) -> Result<()>;
    async fn modify_node_with(
        &self,
        id: NodeID,
        f: impl for<'b> Fn(&'b mut Node) + Send + Sync,
    ) -> Result<Node>; // Auto init node if not exist

    // GUESTS
    async fn contains_guest(&self, id: GID) -> Result<bool>;
    async fn count_guests(&self) -> Result<u64>;

    async fn get_guest(&self, id: GID) -> Result<Guest>;
    async fn get_guests(&self) -> Result<Vec<Guest>>;
    async fn save_guest(&self, id: GID, guest: Option<&Guest>) -> Result<()>;
    async fn modify_guest_with(
        &self,
        id: GID,
        f: impl for<'g> Fn(&'g mut Guest) + Send + Sync,
    ) -> Result<Guest>;

    // SOULS
    async fn get_soul(&self, uid: &String) -> Result<Soul>;
    async fn get_souls(&self) -> Result<Vec<Soul>>;
    async fn save_soul(&self, uid: &String, soul: Option<Soul>) -> Result<()>;

    // META
    fn flush(&self) -> Result<()>;
    async fn flush_async(&self) -> Result<()>;
}

#[derive(Debug, Clone)]
pub(crate) struct SledStorage {
    // Select a specific namespace / database
    db: sled::Db,
    node: Tree<NodeID, Node>,
    guest: Tree<GID, Guest>,
    soul: Tree<String, Soul>,
}

impl SledStorage {
    pub(crate) fn new(path: PathBuf, temporary: bool) -> Result<Self> {
        // Create database connection
        let db = sled::Config::new().path(path).temporary(temporary).open()?;
        let node = typed_sled::Tree::open(&db, "node");
        let guest = typed_sled::Tree::open(&db, "node");
        let soul = typed_sled::Tree::open(&db, "node");

        Ok(Self {
            db,
            node,
            guest,
            soul,
        })
    }
}

#[async_trait]
impl SaveStorage for SledStorage {
    async fn contains_node(&self, id: NodeID) -> Result<bool> {
        self.node.contains_key(&id).map_err(|x| x.into())
    }
    async fn count_nodes(&self) -> Result<u32> {
        Ok(self.node.len() as u32)
    }

    async fn get_node(&self, id: NodeID) -> Result<Option<Node>> {
        self.node.get(&id).map_err(|x| x.into())
    }
    // Auto init node if not exist
    async fn get_node_or_init(&self, id: NodeID) -> Result<Node> {
        if let Some(node) = self.get_node(id).await? {
            Ok(node)
        } else {
            let node = Node::generate_new();
            self.save_node(id, Some(&node)).await?;
            Ok(node)
        }
    }
    async fn save_node(&self, id: NodeID, node: Option<&Node>) -> Result<()> {
        if let Some(node) = node {
            self.node.insert(&id, node)?;
            Ok(())
        } else {
            self.node.remove(&id)?;
            Ok(())
        }
    }
    async fn modify_node_with(
        &self,
        id: NodeID,
        f: impl for<'b> Fn(&'b mut Node) + Send,
    ) -> Result<Node> {
        let full_f = |x: Option<Node>| -> Option<Node> {
            let mut temp_node = x.unwrap_or_else(Node::generate_new);
            f(&mut temp_node);
            Some(temp_node)
        };
        let rtn = self.node.update_and_fetch(&id, full_f)?;
        if let Some(node) = rtn {
            Ok(node)
        } else {
            Err(NodeError::NotExist(id).into())
        }
    }

    // GUESTS
    async fn contains_guest(&self, id: GID) -> Result<bool> {
        self.guest.contains_key(&id).map_err(|x| x.into())
    }
    async fn count_guests(&self) -> Result<u64> {
        Ok(self.guest.len() as u64)
    }

    async fn get_guest(&self, id: GID) -> Result<Guest> {
        if let Some(guest) = self.guest.get(&id)? {
            Ok(guest)
        } else {
            Err(GuestError::NotExist(id).into())
        }
    }
    async fn get_guests(&self) -> Result<Vec<Guest>> {
        Ok(self.guest.iter().values().filter_map(|x| x.ok()).collect())
    }
    async fn save_guest(&self, id: GID, guest: Option<&Guest>) -> Result<()> {
        if let Some(guest) = guest {
            self.guest.insert(&id, guest)?;
            Ok(())
        } else {
            self.guest.remove(&id)?;
            Ok(())
        }
    }
    async fn modify_guest_with(
        &self,
        id: GID,
        f: impl for<'g> Fn(&'g mut Guest) + Send + Sync,
    ) -> Result<Guest> {
        if !self.contains_guest(id).await? {
            return Err(GuestError::NotExist(id).into());
        }

        self.guest
            .update_and_fetch(&id, |x| {
                x.and_then(|mut g| {
                    f(&mut g);
                    Some(g)
                })
            })
            .map_err(|e| e.into()) // SledError -> Error
            .and_then(|x| x.ok_or(GuestError::NotExist(id).into())) // GuestError::NotExist -> Error
    }

    // SOULS
    async fn get_soul(&self, uid: &String) -> Result<Soul> {
        self.soul.get(&uid)?.ok_or(SoulError::NotExist(uid.clone()).into())
    }
    async fn get_souls(&self) -> Result<Vec<Soul>> {
        Ok(self.soul.iter().values().filter_map(|x| x.ok()).collect())
    }
    async fn save_soul(&self, uid: &String, soul: Option<Soul>) -> Result<()> {
        if let Some(soul) = soul {
            self.soul
                .insert(&uid, &soul)
                .map_err(|x| x.into())
                .map(|_| ())
        } else {
            self.soul.remove(&uid).map_err(|x| x.into()).map(|_| ())
        }
    }

    // META
    fn flush(&self) -> Result<()> {
        self.db.flush().map(|_| ()).map_err(|x| x.into())
    }
    async fn flush_async(&self) -> Result<()> {
        self.db
            .flush_async()
            .await
            .map(|_| ())
            .map_err(|x| x.into())
    }
}
#[derive(Debug, Error)]
pub enum DatabaseError {}
