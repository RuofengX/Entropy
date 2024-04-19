use serde::{Deserialize, Serialize};

use crate::db::Storage;
use crate::soul::{Soul, WonderingSoul};
use crate::{
    err::Result,
    guest::{Guest, GID},
    node::{NodeData, NodeID},
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Serialize, Deserialize)]
pub struct WorldID(pub u64);
impl WorldID {
    pub fn get_then_increase(&mut self) -> WorldID {
        let rtn = self.clone();
        self.0 += 1;
        rtn
    }
}

#[derive(Debug, Clone)]
pub struct World {
    pub storage: Storage,
}

impl World {
    pub fn new(storage: Storage) -> World {
        World { storage }
    }
}
impl Drop for World {
    fn drop(&mut self) {
        self.storage.flush().unwrap();
    }
}

impl World {
    /// Admin usage
    pub async fn count_guest(&self) -> u64 {
        self.storage.count_guests().await.unwrap()
    }
    /// Admin usage
    pub async fn spawn(&self) -> GID {
        let g_id = GID(self.count_guest().await);
        let g = Guest::new(g_id, NodeID(0, 0));
        self.storage.save_guest(g_id, Some(&g)).await.unwrap();
        g_id
    }

    /// Soul usage
    pub async fn register_soul(&self, name: String, pw_hash: String) -> Result<Soul> {
        let s = Soul::new(self, name, pw_hash).await;
        // self.storage.save_soul(&s.uid, Some(s.clone())).await?;
        Ok(s)
    }

    /// Soul usage
    pub async fn get_wondering_soul(&self, uid: &String) -> Result<Option<WonderingSoul>> {
        Ok(self
            .storage
            .get_soul(uid)
            .await?
            .and_then(|soul| Some(WonderingSoul::new(&self, soul))))
    }

    pub async fn get_soul(&self, uid: &String) -> Result<Option<Soul>> {
        Ok(self.storage.get_soul(uid).await?)
    }

    pub async fn verify_soul(&self, uid: &String, pw_hash: &String) -> Result<bool> {
        Ok(self
            .storage
            .get_soul(uid)
            .await?
            .is_some_and(|true_soul| true_soul.pw_hash == *pw_hash))
    }

    /// Soul usage
    pub async fn get_guest(&self, id: GID) -> Result<Option<Guest>> {
        self.storage.get_guest(id).await
    }

    /// Soul usage
    pub async fn detect_node(&self, id: NodeID) -> NodeData {
        self.storage.get_node_or_init(id).await.unwrap().data
    }

    /// Soul usage
    pub async fn modify_node_with(
        &self,
        id: NodeID,
        f: impl Fn(&mut NodeData) + Send + Sync,
    ) -> Result<Option<NodeData>> {
        Ok(self
            .storage
            .modify_node_with(id, |x| f(&mut x.data))
            .await?
            .and_then(|n| Some(n.data)))
    }

    /// Soul usage
    pub async fn contains_guest(&self, id: GID) -> bool {
        self.storage.contains_guest(id).await.unwrap()
    }

    /// Soul usage
    pub async fn modify_guest_with(
        &self,
        id: GID,
        f: impl Fn(&mut Guest) + Send + Sync,
    ) -> Result<Option<Guest>> {
        self.storage.modify_guest_with(id, f).await
    }

    pub async fn flush_async(&self) {
        let _ = self.storage.flush_async().await;
    }

    pub fn flush(&self) {
        let _ = self.storage.flush();
    }
}

#[cfg(test)]
mod test {
    use super::World;
    use crate::db::Storage;
    use crate::guest::GID;
    use crate::node::NodeData;
    use crate::node::NodeID;

    #[tokio::test]
    async fn test_sled() {
        let sled = Storage::new("test_sled".into(), true).unwrap();
        let w = World {
            storage: sled.clone(),
        };
        assert_eq!(w.spawn().await, GID(0));
        assert_eq!(w.spawn().await, GID(1));
        assert_eq!(w.spawn().await, GID(2));
        let g1 = w.get_guest(GID(1)).await.unwrap().clone();
        drop(w);

        let w = World { storage: sled };
        assert_eq!(w.spawn().await, GID(3));
        assert_eq!(w.spawn().await, GID(4));
        assert_eq!(w.spawn().await, GID(5));

        assert_eq!(w.get_guest(GID(1)).await.unwrap().clone(), g1);
    }

    #[tokio::test]
    async fn test_node() {
        // Create world
        let sled = Storage::new("test_node".into(), true).unwrap();
        let w = World {
            storage: sled.clone(),
        };

        //  assert nodes_active length
        assert_eq!(w.storage.count_guests().await.unwrap(), 0);
        w.detect_node(NodeID(1, 1)).await;
        assert_eq!(w.storage.count_guests().await.unwrap(), 1);
        // assert nodes' data is same after detach database
        let data1 = w.detect_node(NodeID(114, 514)).await;
        drop(w);

        let w = World {
            storage: sled.clone(),
        };
        let data2 = w.detect_node(NodeID(114, 514)).await;
        assert_eq!(data1, data2);

        // assert data change is also saved
        fn temperature_minus_one(data: &mut NodeData) {
            data.0.iter_mut().for_each(|x| *x = x.saturating_sub(1));
        }

        let tep1 = w.detect_node(NodeID(114, 514)).await.0[0];
        let _ = w
            .modify_node_with(NodeID(114, 514), temperature_minus_one)
            .await;
        drop(w);

        let w = World {
            storage: sled.clone(),
        };
        let tep2 = w.detect_node(NodeID(114, 514)).await.0[0];

        assert_eq!(tep1.saturating_sub(1), tep2);
    }

    #[tokio::test]
    async fn save_lot_nodes() {
        let sled = Storage::new("test_lot_nodes".into(), true).unwrap();
        let w = World::new(sled.clone());
        for i in 0..1001 {
            w.detect_node(NodeID(i, i)).await;
        }
        drop(w);

        let w = World::new(sled.clone());
        assert_eq!(w.storage.count_nodes().await.unwrap(), 1001);
    }
}
