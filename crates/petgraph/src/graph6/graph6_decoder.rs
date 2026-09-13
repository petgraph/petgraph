//! [graph6 format](https://users.cecs.anu.edu.au/~bdm/data/formats.txt) decoder for undirected graphs.

use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};
use core::fmt;
#[cfg(any(feature = "graphmap", feature = "matrix_graph"))]
use core::hash::BuildHasher;

#[cfg(feature = "graphmap")]
use crate::graphmap::GraphMap;
#[cfg(feature = "matrix_graph")]
use crate::matrix_graph::{MatrixGraph, Nullable};
#[cfg(feature = "stable_graph")]
use crate::stable_graph::{StableGraph, StableUnGraph};
use crate::{Graph, Undirected, csr::Csr, graph::IndexType};

const N: usize = 63;

/// The error type for decoding a graph6 format string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Graph6Error {
    /// The string was empty, so it has no order byte.
    Empty,
    /// The string holds a character outside the graph6 printable range `'?'..='~'`.
    InvalidCharacter(char),
    /// The string starts the `~` long form but stops before the three order bytes are complete.
    TruncatedOrder,
    /// The string starts the 8-byte `~~` order form, which this decoder does not support.
    UnsupportedOrder,
    /// The adjacency matrix carries fewer bits than the declared order needs.
    TruncatedAdjacencyMatrix {
        /// Number of bits the declared order requires.
        expected: usize,
        /// Number of bits the string actually carries.
        found: usize,
    },
}

impl core::error::Error for Graph6Error {}

impl fmt::Display for Graph6Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Graph6Error::Empty => write!(f, "empty graph6 string"),
            Graph6Error::InvalidCharacter(c) => {
                write!(f, "character {c:?} is outside the graph6 range '?'..='~'")
            }
            Graph6Error::TruncatedOrder => write!(f, "truncated graph6 order in the `~` long form"),
            Graph6Error::UnsupportedOrder => {
                write!(
                    f,
                    "graph order beyond 258047 (the `~~` form) is not supported"
                )
            }
            Graph6Error::TruncatedAdjacencyMatrix { expected, found } => write!(
                f,
                "adjacency matrix has {found} bits, the declared order needs {expected}"
            ),
        }
    }
}

/// A graph that can be converted from graph6 format string.
pub trait FromGraph6: Sized {
    fn from_graph6_string(graph6_string: String) -> Result<Self, Graph6Error>;
}

/// Converts a graph6 format string into data can be used to construct an undirected graph.
/// Returns a tuple containing the graph order and its edges.
///
/// # Errors
///
/// Returns [`Graph6Error`] if the string is not a well formed graph6 representation.
pub fn from_graph6_representation<Ix>(
    graph6_representation: String,
) -> Result<(usize, Vec<(Ix, Ix)>), Graph6Error>
where
    Ix: IndexType,
{
    let (order_bytes, adj_matrix_bytes) =
        get_order_bytes_and_adj_matrix_bytes(graph6_representation)?;

    let order_bits = bytes_vector_to_bits_vector(order_bytes);
    let adj_matrix_bits = bytes_vector_to_bits_vector(adj_matrix_bytes);

    let graph_order = get_bits_as_decimal(order_bits);
    let edges = get_edges(graph_order, adj_matrix_bits)?;

    Ok((graph_order, edges))
}

// Converts a graph6 format string into a vector of bytes, converted from ASCII characters,
// split into two parts, the first representing the graph order, and the second its adjacency
// matrix.
fn get_order_bytes_and_adj_matrix_bytes(
    graph6_representation: String,
) -> Result<(Vec<usize>, Vec<usize>), Graph6Error> {
    let mut bytes = Vec::with_capacity(graph6_representation.len());
    for c in graph6_representation.chars() {
        // graph6 only uses the printable characters '?' (63) to '~' (126),
        // which biases every byte into 0..=63.
        match (c as usize).checked_sub(N) {
            Some(byte) if byte <= N => bytes.push(byte),
            _ => return Err(Graph6Error::InvalidCharacter(c)),
        }
    }

    let (&first_byte, rest) = bytes.split_first().ok_or(Graph6Error::Empty)?;

    let mut order_bytes = vec![];
    let mut adj_matrix_bytes = vec![];

    if first_byte == N {
        if rest.first() == Some(&N) {
            return Err(Graph6Error::UnsupportedOrder);
        }
        if rest.len() < 3 {
            return Err(Graph6Error::TruncatedOrder);
        }
        order_bytes.extend_from_slice(&rest[..3]);
        adj_matrix_bytes.extend_from_slice(&rest[3..]);
    } else {
        order_bytes.push(first_byte);
        adj_matrix_bytes.extend_from_slice(rest);
    };

    Ok((order_bytes, adj_matrix_bytes))
}

// Converts a bytes vector into a bits vector.
fn bytes_vector_to_bits_vector(bytes: Vec<usize>) -> Vec<u8> {
    bytes
        .iter()
        .flat_map(|&byte| get_number_as_bits(byte, 6))
        .collect()
}

