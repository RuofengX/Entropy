use std::fmt;

use rand::{rngs::SmallRng, Rng, RngCore, SeedableRng};
use sea_orm::DeriveValueType;
use serde::{de, ser::SerializeTupleStruct, Deserialize, Serialize, Serializer};

use crate::entity::node;

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

pub const ALLOWED_NAVI: [(i16, i16); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

#[derive(
    Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct NodeID(pub i16, pub i16);
impl NodeID {
    pub const UP_LEFT: Self = NodeID(i16::MIN, i16::MAX);
    pub const UP_MIDDLE: Self = NodeID(0, i16::MAX);
    pub const UP_RIGHT: Self = NodeID(i16::MAX, i16::MAX);
    pub const LEFT_MIDDLE: Self = NodeID(i16::MIN, 0);
    pub const SITU: Self = NodeID(0, 0);
    pub const ORIGIN: Self = NodeID(0, 0);
    pub const RIGHT_MIDDLE: Self = NodeID(i16::MAX, 0);
    pub const DOWN_LEFT: Self = NodeID(i16::MIN, i16::MIN);
    pub const DOWN_MIDDLE: Self = NodeID(0, i16::MIN);
    pub const DOWN_RIGHT: Self = NodeID(i16::MAX, i16::MIN);

    pub fn from_i32(value: i32) -> Self {
        FlatID::from_i32(value).into_node_id()
    }

    pub fn from_xy(x: i16, y: i16) -> Self {
        Self(x, y)
    }

    pub fn into_tuple(self) -> (i16, i16) {
        self.into()
    }

    pub fn into_i32(self) -> i32 {
        self.into_flat().0
    }

    pub fn into_flat(self) -> FlatID {
        self.into()
    }

    pub fn navi_to(&mut self, to: navi::Direction) -> NodeID {
        self.0 = self.0.wrapping_add(to.0);
        self.1 = self.1.wrapping_add(to.1);
        self.clone()
    }
}

impl From<(i16, i16)> for NodeID {
    fn from(value: (i16, i16)) -> Self {
        NodeID(value.0, value.1)
    }
}
impl Into<(i16, i16)> for NodeID {
    fn into(self) -> (i16, i16) {
        (self.0, self.1)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, DeriveValueType)]
pub struct NodeData(Vec<i8>);
impl NodeData {
    pub fn random() -> Self {
        let mut rng = SmallRng::from_entropy();
        let length = rng.gen_range(0..NODE_MAX_SIZE);
        let mut rtn = vec![0u8; length];
        rng.fill_bytes(&mut rtn);
        let rtn = rtn
            .into_iter()
            .map(|cell| i8::from_be_bytes([cell]))
            .collect();
        Self(rtn)
    }
    pub fn get(&self, index: usize) -> Option<i8> {
        self.0.get(index).map(|x| *x)
    }
    pub fn set(&mut self, index: usize, value: i8) -> Option<()> {
        self.0.get_mut(index).map(|cell| *cell = value)
    }
    pub fn to_bytes(self) -> Vec<u8> {
        self.0
            .into_iter()
            .map(|cell| cell.to_be_bytes()[0])
            .collect()
    }
    pub fn from_bytes(value: Vec<u8>) -> Self {
        Self(value.into_iter().map(|b| i8::from_be_bytes([b])).collect())
    }
}

/// Default big endian encoding
impl Into<Vec<u8>> for NodeData {
    fn into(self) -> Vec<u8> {
        self.to_bytes()
    }
}
impl From<Vec<u8>> for NodeData {
    fn from(value: Vec<u8>) -> Self {
        Self::from_bytes(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeID,
    pub data: NodeData,
}
impl Node {
    pub fn from_model(value: node::Model) -> Self {
        value.into()
    }
}
impl From<node::Model> for Node {
    fn from(value: node::Model) -> Self {
        Node {
            id: FlatID::from(value.id).into_node_id(),
            data: value.data.into(),
        }
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    DeriveValueType,
)]
pub struct FlatID(
    #[serde(
        serialize_with = "crate::grid::ser_flat",
        deserialize_with = "crate::grid::de_flat"
    )]
    i32,
);
impl FlatID {
    pub fn into_node_id(self) -> NodeID {
        self.into()
    }

    pub fn into_tuple(self) -> (i16, i16) {
        self.into_node_id().into_tuple()
    }

    pub fn from_i32(value: i32) -> Self {
        Self::from(value)
    }

    pub fn from_xy(x: i16, y: i16) -> Self {
        NodeID::from_xy(x, y).into()
    }
}

impl From<i32> for FlatID {
    fn from(value: i32) -> Self {
        Self(value)
    }
}
impl Into<i32> for FlatID {
    fn into(self) -> i32 {
        self.0
    }
}
impl From<NodeID> for FlatID {
    fn from(value: NodeID) -> Self {
        let [x1, x2] = value.0.to_be_bytes();
        let [y1, y2] = value.1.to_be_bytes();
        let f = i32::from_be_bytes([x1, x2, y1, y2]);
        FlatID(f)
    }
}
impl Into<NodeID> for FlatID {
    fn into(self) -> NodeID {
        let [x1, x2, y1, y2] = self.0.to_be_bytes();
        NodeID::from_xy(i16::from_be_bytes([x1, x2]), i16::from_be_bytes([y1, y2]))
    }
}

pub(crate) fn ser_flat<S>(id: &i32, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let (x, y) = FlatID::from(*id).into_tuple();
    let mut tuple = serializer.serialize_tuple_struct("FlatID", 4)?;
    tuple.serialize_field(&x)?;
    tuple.serialize_field(&y)?;
    tuple.end()
}

pub(crate) fn de_flat<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Debug)]
    struct FlatIDVisitor;

    impl<'de> de::Visitor<'de> for FlatIDVisitor {
        type Value = i32;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a tuple of two i32 values")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<i32, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            let x = seq
                .next_element()?
                .ok_or_else(|| de::Error::invalid_length(0, &self))?;
            let y = seq
                .next_element()?
                .ok_or_else(|| de::Error::invalid_length(1, &self))?;

            if seq.next_element::<()>()?.is_some() {
                return Err(de::Error::invalid_length(2, &self));
            }

            Ok(FlatID::from_xy(x, y).into())
        }
    }

    deserializer.deserialize_tuple_struct("FlatID", 2, FlatIDVisitor)
}
