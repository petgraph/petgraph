use core::{
    fmt,
    fmt::{Debug, Display, Formatter},
};

use error_stack::Context;
use petgraph_core::{base::MaybeOwned, GraphStorage};

pub enum FloydWarshallError<'graph, S>
where
    S: GraphStorage,
{
    NegativeCycle {
        including: Option<MaybeOwned<'graph, S::NodeId>>,
    },
}

impl<S> Debug for FloydWarshallError<S>
where
    S: GraphStorage,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::NegativeCycle { including } => f
                .debug_struct("NegativeCycle")
                .field("including", including)
                .finish(),
        }
    }
}

impl<S> Display for FloydWarshallError<S>
where
    S: GraphStorage,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::NegativeCycle { including } => {
                write!(f, "negative cycle detected, including node {including}")
            }
        }
    }
}

impl<S> Context for FloydWarshallError<S> where S: GraphStorage {}
