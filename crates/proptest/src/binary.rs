use alloc::{boxed::Box, sync::Arc};
use core::fmt::Debug;

use petgraph_core::{
    data::{Build, Create},
    visit::Data,
};
use proptest::{
    arbitrary::{any, Arbitrary},
    strategy::{BoxedStrategy, LazyJust, Strategy, TupleUnion, Union},
};

use crate::{vtable, vtable::VTable};

#[derive(Debug)]
struct Leaf<N> {
    weight: N,
}

#[derive(Debug)]
struct Edge<N, E> {
    weight: E,
    target: Box<TreeNode<N, E>>,
}

#[derive(Debug)]
struct Node<N, E> {
    weight: N,

    left: Edge<N, E>,
    right: Edge<N, E>,
}

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
            Self::Node(Node {
                left,
                right,
                weight,
            }) => {
                let node = (vtable.add_node)(graph, weight);

                // we generate a tree and we start with the top node (which then references the left
                // and right leaf nodes), this means that if we want to generate a tournament graph,
                // we need to add the edges in reverse order
                let Edge { weight, target } = left;
                let target = target.create(graph, vtable);
                (vtable.add_edge)(graph, target, node, weight);

                let Edge { weight, target } = right;
                let target = target.create(graph, vtable);
                (vtable.add_edge)(graph, target, node, weight);

                node
            }
            Self::Leaf(Leaf { weight }) => (vtable.add_node)(graph, weight),
        }
    }
}

pub fn graph_binary_strategy<G>(
    max_depth: Option<u32>,
    max_nodes: Option<u32>,
) -> impl Strategy<Value = G>
where
    G: Create + Build + Data + Debug,
    G::NodeWeight: Arbitrary + Clone + Debug + 'static,
    G::EdgeWeight: Arbitrary + Debug + 'static,
{
    let depth = max_depth.unwrap_or(32);
    let nodes = max_nodes.unwrap_or(256);

    any::<G::NodeWeight>()
        .prop_map(|weight| TreeNode::<G::NodeWeight, G::EdgeWeight>::Leaf(Leaf { weight }))
        .prop_recursive(
            depth,
            nodes,
            2,
            move |element: BoxedStrategy<TreeNode<G::NodeWeight, G::EdgeWeight>>| {
                // either generate a lead of a node, this enables us to create a diverse set of
                // tournament graphs, where we have a new leaf in the middle of the tree.
                let node = (
                    element.clone(),
                    element.clone(),
                    any::<G::EdgeWeight>(),
                    any::<G::EdgeWeight>(),
                    any::<G::NodeWeight>(),
                )
                    .prop_map(
                        |(left, right, left_weight, right_weight, weight)| {
                            TreeNode::Node(Node {
                                weight,
                                left: Edge {
                                    weight: left_weight,
                                    target: Box::new(left),
                                },
                                right: Edge {
                                    weight: right_weight,
                                    target: Box::new(right),
                                },
                            })
                        },
                    );

                let leaf = element;

                TupleUnion::new((
                    (1u32, Arc::new(node)), //
                    (1u32, Arc::new(leaf)),
                ))
            },
        )
        .prop_map(|node| {
            let mut graph = G::default();
            let vtable = vtable::create::<G>();

            node.create(&mut graph, vtable);

            graph
        })
}
