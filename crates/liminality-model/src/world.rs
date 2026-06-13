/// Defines the canonical simulation world state.
use crate::contract::TransformerContract;
use crate::edge::Edge;
use crate::id::{ContractId, EdgeId, NodeId, ResourceId, SpaceId};
use crate::node::Node;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Space {
    pub id: SpaceId,
    pub nodes: HashMap<NodeId, Node>,
    pub edges: HashMap<EdgeId, Edge>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceRegistry {
    pub resources: Vec<ResourceId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractRegistry {
    pub contracts: HashMap<ContractId, TransformerContract>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldModel {
    pub spaces: HashMap<SpaceId, Space>,
    pub resources: ResourceRegistry,
    pub contracts: ContractRegistry,
}
