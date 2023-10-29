use core::fmt::{Display, Formatter};

use error_stack::Context;

#[derive(Debug)]
pub enum AStarError {
    NodeNotFound,
}

impl Display for AStarError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NodeNotFound => write!(f, "node not found"),
        }
    }
}

impl Context for AStarError {}
