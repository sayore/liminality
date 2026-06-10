//! liminality-model
//!
//! This crate contains the pure data model for liminality.
//! It defines spatial/time primitives and base graph/resource structures.
//!
//! It contains no simulation logic, no networking, and no daemon/runtime behavior.

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

pub struct Resource {
    pub id: String,
    pub quantity: u32,
}
