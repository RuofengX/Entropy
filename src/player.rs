use serde::{Deserialize, Serialize};

use crate::node::{direction, NodeID};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EID(i64);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Player {
    stand_on: NodeID,
}

impl Player{
    pub fn walk(&mut self, direction: direction::Direction){
        self.stand_on.0 += direction.0;
        self.stand_on.1 += direction.1;
        todo!()
    }
}
