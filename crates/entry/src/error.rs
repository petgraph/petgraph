use core::{fmt, fmt::Display};

use error_stack::Context;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntryError {
    /// An underlying backend error occurred.
    ///
    /// Refer to the underlying attached backend error for more information on what went wrong.
    Backend,
    /// The node you are trying to insert already exists.
    ///
    /// This means that the [`Entry::key`] you are trying to insert already exists in the graph.
    ///
    /// [`Entry::key`]: crate::Entry::key
    NodeAlreadyExists,
    /// The edge you are trying to insert already exists.
    ///
    /// This means that the [`Entry::key`] you are trying to insert already exists in the graph.
    ///
    /// [`Entry::key`]: crate::Entry::key
    EdgeAlreadyExists,
}

impl Display for EntryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Backend => write!(f, "Backend error"),
            Self::NodeAlreadyExists => write!(f, "Node already exists"),
            Self::EdgeAlreadyExists => write!(f, "Edge already exists"),
        }
    }
}

impl Context for EntryError {}
