use rand::{thread_rng, Rng, RngCore};
use sea_orm::DeriveValueType;
use serde::{Deserialize, Serialize};

pub const NODE_MAX_SIZE: usize = 1024;

///         ^UP
///         |
/// LEFT <-   -> RIGHT
///         |
///         vDOWN
pub mod navi {
    pub type Direction = (i16, i16);
    pub const SITU: (i16, i16) = (0, 0);
    pub const UP: (i16, i16) = (0, 1);
    pub const DOWN: (i16, i16) = (0, -1);
    pub const LEFT: (i16, i16) = (-1, 0);
    pub const RIGHT: (i16, i16) = (1, 0);

    pub const UP_LEFT: (i16, i16) = (-1, 1);
    pub const UP_RIGHT: (i16, i16) = (1, 1);
    pub const DOWN_LEFT: (i16, i16) = (-1, -1);
    pub const DOWN_RIGHT: (i16, i16) = (1, -1);
}

/// y
/// ^ 0,1,2
/// | 3,4,5,
/// | 6,7,8
/// |------> x
pub const INDEXED_NAVI: [(i16, i16); 9] = [
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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, DeriveValueType)]
pub struct NodeData(#[serde(with = "serde_bytes")] Vec<u8>);
impl NodeData {
    pub fn random() -> Self {
        let mut rng = thread_rng();
        let length = rng.gen_range(0..NODE_MAX_SIZE);
        let mut rtn = vec![0u8; length];
        rng.fill_bytes(&mut rtn);
        Self(rtn)
    }
    pub fn get(&self, index: usize) -> Option<i8>{
        self.0.get(index).map(|cell| i8::from_be_bytes([*cell]))
    }
    pub fn set(&mut self, index: usize, value: i8) -> Option<()>{
        self.0.get_mut(index).map(|cell| *cell = value.to_be_bytes()[0])
    }
}
impl Into<Vec<u8>> for NodeData {
    fn into(self) -> Vec<u8> {
        self.0
    }
}
impl From<Vec<u8>> for NodeData {
    fn from(value: Vec<u8>) -> Self {
        Self(value)
    }
}

#[derive(
    Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct NodeID(pub i16, pub i16);
impl NodeID {
    pub const UP_LEFT: Self = NodeID(i16::MIN, i16::MAX);
    pub const UP_MIDDLE: Self = NodeID(0, i16::MAX);
    pub const UP_RIGHT: Self = NodeID(i16::MAX, i16::MAX);
    pub const LEFT_MIDDLE: Self = NodeID(i16::MIN, 0);
    pub const ORIGIN: Self = NodeID(0, 0);
    pub const RIGHT_MIDDLE: Self = NodeID(i16::MAX, 0);
    pub const DOWN_LEFT: Self = NodeID(i16::MIN, i16::MIN);
    pub const DOWN_MIDDLE: Self = NodeID(0, i16::MIN);
    pub const DOWN_RIGHT: Self = NodeID(i16::MAX, i16::MIN);
}

impl NodeID {
    pub fn navi_to(&mut self, to: navi::Direction) -> NodeID {
        self.0 = self.0.wrapping_add(to.0);
        self.1 = self.1.wrapping_add(to.1);
        self.clone()
    }
}

impl From<NodeID> for i32 {
    fn from(value: NodeID) -> Self {
        (value.0 as i32) << 16 | (value.1) as i32
    }
}
impl Into<NodeID> for i32 {
    fn into(self) -> NodeID {
        let high = (self >> 16) as i16;
        let low = self as i16;
        NodeID(high, low)
    }
}
