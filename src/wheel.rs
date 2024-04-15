use std::sync::OnceLock;

use crate::{db::SledStorage, world::World};

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
}

impl Wheel {}

pub static WHEEL: OnceLock<Wheel> = OnceLock::new();
// Wheel::new(WheelConfig::default()));
