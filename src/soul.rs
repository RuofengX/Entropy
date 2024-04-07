use std::sync::Arc;

use crate::{guest::GID, world::World};

pub struct Soul {
    pub id: u64,
    pub username: String,
    password: String,
    owned_guest: Vec<GID>,
    connected_world: Vec<Arc<World>>,
}
