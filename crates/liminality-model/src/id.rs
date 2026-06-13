/// Defines type-safe identifiers for various model entities.
use serde::{Deserialize, Serialize};

macro_rules! define_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
        pub struct $name(pub String);

        impl From<&str> for $name {
            fn from(s: &str) -> Self {
                Self(s.to_string())
            }
        }

        impl From<String> for $name {
            fn from(s: String) -> Self {
                Self(s)
            }
        }
    };
}

define_id!(SpaceId);
define_id!(NodeId);
define_id!(EdgeId);
define_id!(ResourceId);
define_id!(ContractId);
define_id!(GateId);
