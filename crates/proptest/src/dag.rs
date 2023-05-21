use alloc::vec::Vec;
use core::fmt::Debug;

use petgraph_core::{
    data::{Build, Create},
    visit::Data,
};
use proptest::{
    arbitrary::{any, Arbitrary},
    collection::vec,
    prelude::{BoxedStrategy, Strategy},
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

pub fn graph_dag_strategy<G>() -> impl Strategy<Value = G>
where
    G: Create + Build + Data + Debug,
    G::NodeWeight: Arbitrary + Clone + Debug + 'static,
    G::EdgeWeight: Arbitrary + Debug + 'static,
{
    any::<G::NodeWeight>()
        .prop_map(|weight| TreeNode::<G::NodeWeight, G::EdgeWeight>::Leaf(Leaf { weight }))
        .prop_recursive(
            8,   // no more than 8 levels deep
            256, // target around 256 nodes
            16,  // each collection is up to 16 element long
            |element: BoxedStrategy<TreeNode<G::NodeWeight, G::EdgeWeight>>| {
                (
                    any::<G::NodeWeight>(),
                    vec((any::<G::EdgeWeight>(), element), 0..16),
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
