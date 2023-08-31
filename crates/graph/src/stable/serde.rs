use alloc::vec::Vec;
use core::{cell::Cell, marker::PhantomData};

use funty::Integral;
use petgraph_core::deprecated::{
    index::IndexType,
    visit::{EdgeIndexable, NodeIndexable},
};
use serde::{
    de::Error as _,
    ser::{SerializeSeq, SerializeStruct},
    Deserialize, Deserializer, Serialize, Serializer,
};

use crate::{
    serde::{EdgeProperty, InvalidError},
    stable::StableGraph,
    Edge, EdgeIndex, EdgeType, Graph, Node, NodeIndex,
};

struct SerializeIter<T> {
    len: usize,
    iter: Cell<Option<T>>,
}

impl<T> SerializeIter<T> {
    const fn new(len: usize, iter: T) -> Self {
        Self {
            len,
            iter: Cell::new(Some(iter)),
        }
    }
}

impl<T> Serialize for SerializeIter<T>
where
    T: Iterator,
    T::Item: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let iter = self.iter.take().expect("SerializeIter already serialized");

        let mut seq = serializer.serialize_seq(Some(self.len))?;
        let mut count = 0;
        for item in iter {
            seq.serialize_element(&item)?;
            count += 1;
        }

        debug_assert_eq!(count, self.len, "SerializeIter length mismatch");
        seq.end()
    }
}

impl<N, E, Ty, Ix> Serialize for StableGraph<N, E, Ty, Ix>
where
    N: Serialize,
    E: Serialize,
    Ty: EdgeType,
    Ix: Serialize + IndexType,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let nodes = &self.raw_nodes()[..self.node_bound()];
        let node_count = self.node_count();
        let hole_count = nodes.len() - node_count;
        let edges = &self.raw_edges()[..self.edge_bound()];

        let mut struct_ = serializer.serialize_struct("StableGraph", 4)?;

        struct_.serialize_field(
            "nodes",
            &SerializeIter::new(
                node_count,
                nodes.iter().filter_map(|node| node.weight.as_ref()),
            ),
        )?;
        struct_.serialize_field(
            "node_holes",
            &SerializeIter::new(
                hole_count,
                nodes.iter().enumerate().filter_map(|(index, node)| {
                    node.weight.is_none().then(|| NodeIndex::<Ix>::new(index))
                }),
            ),
        )?;
        struct_.serialize_field("edge_property", &EdgeProperty::from_type::<Ty>())?;

        // Convert from `Edge<Option<E, Ix>>` to `Option<Edge<E, Ix>>`
        let edges_len = edges.len();
        struct_.serialize_field(
            "edges",
            &SerializeIter::new(
                edges_len,
                edges.iter().map(|edge| {
                    edge.weight.as_ref().map(|weight| Edge {
                        weight,
                        next: edge.next,
                        node: edge.node,
                    })
                }),
            ),
        )?;

        struct_.end()
    }
}

#[derive(Deserialize)]
#[serde(rename = "Graph")]
struct RemoteStableGraph<N, E, Ix: IndexType> {
    nodes: Vec<Node<N, Ix>>,
    #[serde(default)]
    node_holes: Vec<NodeIndex<Ix>>,
    edge_property: EdgeProperty,
    edges: Vec<Option<Edge<E, Ix>>>,
}

impl<'de, N, E, Ty, Ix> Deserialize<'de> for StableGraph<N, E, Ty, Ix>
where
    N: Deserialize<'de>,
    E: Deserialize<'de>,
    Ty: EdgeType,
    Ix: Deserialize<'de> + IndexType,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let RemoteStableGraph {
            nodes,
            node_holes,
            edge_property,
            edges,
        } = RemoteStableGraph::<N, E, Ix>::deserialize(deserializer)?;

        // convert from `Option<Edge<E, Ix>>` to `Edge<Option<E>, Ix>`
        let edges = edges
            .into_iter()
            .map(|edge| match edge {
                Some(Edge { weight, next, node }) => Edge {
                    weight: Some(weight),
                    next,
                    node,
                },
                None => Edge {
                    weight: None,
                    next: [EdgeIndex::end(); 2],
                    node: [NodeIndex::end(); 2],
                },
            })
            .collect::<Vec<_>>();

        if edges.len() >= <Ix as Integral>::MAX.as_usize() {
            return Err(InvalidError::Length {
                type_: "edge",
                length: edges.len(),
                max: Ix::MAX.as_usize(),
            })
            .map_err(D::Error::custom);
        }

        let expected = EdgeProperty::from_type::<Ty>();
        if edge_property != expected {
            return Err(InvalidError::Direction {
                expected,
                received: edge_property,
            })
            .map_err(D::Error::custom);
        }

        let total_nodes = nodes.len() + node_holes.len();
        let mut compact_nodes = nodes.into_iter().map(|node| Node {
            weight: Some(node.weight),
            next: node.next,
        });

        let mut nodes = Vec::with_capacity(total_nodes);

        let mut node_pos = 0;
        for position in &node_holes {
            let position = position.index();

            if !(node_pos..total_nodes).contains(&position) {
                return Err(InvalidError::Hole { index: position }).map_err(D::Error::custom);
            }

            nodes.extend(compact_nodes.by_ref().take(position - node_pos));
            nodes.push(Node {
                weight: None,
                next: [EdgeIndex::end(); 2],
            });

            node_pos = position + 1;
            debug_assert_eq!(nodes.len(), node_pos);
        }

        nodes.extend(compact_nodes);

        if nodes.len() >= <Ix as Integral>::MAX.as_usize() {
            return Err(InvalidError::Length {
                type_: "node",
                length: nodes.len(),
                max: Ix::MAX.as_usize(),
            })
            .map_err(D::Error::custom);
        }

        let node_bound = nodes.len();
        let mut stable_graph = Self {
            g: Graph {
                nodes,
                edges,
                ty: PhantomData,
            },
            node_count: 0,
            edge_count: 0,
            free_edge: EdgeIndex::end(),
            free_node: NodeIndex::end(),
        };

        stable_graph
            .link_edges()
            .map_err(|i| InvalidError::Node {
                index: i.index(),
                length: node_bound,
            })
            .map_err(D::Error::custom)?;

        Ok(stable_graph)
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use serde::Deserialize;

    use crate::{
        node_index,
        serde::EdgeProperty,
        stable::{serde::RemoteStableGraph, StableUnGraph},
        EdgeIndex, Node,
    };

    #[test]
    fn deserialization_with_holes() {
        let mut graph = StableUnGraph::<i32, ()>::with_capacity(0, 0);

        let a = graph.add_node(0);
        let b = graph.add_node(1);
        let c = graph.add_node(2);
        let d = graph.add_node(3);
        let e = graph.add_node(4);
        let f = graph.add_node(5);
        let g = graph.add_node(6);

        graph.remove_node(a);
        graph.remove_node(c);
        graph.remove_node(d);
        // as optimization technique, we use `node_bound()` for raw_nodes() and raw_edges(), this
        // means that if we have a trailing hole it will be skipped.
        graph.remove_node(g);

        let value = serde_value::to_value(&graph).unwrap();
        let graph = StableUnGraph::<i32, ()>::deserialize(value).unwrap();

        assert_eq!(graph.node_count(), 3);
        assert_eq!(
            graph
                .raw_nodes()
                .iter()
                .map(|n| n.weight)
                .collect::<Vec<_>>(),
            vec![None, Some(1), None, None, Some(4), Some(5)],
        );
    }
}
