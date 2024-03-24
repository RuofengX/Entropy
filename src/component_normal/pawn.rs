use crate::{
    props::{
        concrete::{Concrete, Positional},
        position::Moveable,
        Tickable,
    },
    scalar::{Accelerate, Coordinate, Force, Massive, Velocity},
};

/// A Component that moves
pub struct Claw {
    mass: Massive,
    force: Force,
    velo: Velocity,
    coor: Coordinate,
    acc: Accelerate,
}
impl Tickable for Claw {
    fn tick(&mut self) {
        self.tick_moveable()
    }
}

impl Concrete for Claw {
    fn add_massive(&mut self, value: Massive) {
        self.mass += value
    }

    fn move_force(&mut self) -> Force {
        let rtn = self.force;
        self.force = Default::default();
        rtn
    }

    fn calc_accelerate(&self) -> Accelerate {
        self.force / self.mass
    }
}
impl Positional for Claw {
    fn add_coordinate(&mut self, value: Coordinate) {
        self.coor += value
    }
}
impl Moveable for Claw {
    fn add_accelerate(&mut self, value: Accelerate) {
        self.acc += value
    }

    fn get_coordinate_and_velocity_mut(&mut self) -> (&mut Coordinate, &mut Velocity) {
        (&mut self.coor, &mut self.velo)
    }
}
