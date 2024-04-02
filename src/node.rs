use serde::{de::Visitor, ser::SerializeSeq, Deserialize, Serialize};
use std::{
    hash::Hash,
    io::{Read, Write},
};

#[derive(Debug)]
pub struct NodeData([u8; 1024]);
impl Serialize for NodeData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Transparent storage, ignore newtype
        serializer.serialize_bytes(&self.0)
    }
}
impl<'de> Deserialize<'de> for NodeData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct NodeDataVisitor;
        impl<'de> Visitor<'de> for NodeDataVisitor {
            type Value = NodeData;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("")
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let mut rtn = [0u8; 1024];
                if v.take(1024).read(&mut rtn).is_ok() {
                    Ok(NodeData(rtn))
                } else {
                    Err(E::custom(format!(
                        "byte length of node data not correct(!=1024)"
                    )))
                }
            }
        }
        deserializer.deserialize_bytes(NodeDataVisitor)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct NodeID((i16, i16));

#[derive(Debug, Serialize, Deserialize)]
pub struct Node {
    id: NodeID,
    index: NodeIndex,
    data: NodeData,
}
impl Node {
    pub fn save_to_writer(self, writer: impl Write) {
        serde_json::to_writer(writer, &self)
            .expect(format!("error when write node {:?} into writer", self.id).as_str());
    }
    pub fn construct_from_reader(reader: impl Read) -> Self {
        serde_json::from_reader(reader).expect("error when read from reader")
    }
}

/// 0,1,2
/// 3,4,5,
/// 6,7,8
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct NodeIndex(pub [NodeID; 9]);
impl NodeIndex {
    pub fn get_self_id(&self) -> NodeID {
        self.0[4]
    }
}

#[derive(Debug, Hash, Clone, Copy, Serialize, Deserialize)]
pub struct FlatIndex(pub i32);
impl From<NodeID> for FlatIndex {
    fn from(value: NodeID) -> Self {
        FlatIndex((value.0 .0 as i32) << 16 | (value.0 .1) as i32)
    }
}
impl Into<NodeID> for FlatIndex {
    fn into(self) -> NodeID {
        let high = (self.0 >> 16) as i16;
        let low = self.0 as i16;
        NodeID((high, low))
    }
}
