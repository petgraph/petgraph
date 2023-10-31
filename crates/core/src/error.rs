use core::fmt::{Display, Formatter};

use error_stack::Context;

/// General error type for `petgraph-core`.
///
/// This error is used in [`Graph`] as a context from the returned result of fallible
/// implementations of the underlying [`GraphStorage`] implementation.
///
/// [`Graph`]: crate::graph::Graph
/// [`GraphStorage`]: crate::storage::GraphStorage
#[derive(Debug, Copy, Clone)]
pub struct Error;

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str("Graph Error")
    }
}

#[cfg(not(feature = "std"))]
impl Context for Error {}

#[cfg(feature = "std")]
impl std::error::Error for Error {}
