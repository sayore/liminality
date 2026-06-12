/// Defines spatial coordinates across different spaces.
use crate::id::SpaceId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpacePos {
    pub w: SpaceId,
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl SpacePos {
    pub fn new(w: SpaceId, x: i32, y: i32, z: i32) -> Self {
        Self { w, x, y, z }
    }
}
