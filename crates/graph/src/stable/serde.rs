use alloc::vec::Vec;
use core::{cell::Cell, marker::PhantomData};

use funty::Integral;
use petgraph_core::{
    index::IndexType,
    visit::{EdgeIndexable, NodeIndexable},
};
use serde::{
    de::Error as _,
    ser::{SerializeSeq, SerializeStruct},
    Deserialize, Deserializer, Serialize, Serializer,
};

use crate::{
    serde::{EdgeProperty, Error},
    stable::StableGraph,
    Edge, EdgeIndex, EdgeType, Graph, Node, NodeIndex,
};

struct SerializeIter<T> {
    len: usize,
    iter: Cell<Option<T>>,
}

impl<T> SerializeIter<T> {
    pub fn new(len: usize, iter: T) -> Self {
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

        // convert from `Edge<Option<E>, Ix>` to `Option<Edge<E, Ix>>`
        // in practice is simply wraps it in `Some` if `weight` is `Some`, this makes serialization
        // easier and will result in the same payload.
        let edges_len = edges.len();
        struct_.serialize_field(
            "edges",
            &SerializeIter::new(
                edges_len,
                edges
                    .iter()
                    .filter_map(|edge| edge.weight.is_some().then(|| edge)),
            ),
        )?;

        struct_.end()
    }
}

#[derive(Deserialize)]
#[serde(rename = "Graph")]
struct RemoteStableGraph<N, E, Ix: IndexType> {
    nodes: Vec<Node<Option<N>, Ix>>,
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
            return Err(Error::InvalidLength {
                type_: "edge",
                length: edges.len(),
                max: Ix::MAX.as_usize(),
            })
            .map_err(D::Error::custom);
        }

        let expected = EdgeProperty::from_type::<Ty>();
        if edge_property != expected {
            return Err(Error::InvalidDirection {
                expected,
                received: edge_property,
            })
            .map_err(D::Error::custom);
        }

        let total_nodes = nodes.len() + node_holes.len();
        let mut compact_nodes = nodes.into_iter();

        let mut nodes = Vec::with_capacity(total_nodes);

        let mut node_pos = 0;

        for hole_pos in node_holes.iter() {
            let hole_pos = hole_pos.index();
            if !(node_pos..total_nodes).contains(&hole_pos) {
                return Err(Error::InvalidHole { index: hole_pos }).map_err(D::Error::custom);
            }
            nodes.extend(compact_nodes.by_ref().take(hole_pos - node_pos));
            nodes.push(Node {
                weight: None,
                next: [EdgeIndex::end(); 2],
            });
            node_pos = hole_pos + 1;
            debug_assert_eq!(nodes.len(), node_pos);
        }

        nodes.extend(compact_nodes);

        if nodes.len() >= <Ix as Integral>::MAX.as_usize() {
            return Err(Error::InvalidLength {
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
            .map_err(|i| Error::InvalidNode {
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
        let input = RemoteStableGraph::<_, (), u32> {
            nodes: vec![
                Node {
                    weight: Some(1),
                    next: [EdgeIndex::end(); 2],
                },
                Node {
                    weight: Some(4),
                    next: [EdgeIndex::end(); 2],
                },
                Node {
                    weight: Some(5),
                    next: [EdgeIndex::end(); 2],
                },
            ],
            node_holes: vec![node_index(0), node_index(2), node_index(3), node_index(6)],
            edges: vec![],
            edge_property: EdgeProperty::Undirected,
        };

        let value = serde_value::to_value(&input).unwrap();
        let graph = StableUnGraph::deserialize(value).unwrap();

        assert_eq!(graph.node_count(), 3);
        assert_eq!(
            graph
                .raw_nodes()
                .iter()
                .map(|n| n.weight.as_ref().cloned())
                .collect::<Vec<_>>(),
            vec![None, Some(1), None, None, Some(4), Some(5), None],
        );
    }
}
