use std::{collections::HashMap, sync::Arc};

use crate::{soul::Soul, world::World};

pub struct Wheel {
    world: Arc<World>,
    souls: HashMap<u64, Soul>,
}
