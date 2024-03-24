pub mod props;
pub mod component_normal;
pub mod scalar;

use chrono::Utc;
use serde::{Deserialize, Serialize};

pub type EID = u64; // 更好的语义：EID是一个编号，不能指代数量

pub const WORLD_VERSION: [u8; 3] = [0, 0, 1];

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct World {
    entities: Vec<Entity>,
    //save_model: Json
}
impl World {
    pub fn run(self) {}
    pub fn spawn(&mut self) -> EID {
        let len = self.entities.len() as u64;
        self.entities.push(Entity::new(len));
        len
    }
    pub fn get_ent(&self, id: EID) -> Option<&Entity> {
        self.entities.get(id as usize)
    }
    pub fn get_ent_mut(&mut self, id: EID) -> Option<&mut Entity> {
        self.entities.get_mut(id as usize)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Entity {
    pub holding: bool,
    pub id: EID,
    pub born_at_micros: i64,
}
impl Entity {
    pub fn new(id: EID) -> Self {
        Self {
            holding: true,
            id,
            born_at_micros: Utc::now().timestamp_micros(),
        }
    }
}

pub trait Property {
    fn new() -> Self;

    #[allow(unused_variables)]
    fn ignite(&mut self, world: &mut World) {}

    #[allow(unused_variables)]
    fn rolling(&mut self, world: &World) {}

    #[allow(unused_variables)]
    fn tick(&mut self, entity: &mut Entity, world: &World) {}

    #[allow(unused_variables)]
    fn exclusive_tick(&mut self, world: &mut World) {}
}

pub trait Component {}

fn main() {
    println!("Hello, world!");
    let mut world = World::default();
    let idx0 = world.spawn();

    let ent0 = world.get_ent(idx0).unwrap();
    dbg!(ent0);
}
