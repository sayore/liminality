//! The pure data model for Liminality.
//!
//! This crate contains the canonical simulation model plus a few root-level
//! DTOs that the protocol crate already consumes.

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SpacePos {
    pub w: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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
