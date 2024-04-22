use ordered_float::NotNan;
use serde::{Deserialize, Serialize};

use crate::node::NodeID;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Serialize, Deserialize)]
pub struct GID(pub u64);
impl GID {
    pub fn get_then_increase(&mut self) -> GID {
        let rtn = self.clone();
        self.0 += 1;
        rtn
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Guest {
    pub id: GID,
    pub node: NodeID,
    pub energy: u64,
    pub walk_cost: u64,
    temperature: u8,
    engine_efficiency: NotNan<f32>,
}

impl Guest {
    /// used by World::Spawn
    pub(crate) fn new(id: GID, node: NodeID) -> Guest {
        Guest {
            id,
            node,
            temperature: 128,
            energy: 100u64,
            engine_efficiency: NotNan::new(0.8).unwrap(),
            walk_cost: 0u64,
        }
    }

    pub(crate) fn is_energy_enough(&self, cost: u64) -> bool {
        self.energy >= cost
    }

    pub(crate) fn carnot_efficiency(&self, cell: &u8) -> f32 {
        let s = unsafe { NotNan::new_unchecked(self.temperature as f32) };
        let o = unsafe { NotNan::new_unchecked(*cell as f32) };
        let (h, c) = if s > o { (*s, *o) } else { (*o, *s) };
        1f32 - c / h
    }

    pub(crate) fn total_efficiency(&self, cell: &u8) -> f32 {
        *self.engine_efficiency * self.carnot_efficiency(cell)
    }

    /// Return the energy generated by the guest.
    /// The guest will generate energy based on its temperature.
    /// If the guest is hotter than the cell, it will consume the cell's energy and increase its own temperature.
    /// If the guest is colder than the cell, it will produce energy and decrease its own temperature.
    /// The guest will also lose some of its energy for the efficiency cost.
    pub(crate) fn generate_energy(&mut self, cell: &mut u8) -> u8 {
        // Calculate the delta energy first, for borrow check
        let delta = self.temperature.abs_diff(*cell);
        let delta = (self.total_efficiency(cell) * delta as f32).floor() as u8;

        // no overflow will happen, the efficiency proves that, so no need to check

        // Determine which temperature is hotter and colder.
        // and go change
        if self.temperature > *cell {
            self.temperature -= delta;
            *cell += delta;
        } else if self.temperature < *cell {
            self.temperature += delta;
            *cell -= delta;
        } else {
            ()
        };

        self.energy += delta as u64;
        delta
    }
}
