use serde::de::{Error, Visitor, SeqAccess, MapAccess, DeserializeSeed};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use std::marker::PhantomData;
use std::fmt;

use crate::prelude::*;

use crate::graph::Node;
use crate::graph::{Edge, IndexType};
use crate::serde_utils::CollectSeqWithLength;
use crate::serde_utils::MappedSequenceVisitor;
use crate::serde_utils::{FromDeserialized, IntoSerializable};
use crate::stable_graph::StableGraph;
use crate::visit::NodeIndexable;
use crate::EdgeType;

use super::super::serialization::{invalid_length_err, invalid_node_err, EdgeProperty};

// Serialization representation for StableGraph
// Keep in sync with deserialization and Graph
#[derive(Serialize)]
#[serde(rename = "Graph")]
#[serde(bound(serialize = "N: Serialize, E: Serialize, Ix: IndexType + Serialize"))]
pub struct SerStableGraph<'a, N: 'a, E: 'a, Ix: 'a + IndexType> {
    node_holes: Holes<&'a [Node<Option<N>, Ix>]>,
    nodes: Somes<&'a [Node<Option<N>, Ix>]>,
    edge_property: EdgeProperty,
    #[serde(serialize_with = "ser_stable_graph_edges")]
    edges: &'a [Edge<Option<E>, Ix>],
}

static STABLE_GRAPH_FIELDS: &[&str] = &["node_holes", "nodes", "edge_property", "edges"];
// Deserialization representation for StableGraph
// Keep in sync with serialization and Graph
impl<'de, N, E, Ty, Ix> Deserialize<'de> for StableGraph<N, E, Ty, Ix>
    where N: Deserialize<'de>, E: Deserialize<'de>, Ix: IndexType + Deserialize<'de>, Ty: EdgeType
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        deserializer.deserialize_struct("StableGraph", STABLE_GRAPH_FIELDS, StableGraphVisitor::new())
    }
}

#[derive(Deserialize)]
#[serde(field_identifier, rename_all = "snake_case")]
enum SerializedFields {
    NodeHoles,
    Nodes,
    EdgeProperty,
    Edges,
}

enum NodeState<N, Ix> {
    Compact(Vec<N>),
    Expanded(Vec<Node<Option<N>, Ix>>)
}

