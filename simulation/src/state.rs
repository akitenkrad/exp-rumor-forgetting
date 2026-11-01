use rand::Rng;
use serde::{Deserialize, Serialize};
use socsim_core::SimRng;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[repr(u8)]
pub enum RumorState {
    Ignorant = 0,
    Spreader = 1,
    Stifler = 2,
}

#[derive(Clone, Debug)]
pub struct IndexedSet {
    values: Vec<usize>,
    positions: Vec<usize>,
}

impl IndexedSet {
    pub fn new(capacity: usize) -> Self {
        Self {
            values: Vec::new(),
            positions: vec![usize::MAX; capacity],
        }
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn contains(&self, value: usize) -> bool {
        self.positions.get(value).is_some_and(|position| *position != usize::MAX)
    }

    pub fn insert(&mut self, value: usize) -> bool {
        if self.contains(value) {
            return false;
        }
        self.positions[value] = self.values.len();
        self.values.push(value);
        true
    }

    pub fn remove(&mut self, value: usize) -> bool {
        if !self.contains(value) {
            return false;
        }
        let position = self.positions[value];
        let moved = *self.values.last().expect("membership implies a value");
        self.values.swap_remove(position);
        self.positions[value] = usize::MAX;
        if position < self.values.len() {
            self.positions[moved] = position;
        }
        true
    }

    pub fn sample(&self, rng: &mut SimRng) -> Option<usize> {
        (!self.values.is_empty()).then(|| self.values[rng.gen_range(0..self.values.len())])
    }

    pub fn values(&self) -> &[usize] {
        &self.values
    }
}
