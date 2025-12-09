#![no_std]
#![allow(clippy::missing_errors_doc, reason = "bootstrap")]

pub mod edge;
pub mod graph;
pub mod id;
pub mod node;
#[cfg(feature = "utils")]
pub mod utils;

#[must_use]
pub const fn add(left: u64, right: u64) -> u64 {
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
