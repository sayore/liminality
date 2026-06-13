/// Defines edges which connect nodes.
use crate::filter::ResourceFilter;
use crate::id::{EdgeId, NodeId};
use crate::resource::Amount;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Edge {
    pub id: EdgeId,
    pub from: NodeId,
    pub to: NodeId,
    pub filter: Option<ResourceFilter>,
    pub throughput: Option<Amount>,
    pub latency: Option<u64>,
}