// Get binary representation of `n` as a vector of bits with `bits_length` length.
fn get_number_as_bits(n: usize, bits_length: usize) -> Vec<u8> {
    let mut bits = Vec::new();
    for i in (0..bits_length).rev() {
        bits.push(((n >> i) & 1) as u8);
    }
    bits
}

// Convert a bits vector into its decimal representation.
fn get_bits_as_decimal(bits: Vec<u8>) -> usize {
    let bits_str = bits
        .iter()
        .map(|bit| bit.to_string())
        .collect::<Vec<String>>()
        .join("");

    usize::from_str_radix(&bits_str, 2).unwrap()
}

// Get graph edges from its order and bits vector representation of its adjacency matrix.
fn get_edges<Ix>(order: usize, adj_matrix_bits: Vec<u8>) -> Result<Vec<(Ix, Ix)>, Graph6Error>
where
    Ix: IndexType,
{
    // The upper diagonal of an `order` by `order` matrix, computed in u64 so that a declared
    // order near the 18-bit maximum cannot overflow a 32-bit usize.
    let expected = (order as u64) * (order.saturating_sub(1) as u64) / 2;
    if (adj_matrix_bits.len() as u64) < expected {
        return Err(Graph6Error::TruncatedAdjacencyMatrix {
            expected: expected as usize,
            found: adj_matrix_bits.len(),
        });
    }

    let mut edges = vec![];

    let mut i = 0;
    for col in 1..order {
        for lin in 0..col {
            let is_adjacent = adj_matrix_bits[i] == 1;

            if is_adjacent {
                edges.push((Ix::new(lin), Ix::new(col)));
            };

            i += 1;
        }
    }

    Ok(edges)
}

impl<Ix: IndexType> FromGraph6 for Graph<(), (), Undirected, Ix> {
    fn from_graph6_string(graph6_string: String) -> Result<Self, Graph6Error> {
        let (order, edges): (usize, Vec<(Ix, Ix)>) = from_graph6_representation(graph6_string)?;

        let mut graph: Graph<(), (), Undirected, Ix> = Graph::with_capacity(order, edges.len());
        for _ in 0..order {
            graph.add_node(());
        }
        graph.extend_with_edges(edges);

        Ok(graph)
    }
}

#[cfg(feature = "stable_graph")]
impl<Ix: IndexType> FromGraph6 for StableGraph<(), (), Undirected, Ix> {
    fn from_graph6_string(graph6_string: String) -> Result<Self, Graph6Error> {
        let (order, edges): (usize, Vec<(Ix, Ix)>) = from_graph6_representation(graph6_string)?;

        let mut graph: StableGraph<(), (), Undirected, Ix> =
            StableUnGraph::with_capacity(order, edges.len());
        for _ in 0..order {
            graph.add_node(());
        }
        graph.extend_with_edges(edges);

        Ok(graph)
    }
}

#[cfg(feature = "graphmap")]
impl<Ix: IndexType, S: BuildHasher + Default> FromGraph6 for GraphMap<Ix, (), Undirected, S> {
    fn from_graph6_string(graph6_string: String) -> Result<Self, Graph6Error> {
        let (order, edges): (usize, Vec<(Ix, Ix)>) = from_graph6_representation(graph6_string)?;

        let mut graph: GraphMap<Ix, (), Undirected, S> =
            GraphMap::with_capacity(order, edges.len());
        for i in 0..order {
            graph.add_node(Ix::new(i));
        }
        for (a, b) in edges {
            graph.add_edge(a, b, ());
        }

        Ok(graph)
    }
}

#[cfg(feature = "matrix_graph")]
impl<Null, Ix, S> FromGraph6 for MatrixGraph<(), (), S, Undirected, Null, Ix>
where
    Null: Nullable<Wrapped = ()>,
    Ix: IndexType,
    S: BuildHasher + Default,
{
    fn from_graph6_string(graph6_string: String) -> Result<Self, Graph6Error> {
        let (order, edges): (usize, Vec<(Ix, Ix)>) = from_graph6_representation(graph6_string)?;

        let mut graph: MatrixGraph<(), (), S, Undirected, Null, Ix> =
            MatrixGraph::with_capacity(order);
        for _ in 0..order {
            graph.add_node(());
        }
        graph.extend_with_edges(edges.iter());

        Ok(graph)
    }
}

impl<Ix: IndexType> FromGraph6 for Csr<(), (), Undirected, Ix> {
    fn from_graph6_string(graph6_string: String) -> Result<Self, Graph6Error> {
        let (order, edges): (usize, Vec<(Ix, Ix)>) = from_graph6_representation(graph6_string)?;

        let mut graph: Csr<(), (), Undirected, Ix> = Csr::new();
        let mut nodes = Vec::new();
        for _ in 0..order {
            let i = graph.add_node(());
            nodes.push(i);
        }
        for (a, b) in edges {
            graph.add_edge(a, b, ());
        }

        Ok(graph)
    }
}
