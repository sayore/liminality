use crate::id::ResourceId;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum ModelError {
    #[error(
        "Capacity exceeded when adding resource {resource:?}: attempted to add {attempted}, but only {available} space available"
    )]
    CapacityExceeded {
        resource: ResourceId,
        attempted: u64,
        available: u64,
    },
    #[error(
        "Insufficient resources for {resource:?}: requested {requested}, but only {available} available"
    )]
    InsufficientResources {
        resource: ResourceId,
        requested: u64,
        available: u64,
    },
}
