use core::fmt::{Debug, Display, Formatter};

/// An algorithm error
///
/// A cycle of negative weights was found in the graph.
#[derive(Debug, Copy, Clone)]
pub struct NegativeCycleError;

impl Display for NegativeCycleError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str("Negative cycle detected")
    }
}

#[cfg(all(not(nightly), feature = "std"))]
impl std::error::Error for NegativeCycleError {}

#[cfg(nightly)]
impl core::error::Error for NegativeCycleError {}

/// An algorithm error
///
/// A cycle was found in the graph.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CycleError<N> {
    pub(crate) node: N,
}

impl<N> CycleError<N> {
    /// Return a node id that participates in the cycle
    pub fn node_id(&self) -> N
    where
        N: Copy,
    {
        self.node
    }
}

impl<N> Display for CycleError<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str("Cycle detected")
    }
}

#[cfg(all(not(nightly), feature = "std"))]
impl<N> std::error::Error for CycleError<N> where N: Debug {}

#[cfg(nightly)]
impl<N> core::error::Error for CycleError<N> where N: Debug {}
