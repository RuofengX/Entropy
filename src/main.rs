pub mod node;
pub mod world;
mod err;
mod player;

use serde::{Deserialize, Serialize};

pub fn entropy(s: &[u8]) -> f32 {
    let mut histogram = [0u64; 256];

    for &b in s {
        histogram[b as usize] += 1;
    }

    histogram
        .iter()
        .cloned()
        .filter(|&h| h != 0)
        .map(|h| h as f32 / s.len() as f32)
        .map(|ratio| -ratio * ratio.log2())
        .sum()
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Item {
    name: String,
    id: usize,
    mass: f64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Slot {
    index: usize,
    aisle: Vec<u64>,
    data: [u8; 32],
}
impl Slot {
    pub fn new(index: usize) -> Self {
        let mut data = [0; 32];
        data.iter_mut().for_each(|b| {
            *b = rand::random::<u8>();
        });

        Slot {
            index,
            aisle: Vec::new(),
            data,
        }
    }
    pub fn get_entropy(&self) -> f32 {
        entropy(&self.data)
    }
}

fn main() {
}
