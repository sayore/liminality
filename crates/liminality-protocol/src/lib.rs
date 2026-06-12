//! liminality-protocol
//!
//! This crate defines serializable DTOs/messages for snapshots, deltas, queries, and responses.
//! It depends on liminality-model.
//! It must not own simulation logic.

use liminality_model::WorldModel;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorldSnapshot {
    pub state: WorldModel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Query {
    pub seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Response {
    pub state: WorldSnapshot,
}
