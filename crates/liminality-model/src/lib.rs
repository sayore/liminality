//! The pure data model for Liminality.
//! This crate is the canonical representation of a simulated logistics world.

use serde::{Deserialize, Serialize};

pub mod contract;
pub mod edge;
pub mod error;
pub mod filter;
pub mod id;
pub mod node;
pub mod resource;
pub mod space;
pub mod time;
pub mod world;

// Legacy DTOs kept at the crate root so the protocol slice merged on main
// continues to compile while the richer model modules land underneath.
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

#[cfg(test)]
mod tests {
    use crate::contract::TransformerContract;
    use crate::filter::{GateExpr, ResourceFilter};
    use crate::id::{ContractId, ResourceId, SpaceId};
    use crate::resource::{Amount, ResourceAmount, ResourceLedger};
    use crate::space::SpacePos;
    use crate::time::SimTime;

    #[test]
    fn test_space_pos_creation() {
        let space_id = SpaceId::from("dimension_alpha");
        let pos = SpacePos::new(space_id.clone(), 10, 20, 30);
        assert_eq!(pos.w, space_id);
        assert_eq!(pos.x, 10);
        assert_eq!(pos.y, 20);
        assert_eq!(pos.z, 30);
    }

    #[test]
    fn test_storage_add_subtract() {
        let mut ledger = ResourceLedger::new();
        let res_id = ResourceId::from("iron_ingot");

        assert!(ledger.add_amount(res_id.clone(), Amount(100)).is_ok());
        assert_eq!(ledger.get_amount(&res_id), Amount(100));

        assert!(ledger.subtract_amount(&res_id, Amount(40)).is_ok());
        assert_eq!(ledger.get_amount(&res_id), Amount(60));

        assert!(ledger.subtract_amount(&res_id, Amount(60)).is_ok());
        assert_eq!(ledger.get_amount(&res_id), Amount(0));
    }

    #[test]
    fn test_storage_subtract_below_zero() {
        let mut ledger = ResourceLedger::new();
        let res_id = ResourceId::from("copper_ingot");

        ledger.add_amount(res_id.clone(), Amount(50)).unwrap();

        let err = ledger.subtract_amount(&res_id, Amount(60)).unwrap_err();
        match err {
            crate::error::ModelError::InsufficientResources {
                resource,
                requested,
                available,
            } => {
                assert_eq!(resource, res_id);
                assert_eq!(requested, 60);
                assert_eq!(available, 50);
            }
            _ => panic!("Expected InsufficientResources error"),
        }
    }

    #[test]
    fn test_filter_matching() {
        let res_iron = ResourceId::from("iron");
        let res_copper = ResourceId::from("copper");

        let filter_any = ResourceFilter::Any;
        assert!(filter_any.matches(&res_iron));

        let filter_exact = ResourceFilter::Exact(res_iron.clone());
        assert!(filter_exact.matches(&res_iron));
        assert!(!filter_exact.matches(&res_copper));

        let filter_one_of = ResourceFilter::OneOf(vec![res_iron.clone(), res_copper.clone()]);
        assert!(filter_one_of.matches(&res_iron));
        assert!(filter_one_of.matches(&res_copper));
        assert!(!filter_one_of.matches(&ResourceId::from("gold")));

        let filter_none = ResourceFilter::None;
        assert!(!filter_none.matches(&res_iron));
    }

    #[test]
    fn test_gate_expr_building() {
        let res_power = ResourceId::from("power");

        let expr = GateExpr::And(
            Box::new(GateExpr::ResourceAtLeast(res_power.clone(), Amount(100))),
            Box::new(GateExpr::Not(Box::new(GateExpr::ResourceBelow(
                res_power.clone(),
                Amount(10),
            )))),
        );

        let serialized = serde_json::to_string(&expr).unwrap();
        let deserialized: GateExpr = serde_json::from_str(&serialized).unwrap();
        assert_eq!(expr, deserialized);
    }

    #[test]
    fn test_contract_serialization() {
        let contract = TransformerContract {
            id: ContractId::from("smelt_iron"),
            inputs: vec![ResourceAmount {
                resource: ResourceId::from("raw_iron"),
                amount: Amount(1),
            }],
            outputs: vec![ResourceAmount {
                resource: ResourceId::from("iron_ingot"),
                amount: Amount(1),
            }],
            catalysts: vec![],
            fuel: vec![ResourceAmount {
                resource: ResourceId::from("coal"),
                amount: Amount(1),
            }],
            duration: SimTime::new(100),
            deterministic: true,
            parallelism: 1,
        };

        let serialized = serde_json::to_string(&contract).unwrap();
        let deserialized: TransformerContract = serde_json::from_str(&serialized).unwrap();
        assert_eq!(contract, deserialized);
    }

    #[test]
    fn test_warp_connects_different_spaces() {
        use crate::node::{NodeKind, WarpNode};

        let space1 = SpaceId::from("dimension_alpha");
        let space2 = SpaceId::from("dimension_beta");

        let target_pos = SpacePos::new(space2.clone(), 0, 0, 0);

        let warp = WarpNode {
            target: target_pos.clone(),
            latency: None,
            throughput: None,
            filter: ResourceFilter::Any,
        };

        let node_kind = NodeKind::Warp(warp);

        if let NodeKind::Warp(w) = node_kind {
            assert_ne!(space1, w.target.w);
            assert_eq!(space2, w.target.w);
            assert_eq!(target_pos, w.target);
        } else {
            panic!("Expected WarpNode");
        }
    }
}
