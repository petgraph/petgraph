use alloc::vec::Vec;
use core::hash::{BuildHasher, Hash};

use hashbrown::HashMap;
use petgraph_core::{GraphStorage, Node};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(in crate::shortest_paths) enum Intermediates {
    Discard,
    Record,
}

pub(in crate::shortest_paths) fn reconstruct_intermediates<'a, S, H>(
    previous: &HashMap<&'a S::NodeId, Option<Node<'a, S>>, H>,
    target: &'a S::NodeId,
) -> Vec<Node<'a, S>>
where
    S: GraphStorage,
    S::NodeId: Eq + Hash,
    H: BuildHasher,
{
    let mut current = target;

    let mut path = Vec::new();

    loop {
        let Some(node) = previous[current] else {
            // this case should in theory _never_ happen, as the next statement
            // terminates if the next node is `None` (we're at a source node)
            // we do it this way, so that we don't need to push and then pop immediately.
            break;
        };

        if previous[node.id()].is_none() {
            // we have reached the source node
            break;
        }

        path.push(node);
        current = node.id();
    }

    path.reverse();

    path
}
