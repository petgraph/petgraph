

use serde::de::Error;
use serde::{Serialize, Serializer, Deserialize, Deserializer};

use std::marker::PhantomData;

use prelude::*;

use EdgeType;
use graph::Node;
use graph::{IndexType, Edge};
use stable_graph::StableGraph;
use util::rev;
use serde_utils::MappedSequenceVisitor;
use serde_utils::CollectSeqWithLength;
use serde_utils::{IntoSerializable, FromDeserialized};

use super::super::serialization::{EdgeProperty, invalid_length_err, invalid_node_err};

// Serialization representation for StableGraph
// Keep in sync with deserialization and Graph
#[derive(Serialize)]
#[serde(rename = "Graph")]
#[serde(bound(serialize = "N: Serialize, E: Serialize, Ix: IndexType + Serialize"))]
pub struct SerStableGraph<'a, N: 'a, E: 'a, Ix: 'a + IndexType> {
    #[serde(serialize_with="ser_stable_graph_nodes")]
    nodes: &'a [Node<Option<N>, Ix>],
    #[serde(serialize_with="ser_stable_graph_node_holes")]
    node_holes: &'a [Node<Option<N>, Ix>],
    edge_property: EdgeProperty,
    #[serde(serialize_with="ser_stable_graph_edges")]
    edges: &'a [Edge<Option<E>, Ix>],
}

// Deserialization representation for StableGraph
// Keep in sync with serialization and Graph
#[derive(Deserialize)]
#[serde(rename = "Graph")]
#[serde(bound(deserialize = "N: Deserialize<'de>, E: Deserialize<'de>, Ix: IndexType + Deserialize<'de>"))]
pub struct DeserStableGraph<N, E, Ix> {
    #[serde(deserialize_with="deser_stable_graph_nodes")]
    nodes: Vec<Node<Option<N>, Ix>>,
    #[serde(default="Vec::new")]
    node_holes: Vec<NodeIndex<Ix>>,
    edge_property: EdgeProperty,
    #[serde(deserialize_with="deser_stable_graph_edges")]
    edges: Vec<Edge<Option<E>, Ix>>,
}


fn ser_stable_graph_nodes<S, N, Ix>(nodes: &&[Node<Option<N>, Ix>], serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer,
          N: Serialize,
          Ix: Serialize + IndexType,
{
    serializer.collect_seq_with_length(
        nodes.iter()
             .filter(|node| node.weight.is_some())
             .count(),
        nodes.iter()
             .filter_map(|node| node.weight.as_ref()))
}

fn ser_stable_graph_node_holes<S, N, Ix>(nodes: &&[Node<Option<N>, Ix>], serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer,
          N: Serialize,
          Ix: Serialize + IndexType,
{
    serializer.collect_seq_with_length(
        nodes.iter()
             .filter(|node| node.weight.is_none())
             .count(),
        nodes.iter()
             .enumerate()
             .filter_map(|(i, node)| if node.weight.is_none() { Some(NodeIndex::<Ix>::new(i)) } else { None }))
}

fn ser_stable_graph_edges<S, E, Ix>(edges: &&[Edge<Option<E>, Ix>], serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer,
          E: Serialize,
          Ix: Serialize + IndexType,
{
    serializer.collect_seq_exact(
        edges.iter()
            .map(|edge| edge.weight.as_ref().map(|w| (
               edge.source(),
               edge.target(),
               w
            ))))
}

fn deser_stable_graph_nodes<'de, D, N, Ix>(deserializer: D) -> Result<Vec<Node<Option<N>, Ix>>, D::Error>
    where D: Deserializer<'de>,
          N: Deserialize<'de>,
          Ix: IndexType + Deserialize<'de>,
{
    deserializer.deserialize_seq(MappedSequenceVisitor::new(|n|
        Ok(Node {
            weight: Some(n),
            next: [EdgeIndex::end(); 2],
        })
    ))
}

fn deser_stable_graph_edges<'de, D, N, Ix>(deserializer: D) -> Result<Vec<Edge<Option<N>, Ix>>, D::Error>
    where D: Deserializer<'de>,
          N: Deserialize<'de>,
          Ix: IndexType + Deserialize<'de>,
{
    deserializer.deserialize_seq(MappedSequenceVisitor::<Option<(NodeIndex<Ix>, NodeIndex<Ix>, N)>, _, _>::new(|x|
        if let Some((i, j, w)) = x {
            Ok(Edge {
                weight: Some(w),
                node: [i, j],
                next: [EdgeIndex::end(); 2],
            })
        } else {
            Ok(Edge {
                weight: None,
                node: [NodeIndex::end(); 2],
                next: [EdgeIndex::end(); 2],
            })
        }
    ))
}

impl<'a, N, E, Ty, Ix> IntoSerializable for &'a StableGraph<N, E, Ty, Ix>
    where Ix: IndexType,
          Ty: EdgeType,
{
    type Output = SerStableGraph<'a, N, E, Ix>;
    fn into_serializable(self) -> Self::Output {
        SerStableGraph {
            nodes: self.raw_nodes(),
            node_holes: self.raw_nodes(),
            edges: self.raw_edges(),
            edge_property: if self.is_directed() {
                EdgeProperty::Directed
            } else {
                EdgeProperty::Undirected
            },
        }
    }
}

impl<N, E, Ty, Ix> Serialize for StableGraph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType + Serialize,
          N: Serialize,
          E: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        self.into_serializable().serialize(serializer)
    }
}

impl<'a, N, E, Ty, Ix> FromDeserialized for StableGraph<N, E, Ty, Ix>
    where Ix: IndexType,
          Ty: EdgeType,
{
    type Input = DeserStableGraph<N, E, Ix>;
    fn from_deserialized<E2>(input: Self::Input) -> Result<Self, E2>
        where E2: Error
    {
        let ty = PhantomData::<Ty>::from_deserialized(input.edge_property)?;
        let mut nodes = input.nodes;
        let node_holes = input.node_holes;
        let edges = input.edges;
        if edges.len() >= <Ix as IndexType>::max().index() {
            Err(invalid_length_err::<Ix, _>("edge", edges.len()))?
        }

        // insert Nones for each hole
        let mut offset = node_holes.len();
        for hole_pos in rev(node_holes) {
            offset -= 1;
            nodes.insert(hole_pos.index() + offset, Node {
                weight: None,
                next: [EdgeIndex::end(); 2],
            });
        }

        if nodes.len() >= <Ix as IndexType>::max().index() {
            Err(invalid_length_err::<Ix, _>("node", nodes.len()))?
        }

        let node_bound = nodes.len();
        let mut sgr = StableGraph {
            g: Graph {
                nodes: nodes,
                edges: edges,
                ty: ty,
            },
            node_count: 0,
            edge_count: 0,
            free_edge: EdgeIndex::end(),
            free_node: NodeIndex::end(),
        };
        sgr.link_edges().map_err(|i| invalid_node_err(i.index(), node_bound))?;
        Ok(sgr)
    }
}

impl<'de, N, E, Ty, Ix> Deserialize<'de> for StableGraph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType + Deserialize<'de>,
          N: Deserialize<'de>,
          E: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        Self::from_deserialized(DeserStableGraph::deserialize(deserializer)?)
    }
}
