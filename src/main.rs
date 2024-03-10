pub mod props;

use std::sync::Arc;

use chrono::Utc;
use props::r#move::{Map, Movement, MovementComponent};
use serde::{Deserialize, Serialize};

use crate::props::r#move::{Accelerate, Coordinate, Velocity};

pub type EID = u64; // 更好的语义：EID是一个编号，不能指代数量

pub const WORLD_VERSION: [u8; 3] = [0, 0, 1];

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct World {
    entities: Vec<Entity>,
    map: Map,
    //save_model: Json
}
impl World {
    pub fn run(self){
    }
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
    pub fn register_movement(&mut self, id: EID, movement: Movement) {
        if let Some(entity) = self.get_ent(id) {
            if entity.movement_component.is_none() {
                return;
            }
        };
        let index = self.map.register(movement);
        self.get_ent_mut(id).unwrap().movement_component = Some(MovementComponent { index });
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Entity {
    pub holding: bool,
    pub id: EID,
    pub born_at_micros: i64,
    pub movement_component: Option<MovementComponent>,
}
impl Entity {
    pub fn new(id: EID) -> Self {
        Self {
            holding: true,
            id,
            born_at_micros: Utc::now().timestamp_micros(),
            movement_component: None,
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
    world.register_movement(
        idx0,
        Movement(
            Coordinate(0.0, 0.0, 0.0),
            Velocity(0.0, 0.0, 0.0),
            Accelerate(0.0, 0.0, 0.0),
        ),
    );

    let ent0 = world.get_ent(idx0).unwrap();
    dbg!(ent0);
}
