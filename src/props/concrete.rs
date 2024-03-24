use crate::scalar::*;

pub trait Positional {
    fn get_coordinate(&self) -> Coordinate;
    fn get_coordinate_mut(&self) -> &mut Coordinate;
    fn set_coordinate(&self, value: Coordinate);
}

pub trait Concrete{
    fn get_massive(&self) -> Massive;
    fn set_massive(&mut self, value: Massive);
    fn get_force(&self) -> Massive;
    fn set_force(&mut self, value: Massive);
}
