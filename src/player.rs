use serde::{Deserialize, Serialize};

use crate::node::{direction, NodeID};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EID(i64);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Guest {
    on: NodeID,
}

impl Guest{
    pub fn walk(&mut self, direction: direction::Direction){
        self.on.0 += direction.0;
        self.on.1 += direction.1;
        todo!()
    }
}