struct StableGraphVisitor<N, E, Ty, Ix> {
    marker: PhantomData<(N, E, Ty, Ix)>,
}
impl<'de, N, E, Ty, Ix> StableGraphVisitor<N, E, Ty, Ix>
    where N: Deserialize<'de>, E: Deserialize<'de>, Ix: IndexType, Ty: EdgeType
{
    fn new() -> Self {
        StableGraphVisitor { marker: PhantomData }
    }
    fn expand_nodes<Er, I>(node_holes: Vec<NodeIndex<Ix>>, mut compact_nodes: I, compact_nodes_len: Option<usize>)
        -> Result<Vec<Node<Option<N>, Ix>>, Er>
        where I: Iterator<Item=Result<N, Er>>, Er: Error
    {
        let mut nodes = Vec::with_capacity(compact_nodes_len.unwrap_or(0) + node_holes.len());

        let make_node = |n| Node {
            weight: Some(n),
            next: [EdgeIndex::end(); 2],
        };

        let mut num_compact = 0;
        let mut node_pos = 0;
        for hole_pos in node_holes.iter() {
            for _ in 0 .. hole_pos.index() - node_pos {
                nodes.push(make_node(compact_nodes.next().ok_or_else(|| Error::invalid_length(num_compact, &"more nodes"))??));
                num_compact += 1;
            }
            nodes.push(Node {
                weight: None,
                next: [EdgeIndex::end(); 2],
            });
            node_pos = hole_pos.index() + 1;
            debug_assert_eq!(nodes.len(), node_pos);
        }
        for r in compact_nodes {
            nodes.push(make_node(r?));
            num_compact += 1;
        }

        Ok(nodes)
    }

    fn merge_nodes_with_holes<Er: Error>(node_holes: Vec<NodeIndex<Ix>>, compact_nodes: Vec<N>)
        -> Result<Vec<Node<Option<N>, Ix>>, Er>
    {
        let size_hint = Some(compact_nodes.len());
        Self::expand_nodes(node_holes, compact_nodes.into_iter().map(Ok), size_hint)
    }

    fn build<Er: Error>(nodes: Vec<Node<Option<N>, Ix>>, edges: Vec<Edge<Option<E>, Ix>>, edge_property: EdgeProperty)
        -> Result<StableGraph<N, E, Ty, Ix>, Er>
    {
        let ty = PhantomData::<Ty>::from_deserialized(edge_property)?;
        if edges.len() >= <Ix as IndexType>::max().index() {
            Err(invalid_length_err::<Ix, _>("edge", edges.len()))?
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
        sgr.link_edges()
            .map_err(|i| invalid_node_err(i.index(), node_bound))?;
        Ok(sgr)
    }
}

impl<'de, N, E, Ty, Ix> Visitor<'de> for StableGraphVisitor<N, E, Ty, Ix>
    where N: Deserialize<'de>, E: Deserialize<'de>, Ix: IndexType + Deserialize<'de>, Ty: EdgeType
{
    type Value = StableGraph<N, E, Ty, Ix>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "struct StableGraph")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let node_holes = NodeHoles::new(seq.next_element()?.ok_or(A::Error::missing_field("node_holes"))?);
        let nodes = seq.next_element_seed(node_holes)?.ok_or(A::Error::missing_field("node_holes"))?;
        let edge_property = seq.next_element()?.ok_or(A::Error::missing_field("edge_property"))?;
        let edges = seq.next_element::<StableGraphEdges<E, Ix>>()?.ok_or(A::Error::missing_field("edge_property"))?.0;

        StableGraphVisitor::build(nodes, edges, edge_property)
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where A: MapAccess<'de>
    {
        let mut node_holes: Option<Vec<NodeIndex<Ix>>> = None;
        let mut nodes: Option<NodeState<N, Ix>> = None;
        let mut edges: Option<StableGraphEdges<E, Ix>> = None;
        let mut edge_property = None;

        while let Some(key) = map.next_key()? {
            match key {
                SerializedFields::NodeHoles => {
                    if node_holes.is_some() {
                        return Err(Error::duplicate_field("node_holes"));
                    }
                    node_holes = Some(map.next_value()?);
                }
                SerializedFields::Nodes => {
                    if nodes.is_some() {
                        return Err(Error::duplicate_field("nodes"));
                    }
                    nodes = Some(match node_holes.take() {
                        None => NodeState::Compact(map.next_value()?),
                        Some(node_holes) => NodeState::Expanded(map.next_value_seed(NodeHoles::new(node_holes))?),
                    });
                }
                SerializedFields::Edges => {
                    if edges.is_some() {
                        return Err(Error::duplicate_field("edges"));
                    }
                    edges = Some(map.next_value()?);
                }
                SerializedFields::EdgeProperty => {
                    if edge_property.is_some() {
                        return Err(Error::duplicate_field("edge_property"));
                    }
                    edge_property = Some(map.next_value()?);
                }
            }
        }

        let nodes = match (node_holes, nodes) {
            (None, Some(NodeState::Expanded(nodes))) => nodes,
            (Some(node_holes), Some(NodeState::Compact(nodes))) => Self::merge_nodes_with_holes(node_holes, nodes)?,
            (None, _) => return Err(Error::missing_field("node_holes")),
            (Some(_), None) => return Err(Error::missing_field("nodes")),
            (Some(_), Some(_)) => panic!("logic error. this should not happen.")
        };
        let edges = edges.ok_or_else(|| Error::missing_field("edges"))?.0;
        let edge_property = edge_property.ok_or_else(|| Error::missing_field("edge_property"))?;

        StableGraphVisitor::build(nodes, edges, edge_property)
    }
}

struct NodeHoles<N, Ix>(Vec<NodeIndex<Ix>>, PhantomData<N>);
impl<N, Ix> NodeHoles<N, Ix> {
    fn new(node_holes: Vec<NodeIndex<Ix>>) -> Self {
        NodeHoles(node_holes, PhantomData)
    }
}
impl<'de, N, Ix> DeserializeSeed<'de> for NodeHoles<N, Ix>
    where N: Deserialize<'de>, Ix: IndexType + Deserialize<'de>
{
    type Value = Vec<Node<Option<N>, Ix>>;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: Deserializer<'de>
    {
        deserializer.deserialize_seq(CompactNodeVisitor::new(self.0))
    }
}

