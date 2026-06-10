//! liminality-protocol
//!
//! This crate defines serializable DTOs/messages for snapshots, deltas, queries, and responses.
//! It depends on liminality-model.
//! It must not own simulation logic.

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
