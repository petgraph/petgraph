use core::{
    fmt,
    fmt::{Debug, Display, Formatter},
};

use error_stack::Context;
use petgraph_core::{base::MaybeOwned, GraphStorage};

#[derive(Debug, Copy, Clone)]
pub enum FloydWarshallError {
    NegativeCycle,
}

impl Display for FloydWarshallError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::NegativeCycle => f.write_str("graph contains a negative cycle"),
        }
    }
}

impl Context for FloydWarshallError {}
