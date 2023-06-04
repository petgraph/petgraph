use alloc::vec::Vec;
use core::{fmt::Debug, ops::Range};

use petgraph_core::{
    data::{Build, Create},
    visit::Data,
};
use proptest::{
    arbitrary::{any, Arbitrary},
    collection::vec,
    prelude::{BoxedStrategy, Strategy},
    sample::SizeRange,
};

use crate::{vtable, vtable::VTable};

#[derive(Debug)]
enum TreeNode<N, E> {
    Node(Node<N, E>),
    Leaf(Leaf<N>),
}

impl<N, E> TreeNode<N, E> {
    fn create<G, NodeIndex>(self, graph: &mut G, vtable: VTable<G, N, NodeIndex, E>) -> NodeIndex
    where
        G: Debug,
        NodeIndex: Copy,
    {
        match self {
            Self::Node(Node { weight, children }) => {
                let node = (vtable.add_node)(graph, weight);

                for Edge { weight, target } in children {
                    let target = target.create(graph, vtable);

                    (vtable.add_edge)(graph, node, target, weight);
                }

                node
            }
            Self::Leaf(Leaf { weight }) => (vtable.add_node)(graph, weight),
        }
    }
}

#[derive(Debug)]
struct Edge<N, E> {
    weight: E,
    target: TreeNode<N, E>,
}

#[derive(Debug)]
struct Node<N, E> {
    weight: N,

    children: Vec<Edge<N, E>>,
}

#[derive(Debug)]
struct Leaf<T> {
    weight: T,
}

/// Create a new acyclic graph.
///
/// If `max_depth` is `None`, the graph will be created with a depth of 8.
/// If `max_nodes` is `None`, the graph will be created with a maximum of 256 nodes.
/// If `width` is `None`, the graph will be created with a maximum of 16 children per node.
pub fn graph_dag_strategy<G>(
    max_depth: Option<u32>,
    max_nodes: Option<u32>,
    width: Option<Range<u32>>,
) -> impl Strategy<Value = G>
where
    G: Create + Build + Data + Debug,
    G::NodeWeight: Arbitrary + Clone + Debug + 'static,
    G::EdgeWeight: Arbitrary + Debug + 'static,
{
    let depth = max_depth.unwrap_or(8);
    let nodes = max_nodes.unwrap_or(256);
    let width = width.unwrap_or(0..16);

    any::<G::NodeWeight>()
        .prop_map(|weight| TreeNode::<G::NodeWeight, G::EdgeWeight>::Leaf(Leaf { weight }))
        .prop_recursive(
            depth,     // no more than 8 levels deep (default)
            nodes,     // target around 256 nodes (default)
            width.end, // each collection is up to 16 element long (default)
            move |element: BoxedStrategy<TreeNode<G::NodeWeight, G::EdgeWeight>>| {
                let width = (width.start as usize)..(width.end as usize);

                (
                    any::<G::NodeWeight>(),
                    vec((any::<G::EdgeWeight>(), element), width),
                )
                    .prop_map(|(weight, edges)| {
                        if edges.is_empty() {
                            TreeNode::Leaf(Leaf { weight })
                        } else {
                            TreeNode::Node(Node {
                                weight,
                                children: edges
                                    .into_iter()
                                    .map(|(weight, target)| Edge { weight, target })
                                    .collect(),
                            })
                        }
                    })
            },
        )
        .prop_map(|element: TreeNode<G::NodeWeight, G::EdgeWeight>| {
            let vtable = vtable::create::<G>();
            let mut graph = (vtable.with_capacity)(0, 0);

            element.create(&mut graph, vtable);

            graph
        })
}
