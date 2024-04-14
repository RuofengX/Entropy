use std::sync::{Arc, OnceLock};

use dashmap::DashMap;

use crate::{db::SledStorage, soul::WonderingSoul, world::World};

pub struct WheelConfig {
    pub temporary: bool,
}
impl Default for WheelConfig {
    fn default() -> Self {
        Self { temporary: false }
    }
}

pub struct Wheel {
    pub world: World<SledStorage>,
    pub souls: DashMap<String, WonderingSoul<SledStorage>, ahash::RandomState>,
}

impl Wheel {}

pub static WHEEL: OnceLock<Wheel> = OnceLock::new();
// Wheel::new(WheelConfig::default()));
