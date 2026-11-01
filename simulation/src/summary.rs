use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sample {
    pub t: u64,
    pub i: f64,
    pub s: f64,
    pub r: f64,
    pub ever: f64,
}

impl Sample {
    pub fn from_counts(t: u64, n: usize, counts: (usize, usize, usize), ever: usize) -> Self {
        let scale = n as f64;
        Self {
            t,
            i: counts.0 as f64 / scale,
            s: counts.1 as f64 / scale,
            r: counts.2 as f64 / scale,
            ever: ever as f64 / scale,
        }
    }
}
