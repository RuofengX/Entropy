use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::{
    hash::Hash,
    io::{Read, Write},
};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
pub struct NodeData(#[serde(with = "serde_bytes")] pub [u8; 1024]);
impl NodeData {
    pub fn random() -> Self {
        let mut rtn = [0u8; 1024];
        thread_rng().fill(&mut rtn);
        Self(rtn)
    }
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct NodeID(pub i16, pub i16);
impl NodeID {
    pub fn get_nearby_index(&self) -> NearbyIndex {
        let mut rtn = [self.clone(); 9];
        let directions: [(i16, i16); 9] = [
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

        for (node, direction) in rtn.iter_mut().zip(directions.iter()) {
            (*node).0 += direction.0;
            (*node).1 += direction.1;
        }
        NearbyIndex(rtn)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Node {
    index: NearbyIndex,
    data: NodeData,
}
impl Node {
    pub fn new(id: NodeID, data: NodeData) -> Self {
        Node {
            index: id.get_nearby_index(),
            data,
        }
    }

    pub fn save_to_writer(self, writer: impl Write) {
        serde_json::to_writer(writer, &self)
            .expect(format!("error when write node {:?} into writer", self.index.get_id()).as_str());
    }

    pub fn construct_from_reader(reader: impl Read) -> Self {
        serde_json::from_reader(reader).expect("error when read from reader")
    }
}

/// y
/// ^ 0,1,2
/// | 3,4,5,
/// | 6,7,8
/// |------> x
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
        use super::{Node, NodeData, NodeID};
        let node_i = NodeID(1, 2);
        let node = Node::new(node_i, NodeData::random());
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
