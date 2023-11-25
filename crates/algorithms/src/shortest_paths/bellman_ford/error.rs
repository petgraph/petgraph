use core::fmt::{Display, Formatter};

use error_stack::Context;

#[derive(Debug)]
pub enum ShortestPathFasterError {
    NodeNotFound,
    NegativeCycle,
}

impl Display for ShortestPathFasterError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NodeNotFound => write!(f, "node not found"),
            Self::NegativeCycle => write!(f, "negative cycle"),
        }
    }
}

impl Context for ShortestPathFasterError {}
