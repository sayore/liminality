/// Defines node types and the universal Node wrapper.
use crate::filter::{GateExpr, ResourceFilter};
use crate::id::{ContractId, GateId, NodeId};
use crate::resource::{Amount, ResourceLedger};
use crate::space::SpacePos;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageNode {
    pub contents: ResourceLedger,
    pub accepts_filter: ResourceFilter,
    pub exposes_filter: ResourceFilter,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PipeNode {
    pub throughput: Option<Amount>,
    pub filter: ResourceFilter,
    pub directional: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WarpNode {
    pub target: SpacePos,
    pub latency: Option<u64>,
    pub throughput: Option<Amount>,
    pub filter: ResourceFilter,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransformerNode {
    pub contract: ContractId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputNode {
    pub external_id: String,
    pub filter: ResourceFilter,
    pub rate_limit: Option<Amount>,
    pub gate: Option<GateId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputNode {
    pub external_id: String,
    pub filter: ResourceFilter,
    pub rate_limit: Option<Amount>,
    pub gate: Option<GateId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GateNode {
    pub expr: GateExpr,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeKind {
    Storage(StorageNode),
    Pipe(PipeNode),
    Warp(WarpNode),
    Transformer(TransformerNode),
    Input(InputNode),
    Output(OutputNode),
    Gate(GateNode),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub pos: SpacePos,
    pub kind: NodeKind,
}
