//! Serde support for `Graph` and `StableGraph`
//!
//! ## Serialization Format
//!
//! The serialization format is as follows, in pseudocode:
//!
//! ```text
//! Graph {
//!     nodes: [N],
//!     node_holes: [NodeIndex<Ix>],
//!     edge_property: EdgeProperty,
//!     edges: [Option<(NodeIndex<Ix>, NodeIndex<Ix>, E)>]
//! }
//! ```
//!
//! Node indices are serialized as integers and are fixed size for binary formats, so the Ix
//! parameter matters there.
//!
//! A stable graph serialization that obeys these restrictions (effectively, it has no interior
//! vacancies) can de deserialized as a [`Graph`].
//!
//! ## [`Graph`] Restrictions
//!
//! `node_holes` is always empty and `edges` are always `Some`.

use alloc::{string::ToString, vec::Vec};
use core::{
    fmt::{Display, Formatter},
    marker::PhantomData,
};

use funty::Integral;
use petgraph_core::{edge::EdgeType, index::IndexType};
use serde::{
    de::Error as _,
    ser::{SerializeStruct, SerializeTuple},
    Deserialize, Deserializer, Serialize, Serializer,
};

use super::{EdgeIndex, NodeIndex};
use crate::{Edge, Graph, Node};

#[derive(Debug)]
enum Error {
    InvalidNode {
        index: usize,
        length: usize,
    },
    InvalidHole {
        index: usize,
    },
    InvalidLength {
        type_: &'static str,
        length: usize,
        max: usize,
    },
    InvalidDirection {
        expected: EdgeProperty,
        received: EdgeProperty,
    },
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidNode { index, length } => f.write_fmt(format_args!(
                "invalid value: node index `{index}` does not exist in graph with node bound \
                 {length}",
            )),
            Self::InvalidHole { index } => f.write_fmt(format_args!(
                "invalid value: node hole `{index}` is not allowed.",
            )),
            Self::InvalidLength { type_, length, max } => f.write_fmt(format_args!(
                "invalid value: {type_} length `{length}` exceeds maximum of `{max}`",
            )),
            Self::InvalidDirection { expected, received } => f.write_fmt(format_args!(
                "invalid value: expected {expected} graph, but received {received} graph",
            )),
        }
    }
}

impl<T> From<Error> for T
where
    T: serde::de::Error,
{
    fn from(value: Error) -> Self {
        T::custom(value.to_string())
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeProperty {
    Undirected,
    Directed,
}

impl Display for EdgeProperty {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            EdgeProperty::Undirected => f.write_str("undirected"),
            EdgeProperty::Directed => f.write_str("directed"),
        }
    }
}

impl EdgeProperty {
    fn from_type<T>() -> Self
    where
        T: EdgeType,
    {
        if T::is_directed() {
            Self::Directed
        } else {
            Self::Undirected
        }
    }
}

impl<Ix> Serialize for NodeIndex<Ix>
where
    Ix: IndexType + Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<Ix> Serialize for EdgeIndex<Ix>
where
    Ix: IndexType + Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<E, Ix> Serialize for Edge<E, Ix>
where
    E: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tuple = serializer.serialize_tuple(3)?;

        tuple.serialize_element(&self.source())?;
        tuple.serialize_element(&self.target())?;
        tuple.serialize_element(&self.weight)?;

        tuple.end()
    }
}

impl<N, Ix> Serialize for Node<N, Ix>
where
    N: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.weight.serialize(serializer)
    }
}

impl<N, E, Ty, Ix> Serialize for Graph<N, E, Ty, Ix>
where
    N: Serialize,
    E: Serialize,
    Ty: EdgeType,
    Ix: IndexType + Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut struct_ = serializer.serialize_struct("Graph", 4)?;

        struct_.serialize_field("nodes", &self.nodes)?;
        struct_.serialize_field("node_holes", &[])?;
        struct_.serialize_field("edge_property", &EdgeProperty::from_type::<Ty>())?;
        struct_.serialize_field("edges", &self.edges)?;

        struct_.end()
    }
}

#[derive(Deserialize)]
#[serde(rename = "Edge")]
struct EdgeRemote<E, Ix: IndexType>(NodeIndex<Ix>, NodeIndex<Ix>, E);

impl<'de, E, Ix> Deserialize<'de> for Edge<E, Ix>
where
    E: Deserialize<'de>,
    Ix: Deserialize<'de> + IndexType,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let EdgeRemote(source, target, weight) = EdgeRemote::deserialize(deserializer)?;

        Ok(Edge {
            weight,
            node: [source, target],
            next: [EdgeIndex::end(); 2],
        })
    }
}

#[derive(Deserialize)]
#[serde(rename = "Node")]
struct NodeRemote<N>(N);

impl<'de, N, Ix> Deserialize<'de> for Node<N, Ix>
where
    N: Deserialize<'de>,
    Ix: IndexType,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let NodeRemote(weight) = NodeRemote::deserialize(deserializer)?;

        Ok(Node {
            weight,
            next: [EdgeIndex::end(); 2],
        })
    }
}

#[derive(Deserialize)]
#[serde(rename = "Graph")]
struct GraphRemote<N, E, Ix: IndexType> {
    nodes: Vec<Node<N, Ix>>,
    // always empty per serialization format
    #[serde(default)]
    node_holes: [(); 0],
    edge_property: EdgeProperty,
    edges: Vec<Edge<E, Ix>>,
}

impl<'de, N, E, Ty, Ix> Deserialize<'de> for Graph<N, E, Ty, Ix>
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
        let GraphRemote {
            nodes,
            node_holes,
            edge_property,
            edges,
        } = GraphRemote::deserialize(deserializer)?;

        if nodes.len() >= <Ix as Integral>::MAX.to_usize() {
            return Err(Error::InvalidLength {
                type_: "node",
                length: nodes.len(),
                max: Ix::MAX.to_usize(),
            }
            .into());
        }

        if edges.len() >= <Ix as Integral>::MAX.to_usize() {
            return Err(Error::InvalidLength {
                type_: "edge",
                length: edges.len(),
                max: Ix::MAX.to_usize(),
            }
            .into());
        }

        let expected = EdgeProperty::from_type::<Ty>();
        if edge_property != expected {
            return Err(Error::InvalidDirection {
                expected,
                received: edge_property,
            });
        }

        let mut this = Graph {
            nodes,
            edges,
            ty: PhantomData,
        };

        let node_count = this.node_count();
        this.link_edges().map_err(|i| Error::InvalidNode {
            index: i.index(),
            length: node_count,
        })?;

        Ok(this)
    }
}
