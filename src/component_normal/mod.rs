use crate::{props::Tickable, scalar::Massive};

pub mod pawn;

pub struct EID(pub u64);
impl AsRef<u64> for EID{
    fn as_ref(&self) -> &u64 {
        &self.0
    }
}

pub struct Base{
    pub holding: bool,
    pub id: EID,
    pub born_at_micros: i64,
}
impl Tickable for Base{
    fn tick(&mut self) {}
}

pub struct Move{
    pub mass: Massive,
}
