use std::{collections::HashMap, sync::Arc};

use crate::{
    soul::WonderingSoul,
    world::{SaveStorage, SledBackend, World},
};

pub struct WheelConfig {
    pub temporary: bool,
}
pub struct Wheel {
    world: Arc<World>,
    souls: HashMap<String, WonderingSoul>,
}

impl Wheel {
    pub fn new(config: WheelConfig) -> Wheel {
        let back = Arc::new(SledBackend::new(config.temporary));
        let soul_list = back.as_ref().load_souls();
        let world = Arc::new(World::new(back));
        Wheel {
            world: world.clone(),
            souls: soul_list
                .into_iter()
                .map(|x| (x.uid.clone(), WonderingSoul::from_soul(x, world.clone())))
                .collect(),
        }
    }
}
