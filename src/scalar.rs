use std::ops::AddAssign;

pub trait Scalar: Clone + Copy + Send + Sync + AddAssign<Self> {}

pub const DIMENSION: usize = 3;
pub type TIME = usize; // do not change this;
pub type LENGTH = f64;
pub type MASSIVE = f64;
pub type FORCE = f64;

#[derive(Debug, Clone, Copy)]
pub struct Force(pub [LENGTH; DIMENSION]);
impl Scalar for Force {}
impl AddAssign<Force> for Force {
    fn add_assign(&mut self, rhs: Force) {
        for i in 0..DIMENSION {
            self.0[i] += rhs.0[i];
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Coordinate(pub [LENGTH; DIMENSION]);
impl Scalar for Coordinate {}
impl AddAssign<Coordinate> for Coordinate {
    fn add_assign(&mut self, rhs: Coordinate) {
        for i in 0..DIMENSION {
            self.0[i] += rhs.0[i];
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Massive(pub MASSIVE);
impl Scalar for Massive {}
impl AddAssign<Massive> for Massive {
    fn add_assign(&mut self, rhs: Massive) {
        self.0 += rhs.0;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Velocity(pub [LENGTH; DIMENSION]);
impl Scalar for Velocity {}
impl AddAssign<Velocity> for Velocity {
    fn add_assign(&mut self, rhs: Velocity) {
        for i in 0..DIMENSION {
            self.0[i] += rhs.0[i];
        }
    }
}
impl Velocity {
    pub fn into_coordinate(mut self, time: usize) -> Coordinate {
        for i in 0..DIMENSION {
            self.0[i] *= time as f64;
        }
        Coordinate(self.0)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Accelerate(pub [LENGTH; DIMENSION]);
impl Scalar for Accelerate {}
impl AddAssign<Accelerate> for Accelerate {
    fn add_assign(&mut self, rhs: Accelerate) {
        for i in 0..DIMENSION {
            self.0[i] += rhs.0[i];
        }
    }
}
impl Accelerate {
    pub fn into_velocity(mut self, time: usize) -> Velocity {
        for i in 0..DIMENSION {
            self.0[i] *= time as f64;
        }
        Velocity(self.0)
    }
}
