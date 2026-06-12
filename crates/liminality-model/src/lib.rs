//! liminality-model
//!
//! This crate contains the pure data model for liminality.
//! It defines spatial/time primitives and base graph/resource structures.
//!
//! It contains no simulation logic, no networking, and no daemon/runtime behavior.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Resource {
    pub id: String,
    pub quantity: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct WorldModel {
    pub coal_storage: u32,
    pub ore_storage: u32,
    pub output_storage: u32,
}

impl WorldModel {
    pub fn furnace_line_demo() -> Self {
        Self {
            coal_storage: 32,
            ore_storage: 128,
            output_storage: 0,
        }
    }
}
