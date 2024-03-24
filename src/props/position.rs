use crate::scalar::{Accelerate, Coordinate, Massive, Velocity};

use super::{concrete::Positional, Tickable};

pub trait Concrete {
    fn get_massive(&self) -> Massive;
    fn set_massive(&mut self, value: Massive);
}

pub trait Moveable: Positional + Concrete + Tickable + Sized
{
    fn get_velocity(&self) -> Velocity;
    fn get_velocity_mut(&mut self) -> &mut Velocity;
    fn set_velocity(&mut self, value: Velocity);

    fn get_accelerate(&self) -> Accelerate;
    fn get_accelerate_mut(&mut self) -> &mut Accelerate;
    fn set_accelerate(&mut self, value: Accelerate);
    fn add_accelerate(&mut self, value: Accelerate);

    /// move accelerate out of Self, leave a default value(0)
    fn move_accelerate(&mut self) -> Accelerate;
    /// get both mutable reference as once, to avoid double-mut-borrow error.
    fn get_coordinate_and_velocity_mut(&mut self) -> (&mut Coordinate, &mut Velocity);

    fn moveable_tick(&mut self) {
        let a = self.move_accelerate();
        let (p, v) = self.get_coordinate_and_velocity_mut();

        // update accelerate
        *v += a.into_velocity(1);

        // update position
        *p += v.into_coordinate(1);
    }
}
