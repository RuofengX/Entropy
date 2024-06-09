use rand::{thread_rng, Rng};
use sea_orm::{DeriveValueType, TryFromU64};
use serde::{Deserialize, Serialize};
use std::{borrow::Borrow, fmt::Display, hash::Hash};

use crate::err::ModelError;

pub const NODE_SIZE: usize = 1024;

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

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
pub struct NodeData(#[serde(with = "serde_bytes")] pub [u8; NODE_SIZE]);
impl NodeData {
    pub fn random() -> Self {
        let mut rtn = [0u8; 1024];
        thread_rng().fill(&mut rtn);
        Self(rtn)
    }

    pub fn to_bytes(&self) -> &[u8] {
        self.0.borrow()
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}
impl From<[u8; NODE_SIZE]> for NodeData {
    fn from(value: [u8; NODE_SIZE]) -> Self {
        NodeData(value)
    }
}

impl TryFrom<Vec<u8>> for NodeData {
    type Error = ModelError;
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        if let Some(b) = value.first_chunk::<NODE_SIZE>() {
            Ok(NodeData(*b))
        } else {
            Err(ModelError::Parse {
                desc: format!("data length not correct <- {}", value.len()),
            })
        }
    }
}

#[derive(
    Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct NodeID(pub i16, pub i16);
impl NodeID {
    pub const POLAR_UP_LEFT: Self = NodeID(i16::MIN, i16::MAX);
    pub const POLAR_UP_MIDDLE: Self = NodeID(0, i16::MAX);
    pub const POLAR_UP_RIGHT: Self = NodeID(i16::MAX, i16::MAX);
    pub const POLAR_LEFT_MIDDLE: Self = NodeID(i16::MIN, 0);
    pub const POLAR_ORIGIN: Self = NodeID(0, 0);
    pub const POLAR_RIGHT_MIDDLE: Self = NodeID(i16::MAX, 0);
    pub const POLAR_DOWN_LEFT: Self = NodeID(i16::MIN, i16::MIN);
    pub const POLAR_DOWN_MIDDLE: Self = NodeID(0, i16::MIN);
    pub const POLAR_DOWN_RIGHT: Self = NodeID(i16::MAX, i16::MIN);
}

impl NodeID {
    pub fn navi_to(&mut self, to: navi::Direction) -> NodeID {
        self.0 = self.0.wrapping_add(to.0);
        self.1 = self.1.wrapping_add(to.1);
        self.clone()
    }
}

#[derive(
    Debug,
    Hash,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    DeriveValueType,
    Serialize,
    Deserialize,
)]
pub struct FlatID(pub u32);
impl FlatID {
    pub fn into_node_id(self) -> NodeID {
        self.into()
    }
}
impl From<NodeID> for FlatID {
    fn from(value: NodeID) -> Self {
        FlatID((value.0 as u32) << 16 | (value.1) as u32)
    }
}
impl Into<NodeID> for FlatID {
    fn into(self) -> NodeID {
        let high = (self.0 >> 16) as i16;
        let low = self.0 as i16;
        NodeID(high, low)
    }
}

impl TryFromU64 for FlatID {
    fn try_from_u64(n: u64) -> Result<Self, sea_orm::prelude::DbErr> {
        if n <= u32::MAX as u64 {
            Ok(FlatID(n as u32))
        } else {
            Err(sea_orm::DbErr::Custom(
                "Node index out of range. Database may be posioned.".to_string(),
            ))
        }
    }
}

impl Display for FlatID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let id = self.clone().into_node_id();
        f.write_fmt(format_args!("({0},{1})", id.0, id.1))
    }
}
