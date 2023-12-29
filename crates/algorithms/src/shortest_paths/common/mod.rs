//! Common utility types, traits and functions for shortest paths algorithms.
mod closures;
pub(super) mod connections;
pub(super) mod cost;
pub(super) mod path;
pub(super) mod queue;
pub(super) mod route;
#[cfg(test)]
pub(super) mod tests;
pub(super) mod transit;
