use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::hash::Hash;

pub const NODE_SIZE: usize = 1024;

///         ^U
///      LU MU RU
/// L <- LM MM RM -> R
///      LD MD RM
///         vD
pub mod direction {
    pub type Direction = (i16, i16);
    pub const LU: (i16, i16) = (-1, 1);
    pub const MU: (i16, i16) = (0, 1);
    pub const RU: (i16, i16) = (1, 1);
    pub const LM: (i16, i16) = (-1, 0);
    pub const MM: (i16, i16) = (0, 0);
    pub const RM: (i16, i16) = (1, 0);
    pub const LD: (i16, i16) = (-1, -1);
    pub const MD: (i16, i16) = (0, -1);
    pub const RD: (i16, i16) = (1, -1);
}

/// y
/// ^ 0,1,2
/// | 3,4,5,
/// | 6,7,8
/// |------> x
pub const ALL_DIRECTION: [(i16, i16); 9] = [
    (-1, 1),
    (0, 1),
    (1, 1),
    (-1, 0),
    (0, 0),
    (1, 0),
    (-1, -1),
    (0, -1),
    (1, -1),
];

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
pub struct NodeData(#[serde(with = "serde_bytes")] pub [u8; NODE_SIZE]);
impl NodeData {
    pub fn random() -> Self {
        let mut rtn = [0u8; 1024];
        thread_rng().fill(&mut rtn);
        Self(rtn)
    }
}

#[derive(
    Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct NodeID(pub i16, pub i16);
impl NodeID {
    pub fn get_nearby_index(&self) -> NearbyIndex {
        let mut rtn = [self.clone(); 9];

        for (node, direction) in rtn.iter_mut().zip(ALL_DIRECTION.iter()) {
            (*node).0 += direction.0;
            (*node).1 += direction.1;
        }
        NearbyIndex(rtn)
    }
}

impl NodeID {
    pub(crate) fn walk(&mut self, direction: direction::Direction) -> NodeID {
        self.0 = self.0.wrapping_add(direction.0);
        self.1 = self.1.wrapping_add(direction.1);
        self.clone()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Node {
    pub data: NodeData,
}
impl Node {
    pub fn new(data: NodeData) -> Self {
        Node { data }
    }

    pub fn generate_new() -> Self {
        Node {
            data: NodeData::random(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct NearbyIndex(pub [NodeID; 9]);
impl NearbyIndex {
    pub fn get_id(&self) -> NodeID {
        self.0[4]
    }
}

#[derive(Debug, Hash, Clone, Copy, Serialize, Deserialize)]
pub struct FlatIndex(pub i32);
impl From<NodeID> for FlatIndex {
    fn from(value: NodeID) -> Self {
        FlatIndex((value.0 as i32) << 16 | (value.1) as i32)
    }
}
impl Into<NodeID> for FlatIndex {
    fn into(self) -> NodeID {
        let high = (self.0 >> 16) as i16;
        let low = self.0 as i16;
        NodeID(high, low)
    }
}

mod test {

    #[test]
    fn test_node_serde() {
        use super::{Node, NodeData};
        let node = Node::new(NodeData::random());
        let encoded_string = serde_json::to_string(&node).unwrap();
        let encoded_pickle =
            serde_pickle::to_vec(&node, serde_pickle::SerOptions::default()).unwrap();
        let encoded_bin =
            bincode::serde::encode_to_vec(node.clone(), bincode::config::standard()).unwrap();

        // println!("{}", encoded_string);
        println!("{:?}", encoded_pickle);
        println!("json格式长度 -> {}", encoded_string.len());
        println!("pickle格式长度 -> {}", encoded_pickle.len());
        println!("bincode格式长度 -> {}", encoded_bin.len());

        ////////////////////

        let node2 = serde_json::from_str::<Node>(&encoded_string).unwrap();
        assert_eq!(node, node2);

        let node3 =
            serde_pickle::from_slice::<Node>(&encoded_pickle, serde_pickle::DeOptions::default())
                .unwrap();
        assert_eq!(node, node3);

        let (node4, _): (Node, usize) =
            bincode::serde::decode_from_slice(&encoded_bin, bincode::config::standard()).unwrap();
        assert_eq!(node, node4);
    }
}
