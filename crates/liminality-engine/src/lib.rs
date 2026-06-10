//! liminality-engine
//!
//! This crate implements simulation modes and state evaluation for liminality.
//! It depends on liminality-model.
//! It must support debug tick and predictive modes eventually.

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
