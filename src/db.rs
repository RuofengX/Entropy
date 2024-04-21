use std::{path::PathBuf, sync::Arc};

use dbgprint::dbgeprintln;
use typed_sled::Tree;

use crate::{
    err::Result,
    guest::{Guest, GID},
    node::{Node, NodeID},
    soul::Soul,
};

#[derive(Debug, Clone)]
pub struct Storage {
    // Select a specific namespace / database
    db: sled::Db,
    node: Tree<NodeID, Node>,
    guest: Tree<GID, Guest>,
    soul: Tree<String, Soul>,
}

impl Storage {
    pub(crate) fn new(path: PathBuf, temporary: bool) -> Result<Arc<Self>> {
        // Create database connection
        let db = sled::Config::new()
            .path(path)
            .temporary(temporary)
            .print_profile_on_drop(true)
            .open()
            .unwrap();
        let node = typed_sled::Tree::open(&db, "node");
        let guest = typed_sled::Tree::open(&db, "guest");
        let soul = typed_sled::Tree::open(&db, "soul");

        Ok(Arc::new(Self {
            db,
            node,
            guest,
            soul,
        }))
    }
}

impl Storage {
    pub async fn contains_node(&self, id: NodeID) -> Result<bool> {
        self.node.contains_key(&id).map_err(|x| x.into())
    }
    pub async fn count_nodes(&self) -> Result<u32> {
        Ok(self.node.len() as u32)
    }

    pub async fn get_node(&self, id: NodeID) -> Result<Option<Node>> {
        self.node.get(&id).map_err(|x| x.into())
    }
    // Auto init node if not exist
    pub async fn get_node_or_init(&self, id: NodeID) -> Result<Node> {
        if let Some(node) = self.get_node(id).await? {
            Ok(node)
        } else {
            let node = Node::generate_new();
            self.save_node(id, Some(&node)).await?;
            Ok(node)
        }
    }
    pub async fn save_node(&self, id: NodeID, node: Option<&Node>) -> Result<()> {
        if let Some(node) = node {
            self.node.insert(&id, node)?;
            dbgeprintln!("[db] save_node::{:?}", node);
            Ok(())
        } else {
            self.node.remove(&id)?;
            Ok(())
        }
    }

    pub async fn modify_node_with(
        &self,
        id: NodeID,
        mut f: impl for<'b> FnMut(&'b mut Node) + Send,
    ) -> Result<Option<Node>> {
        let full_f = |x: Option<Node>| -> Option<Node> {
            let mut temp_node = x.unwrap_or_else(Node::generate_new);
            f(&mut temp_node);
            Some(temp_node)
        };
        let rtn = self.node.update_and_fetch(&id, full_f)?;
        Ok(rtn)
    }

    pub fn modify_node_with_sync(
        &self,
        id: NodeID,
        mut f: impl for<'b> FnMut(&'b mut Node) + Send,
    ) -> Result<Option<Node>> {
        let full_f = |x: Option<Node>| -> Option<Node> {
            let mut temp_node = x.unwrap_or_else(Node::generate_new);
            f(&mut temp_node);
            Some(temp_node)
        };
        let rtn = self.node.update_and_fetch(&id, full_f)?;
        Ok(rtn)
    }

    // GUESTS
    pub async fn contains_guest(&self, id: GID) -> Result<bool> {
        self.guest.contains_key(&id).map_err(|x| x.into())
    }
    pub async fn count_guests(&self) -> Result<u64> {
        Ok(self.guest.len() as u64)
    }

    pub async fn get_guest(&self, id: GID) -> Result<Option<Guest>> {
        Ok(self.guest.get(&id)?)
    }

    pub async fn get_guests(&self) -> Result<Vec<Guest>> {
        Ok(self.guest.iter().values().filter_map(|x| x.ok()).collect())
    }

    pub async fn save_guest(&self, id: GID, guest: Option<&Guest>) -> Result<()> {
        if let Some(guest) = guest {
            self.guest.insert(&id, guest)?;
            debug_assert_eq!(self.guest.len() as u64, id.0 + 1);
            dbgeprintln!("[db] save_guest::{:?}", guest);
            Ok(())
        } else {
            self.guest.remove(&id)?;
            Ok(())
        }
    }
    pub async fn modify_guest_with(
        &self,
        id: GID,
        mut f: impl for<'g> FnMut(&'g mut Guest) + Send + Sync,
    ) -> Result<Option<Guest>> {
        if !self.contains_guest(id).await? {
            return Ok(None);
        }

        Ok(self.guest.update_and_fetch(&id, |x| {
            x.and_then(|mut g| {
                f(&mut g);
                Some(g)
            })
        })?)
    }

    // SOULS
    pub async fn get_soul(&self, uid: &String) -> Result<Option<Soul>> {
        Ok(self.soul.get(&*uid)?)
    }
    pub async fn get_souls(&self) -> Result<Vec<Soul>> {
        Ok(self.soul.iter().values().filter_map(|x| x.ok()).collect())
    }

    /// Return the old value, if any.
    pub async fn save_soul(&self, uid: &String, soul: Option<Soul>) -> Result<Option<Soul>> {
        if let Some(soul) = soul {
            dbgeprintln!("[db] save_soul::{:?}", soul);
            let rtn = self.soul.insert(uid, &soul)?;
            Ok(rtn)
        } else {
            Ok(self.soul.remove(uid)?)
        }
    }

    // META
    pub fn flush(&self) -> Result<()> {
        self.db.flush().map(|_| ()).map_err(|x| x.into())
    }
    pub async fn flush_async(&self) -> Result<()> {
        self.db
            .flush_async()
            .await
            .map(|_| ())
            .map_err(|x| x.into())
    }
}
impl Drop for Storage {
    fn drop(&mut self) {
        println!("shutting down db");
        let _ = self.flush();
    }
}

// todo!("test db::save_soul and db::get_soul");

#[cfg(test)]
mod test {
    use crate::world::World;

    use super::*;
    #[tokio::test]
    async fn test_soul() {
        let db = Storage::new("./test.sled".into(), true).unwrap();
        let world = World::new(db.clone());

        let soul = world
            .register_soul("name".to_string(), "123456".to_string())
            .await
            .unwrap();

        assert_eq!(db.get_soul(&soul.uid).await.unwrap(), Some(soul));
    }
}
