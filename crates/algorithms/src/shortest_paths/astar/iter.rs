use core::{hash::Hash, ops::Add};

use fxhash::FxBuildHasher;
use hashbrown::HashMap;
use num_traits::Zero;
use petgraph_core::{base::MaybeOwned, Edge, GraphStorage, Node};

use crate::shortest_paths::common::queue::Queue;

pub(super) struct AStarIter<'a, S, T, E, H, C>
where
    S: GraphStorage,
    T: Ord,
{
    queue: Queue<'a, S, T>,

    edge_cost: E,
    heuristic: H,
    connections: C,

    source: Node<'a, S>,

    num_nodes: usize,

    distances: HashMap<&'a S::NodeId, T, FxBuildHasher>,
    heuristic_values: HashMap<&'a S::NodeId, T, FxBuildHasher>,
    previous: HashMap<&'a S::NodeId, Option<Node<'a, S>>, FxBuildHasher>,
}

impl<'a, S, T, E, H, C> AStarIter<'a, S, T, E, H, C>
where
    S: GraphStorage,
    S::NodeId: Eq + Hash,
    T: PartialOrd + Ord + Zero + Clone + 'a,
    for<'b> &'b T: Add<Output = T>,
    E: Fn(Edge<'a, S>) -> MaybeOwned<'a, T>,
    H: Fn(Node<'a, S>, Node<'a, S>) -> MaybeOwned<'a, T>,
    C: ConnectionFn
{
}
