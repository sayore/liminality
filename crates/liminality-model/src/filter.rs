/// Defines resource filters and gate expressions.
use crate::id::ResourceId;
use crate::resource::Amount;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceFilter {
    Any,
    Exact(ResourceId),
    OneOf(Vec<ResourceId>),
    None,
}

impl ResourceFilter {
    pub fn matches(&self, resource: &ResourceId) -> bool {
        match self {
            Self::Any => true,
            Self::Exact(id) => id == resource,
            Self::OneOf(ids) => ids.contains(resource),
            Self::None => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GateExpr {
    Always,
    ResourceAtLeast(ResourceId, Amount),
    ResourceBelow(ResourceId, Amount),
    And(Box<GateExpr>, Box<GateExpr>),
    Or(Box<GateExpr>, Box<GateExpr>),
    Not(Box<GateExpr>),
}
