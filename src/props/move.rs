use std::ops::AddAssign;

use serde::{Deserialize, Serialize};

use crate::{Component, Property};

pub type X = f64;
pub type Y = f64;
pub type Z = f64;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Coordinate(pub X, pub Y, pub Z);

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Velocity(pub X, pub Y, pub Z);

impl AddAssign<&Velocity> for Coordinate {
    fn add_assign(&mut self, rhs: &Velocity) {
        self.0 += rhs.0;
        self.1 += rhs.1;
        self.2 += rhs.2;
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Accelerate(pub X, pub Y, pub Z);
impl AddAssign<&Accelerate> for Velocity {
    fn add_assign(&mut self, rhs: &Accelerate) {
        self.0 += rhs.0;
        self.1 += rhs.1;
        self.2 += rhs.2;
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Movement(pub Coordinate, pub Velocity, pub Accelerate);
impl Movement {
    pub fn tick(&mut self) {
        self.0 += &self.1;
        self.1 += &self.2;
        self.2 = Accelerate::default();
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Map {
    data: Vec<Movement>,
}
impl Map {
    pub fn register(&mut self, movement: Movement) -> usize {
        self.data.push(movement);
        self.data.len() - 1
    }
}

impl Property for Map {
    fn new() -> Self {
        Self::default()
    }

    fn exclusive_tick(&mut self, world: &mut crate::World) {
        let data = &mut self.data;
        world.entities.iter().for_each(|e| {
            if let Some(vc) = &e.movement_component {
                unsafe { data.get_unchecked_mut(vc.index).tick() };
            }
        });
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MovementComponent {
    pub index: usize,
}
impl Component for MovementComponent {}
