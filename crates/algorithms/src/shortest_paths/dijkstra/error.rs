use core::fmt::{Display, Formatter};

use error_stack::Context;

#[derive(Debug)]
pub enum DijkstraError {
    NodeNotFound,
}

impl Display for DijkstraError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NodeNotFound => write!(f, "node not found"),
        }
    }
}

impl Context for DijkstraError {}
