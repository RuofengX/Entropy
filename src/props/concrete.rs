use crate::scalar::*;

use super::Tickable;

pub trait Positional {
    fn add_coordinate(&mut self, value:Coordinate);
}

pub trait Concrete: Tickable{
    fn add_massive(&mut self, value: Massive);
    fn move_force(&mut self) -> Force;
    fn calc_accelerate(&self) -> Accelerate;
}
