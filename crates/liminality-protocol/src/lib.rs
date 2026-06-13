//! liminality-protocol
//!
//! This crate defines serializable DTOs/messages for snapshots, deltas, queries, and responses.
//! It depends on liminality-model.
//! It must not own simulation logic.

use liminality_model::{Resource, SpacePos};
use serde::{Deserialize, Serialize};

/// 1. Handshake Role
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Role {
    BrowserUi,
    MinecraftBridge,
    TestClient,
    Unknown,
}

/// Handshake Messages
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ClientHello {
    pub protocol_version: u32,
    pub client_name: String,
    pub role: Role,
    pub auth_token: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ServerHello {
    pub protocol_version: u32,
    pub server_name: String,
}

/// 2. Snapshot
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct WorldSnapshot {
    pub protocol_version: u32,
    pub model_version: u32,
    pub current_sim_time: u64,
    pub spaces: Vec<String>,
    // Assuming simple string representation for nodes and edges for now as generic DTOs
    pub nodes: Vec<String>,
    pub edges: Vec<String>,
    pub resource_registry_summary: Vec<String>,
    pub contract_registry_summary: Vec<String>,
}

/// 3. Delta
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum WorldDelta {
    NodeAdded { node_id: String },
    NodeRemoved { node_id: String },
    NodeChanged { node_id: String },
    EdgeAdded { edge_id: String },
    EdgeRemoved { edge_id: String },
    LedgerChanged { ledger_id: String },
    SegmentChanged { segment_id: String },
    TimeAdvanced { new_time: u64 },
    FullResyncRequired,
}

/// 4. Query
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Query {
    StateAt {
        time: u64,
    },
    NodeAt {
        pos: SpacePos,
    },
    NodeView {
        node: String,
        time: u64,
    },
    RegionView {
        space: String,
        min: SpacePos,
        max: SpacePos,
        time: u64,
    },
    PredictUntil {
        time: u64,
    },
    WorldSnapshot,
}

/// 5. Response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Response {
    Snapshot(WorldSnapshot),
    StateAt(String),    // Placeholder for actual state representation
    NodeView(String),   // Placeholder for actual node representation
    RegionView(String), // Placeholder for actual region representation
    Prediction(String), // Placeholder for actual prediction representation
    Error(String),
}

/// 6. Events from bridge
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum BridgeEvent {
    ExternalNodeObserved {
        node_id: String,
    },
    ExternalNodeRemoved {
        node_id: String,
    },
    ExternalInventorySample {
        node_id: String,
        resources: Vec<Resource>,
    },
    ExternalTransformerSample {
        node_id: String,
        status: String,
    },
    ExternalTopologyChanged,
    ExternalTickMetrics {
        metrics: String,
    },
}

/// Protocol Envelope
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ProtocolEnvelope<T> {
    pub message_id: String,
    pub protocol_version: u32,
    pub timestamp: Option<u64>,
    pub payload: T,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_space_pos_roundtrip() {
        let pos = SpacePos {
            w: "space1".to_string(),
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };
        let json = serde_json::to_string(&pos).unwrap();
        let deserialized: SpacePos = serde_json::from_str(&json).unwrap();
        assert_eq!(pos, deserialized);
    }

    #[test]
    fn test_handshake_roundtrip() {
        let hello = ClientHello {
            protocol_version: 1,
            client_name: "test".to_string(),
            role: Role::MinecraftBridge,
            auth_token: None,
        };
        let json = serde_json::to_string(&hello).unwrap();
        let deserialized: ClientHello = serde_json::from_str(&json).unwrap();
        assert_eq!(hello, deserialized);

        let s_hello = ServerHello {
            protocol_version: 1,
            server_name: "server1".to_string(),
        };
        let s_json = serde_json::to_string(&s_hello).unwrap();
        let s_deserialized: ServerHello = serde_json::from_str(&s_json).unwrap();
        assert_eq!(s_hello, s_deserialized);
    }

    #[test]
    fn test_snapshot_roundtrip() {
        let snapshot = WorldSnapshot {
            protocol_version: 1,
            model_version: 2,
            current_sim_time: 1000,
            spaces: vec!["space1".to_string()],
            nodes: vec!["node1".to_string()],
            edges: vec!["edge1".to_string()],
            resource_registry_summary: vec!["res1".to_string()],
            contract_registry_summary: vec!["contract1".to_string()],
        };
        let json = serde_json::to_string(&snapshot).unwrap();
        let deserialized: WorldSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(snapshot, deserialized);
    }

    #[test]
    fn test_delta_roundtrip() {
        let delta = WorldDelta::NodeAdded {
            node_id: "node1".to_string(),
        };
        let json = serde_json::to_string(&delta).unwrap();
        let deserialized: WorldDelta = serde_json::from_str(&json).unwrap();
        assert_eq!(delta, deserialized);

        let delta2 = WorldDelta::FullResyncRequired;
        let json2 = serde_json::to_string(&delta2).unwrap();
        let deserialized2: WorldDelta = serde_json::from_str(&json2).unwrap();
        assert_eq!(delta2, deserialized2);
    }

    #[test]
    fn test_query_response_roundtrip() {
        let query = Query::NodeAt {
            pos: SpacePos {
                w: "space".to_string(),
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        };
        let json_query = serde_json::to_string(&query).unwrap();
        let deserialized_query: Query = serde_json::from_str(&json_query).unwrap();
        assert_eq!(query, deserialized_query);

        let response = Response::Error("Unknown".to_string());
        let json_resp = serde_json::to_string(&response).unwrap();
        let deserialized_resp: Response = serde_json::from_str(&json_resp).unwrap();
        assert_eq!(response, deserialized_resp);
    }

    #[test]
    fn test_envelope_roundtrip() {
        let envelope = ProtocolEnvelope {
            message_id: "msg_123".to_string(),
            protocol_version: 1,
            timestamp: Some(500),
            payload: WorldDelta::TimeAdvanced { new_time: 100 },
        };
        let json = serde_json::to_string(&envelope).unwrap();
        let deserialized: ProtocolEnvelope<WorldDelta> = serde_json::from_str(&json).unwrap();
        assert_eq!(envelope, deserialized);
    }
}