/// Serde combinator. A sequence visitor that maps deserialized elements
/// lazily; the visitor can also emit new errors if the elements have errors.
pub struct CompactNodeVisitor<N, Ix> {
    node_holes: Vec<NodeIndex<Ix>>,
    marker: PhantomData<N>,
}
impl<'de, N, Ix> CompactNodeVisitor<N, Ix>
    where N: Deserialize<'de>, Ix: IndexType
{
    pub fn new(node_holes: Vec<NodeIndex<Ix>>) -> Self {
        CompactNodeVisitor {
            node_holes,
            marker: PhantomData,
        }
    }
    fn expand_nodes<A>(self, mut nodes_seq: A) -> Result<Vec<Node<Option<N>, Ix>>, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut nodes = Vec::with_capacity(self.node_holes.len() + nodes_seq.size_hint().unwrap_or(0));
        let mut node_pos = 0;
        let mut full_nodes = 0;

        for hole_pos in self.node_holes.iter() {
            for _ in 0 .. hole_pos.index() - node_pos {
                match nodes_seq.next_element()? {
                    Some(n) => {
                        nodes.push(Node {
                            weight: Some(n),
                            next: [EdgeIndex::end(); 2],
                        });
                        full_nodes += 1;
                    }
                    None => return Err(A::Error::invalid_length(full_nodes, &"more nodes"))
                }
            }
            nodes.push(Node {
                weight: None,
                next: [EdgeIndex::end(); 2],
            });
            node_pos = hole_pos.index() + 1;
            debug_assert_eq!(nodes.len(), node_pos);
        }
        while let Some(n) = nodes_seq.next_element()? {
            nodes.push(Node {
                weight: Some(n),
                next: [EdgeIndex::end(); 2],
            });
        }

        Ok(nodes)
    }
}

impl<'de, N, Ix> Visitor<'de> for CompactNodeVisitor<N, Ix>
    where N: Deserialize<'de>, Ix: IndexType + Deserialize<'de>
{
    type Value = Vec<Node<Option<N>, Ix>>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a sequence")
    }

    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        self.expand_nodes(seq)
    }
}

/// `Somes` are the present node weights N, with known length.
struct Somes<T>(usize, T);

impl<'a, N, Ix> Serialize for Somes<&'a [Node<Option<N>, Ix>]>
where
    N: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_seq_with_length(
            self.0,
            self.1.iter().filter_map(|node| node.weight.as_ref()),
        )
    }
}

/// Holes are the node indices of vacancies, with known length
struct Holes<T>(usize, T);

impl<'a, N, Ix> Serialize for Holes<&'a [Node<Option<N>, Ix>]>
where
    Ix: Serialize + IndexType,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_seq_with_length(
            self.0,
            self.1.iter().enumerate().filter_map(|(i, node)| {
                if node.weight.is_none() {
                    Some(NodeIndex::<Ix>::new(i))
                } else {
                    None
                }
            }),
        )
    }
}

fn ser_stable_graph_edges<S, E, Ix>(
    edges: &&[Edge<Option<E>, Ix>],
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    E: Serialize,
    Ix: Serialize + IndexType,
{
    serializer.collect_seq_exact(edges.iter().map(|edge| {
        edge.weight
            .as_ref()
            .map(|w| (edge.source(), edge.target(), w))
    }))
}

struct StableGraphEdges<E, Ix>(Vec<Edge<Option<E>, Ix>>);
impl<'de, E, Ix> Deserialize<'de> for StableGraphEdges<E, Ix>
    where E: Deserialize<'de>, Ix: IndexType + Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where
        D: Deserializer<'de>,
    {
        let edges = deserializer.deserialize_seq(MappedSequenceVisitor::<
            Option<(NodeIndex<Ix>, NodeIndex<Ix>, E)>,
            _,
            _,
        >::new(|x| {
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
        }))?;
        Ok(StableGraphEdges(edges))
    }
}

impl<'a, N, E, Ty, Ix> IntoSerializable for &'a StableGraph<N, E, Ty, Ix>
where
    Ix: IndexType,
    Ty: EdgeType,
{
    type Output = SerStableGraph<'a, N, E, Ix>;
    fn into_serializable(self) -> Self::Output {
        let nodes = &self.raw_nodes()[..self.node_bound()];
        let node_count = self.node_count();
        let hole_count = nodes.len() - node_count;
        let edges = &self.raw_edges()[..self.edge_bound()];
        SerStableGraph {
            nodes: Somes(node_count, nodes),
            node_holes: Holes(hole_count, nodes),
            edges: edges,
            edge_property: EdgeProperty::from(PhantomData::<Ty>),
        }
    }
}

/// Requires crate feature `"serde-1"`
impl<N, E, Ty, Ix> Serialize for StableGraph<N, E, Ty, Ix>
where
    Ty: EdgeType,
    Ix: IndexType + Serialize,
    N: Serialize,
    E: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.into_serializable().serialize(serializer)
    }
}

/*
#[test]
fn test_from_deserialized_with_holes() {
    use crate::graph::node_index;
    use crate::stable_graph::StableUnGraph;
    use itertools::assert_equal;
    use serde::de::value::Error as SerdeError;

    let input = DeserStableGraph::<_, (), u32> {
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
    let graph = StableUnGraph::from_deserialized::<SerdeError>(input).unwrap();

    assert_eq!(graph.node_count(), 3);
    assert_equal(
        graph.raw_nodes().iter().map(|n| n.weight.as_ref().cloned()),
        vec![None, Some(1), None, None, Some(4), Some(5), None],
    );
}
*/
