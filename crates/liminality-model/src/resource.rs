/// Defines resources, amounts, and storage ledgers.
use crate::id::ResourceId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Amount(pub u64);

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceAmount {
    pub resource: ResourceId,
    pub amount: Amount,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ResourceLedger {
    pub contents: HashMap<ResourceId, Amount>,
    pub capacity: Option<Amount>,
}

use crate::error::ModelError;

impl ResourceLedger {
    pub fn new() -> Self {
        Self {
            contents: HashMap::new(),
            capacity: None,
        }
    }

    pub fn with_capacity(capacity: u64) -> Self {
        Self {
            contents: HashMap::new(),
            capacity: Some(Amount(capacity)),
        }
    }

    pub fn get_amount(&self, resource: &ResourceId) -> Amount {
        self.contents.get(resource).copied().unwrap_or(Amount(0))
    }

    pub fn total_amount(&self) -> Amount {
        Amount(self.contents.values().map(|a| a.0).sum())
    }

    pub fn add_amount(&mut self, resource: ResourceId, amount: Amount) -> Result<(), ModelError> {
        if amount.0 == 0 {
            return Ok(());
        }

        if let Some(cap) = self.capacity {
            let current_total = self.total_amount();
            if current_total.0 + amount.0 > cap.0 {
                return Err(ModelError::CapacityExceeded {
                    resource,
                    attempted: amount.0,
                    available: cap.0 - current_total.0,
                });
            }
        }

        let current = self.contents.entry(resource).or_insert(Amount(0));
        current.0 += amount.0;
        Ok(())
    }

    pub fn subtract_amount(
        &mut self,
        resource: &ResourceId,
        amount: Amount,
    ) -> Result<(), ModelError> {
        if amount.0 == 0 {
            return Ok(());
        }

        let current = self.get_amount(resource);
        if current.0 < amount.0 {
            return Err(ModelError::InsufficientResources {
                resource: resource.clone(),
                requested: amount.0,
                available: current.0,
            });
        }

        if let Some(val) = self.contents.get_mut(resource) {
            val.0 -= amount.0;
            if val.0 == 0 {
                self.contents.remove(resource);
            }
        }
        Ok(())
    }
}
