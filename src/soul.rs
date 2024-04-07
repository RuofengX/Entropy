use std::sync::Arc;

use ahash::HashMap;

use crate::{guest::GID, node::direction, world::World};

pub struct Soul {
    pub id: u64,
    pub username: String,
    password: String,
    owned_guest: HashMap<GID, Arc<World>>,
}
impl Soul {
    pub fn walk(&mut self, direction: direction::Direction) {
        todo!()
        // self.node.0 += direction.0;
        // self.node.1 += direction.1;
        // self.energy -= self.walk_cost;
    }
}
