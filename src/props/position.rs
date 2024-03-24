use crate::scalar::{Accelerate, Coordinate, Velocity};

use super::{
    concrete::{Concrete, Positional},
    Tickable,
};

pub trait Moveable: Positional + Concrete + Tickable + Sized {
    fn add_accelerate(&mut self, value: Accelerate);

    /// get both mutable reference as once, to avoid double-mut-borrow error.
    fn get_coordinate_and_velocity_mut(&mut self) -> (&mut Coordinate, &mut Velocity);

    fn tick_moveable(&mut self) {
        let a = self.calc_accelerate();
        let (p, v) = self.get_coordinate_and_velocity_mut();

        // update accelerate
        *v += a.into_velocity(1);

        // update position
        *p += v.into_coordinate(1);
    }
}
