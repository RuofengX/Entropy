use std::sync::{Arc, OnceLock};

use dashmap::DashMap;

use crate::{
    soul::WonderingSoul,
    world::{SaveStorage, SledBackend, World},
};

pub struct WheelConfig {
    pub temporary: bool,
}
impl Default for WheelConfig {
    fn default() -> Self {
        Self { temporary: false }
    }
}

pub struct Wheel {
    world: Arc<World>,
    souls: DashMap<String, WonderingSoul, ahash::RandomState>,
}

impl Wheel {
    pub fn init(config: WheelConfig) {
        let back = Arc::new(SledBackend::new(config.temporary));
        let soul_list = back.as_ref().load_souls();
        let world = Arc::new(World::new(back));
        let _ = WHEEL.set(Wheel {
            world: world.clone(),
            souls: soul_list
                .into_iter()
                .map(|x| (x.uid.clone(), WonderingSoul::from_soul(x, world.clone())))
                .collect(),
        });
    }

}

pub static WHEEL: OnceLock<Wheel> = OnceLock::new();
// Wheel::new(WheelConfig::default()));
