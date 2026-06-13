/// Defines the simulation time concept.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SimTime(pub u64);

impl SimTime {
    pub fn new(t: u64) -> Self {
        Self(t)
    }

    pub fn advance(&mut self, dt: u64) {
        self.0 += dt;
    }
}
