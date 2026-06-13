/// Defines contracts for transformers.
use crate::id::ContractId;
use crate::resource::ResourceAmount;
use crate::time::SimTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransformerContract {
    pub id: ContractId,
    pub inputs: Vec<ResourceAmount>,
    pub outputs: Vec<ResourceAmount>,
    pub catalysts: Vec<ResourceAmount>,
    pub fuel: Vec<ResourceAmount>,
    pub duration: SimTime,
    pub deterministic: bool,
    pub parallelism: u32,
}
