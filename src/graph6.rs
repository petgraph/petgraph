//! Graph6 file format input and output.

use crate::visit::{GetAdjacencyMatrix, IntoNodeIdentifiers};

pub struct Graph6 {
    graph6: String,
}

impl Graph6 {
    pub fn from_graph<G>(graph: G) -> Self
    where
        G: GetAdjacencyMatrix + IntoNodeIdentifiers,
    {
        Graph6 {
            graph6: get_graph6_representation(graph),
        }
    }

    pub fn from_string(str: String) -> Self {
        Graph6 { graph6: str }
    }

    pub fn to_string(self) -> String {
        self.graph6
    }
}

const N: usize = 63;

fn get_graph6_representation<G>(graph: G) -> String
where
    G: GetAdjacencyMatrix + IntoNodeIdentifiers,
{
    let (graph_order, mut diagonal_bits) = get_adj_matrix_upper_diagonal_as_bits(graph);
    let mut order_bits = get_graph_order_as_bits(graph_order);

    let mut graph_as_bits = vec![];
    graph_as_bits.append(&mut order_bits);
    graph_as_bits.append(&mut diagonal_bits);

    bits_to_ascii(graph_as_bits)
}

fn bits_to_ascii(bits: Vec<usize>) -> String {
    let bits_strs = bits.iter().map(|bit| bit.to_string()).collect::<Vec<_>>();

    let bytes = bits_strs
        .chunks(6)
        .map(|bits_chunk| bits_chunk.join(""))
        .map(|bits_str| usize::from_str_radix(&bits_str, 2));

    bytes
        .map(|byte| char::from((N + byte.unwrap()) as u8))
        .collect()
}

// Traverse graph nodes and build the upper diagonal of its adjacency matrix.
// Returns a tuple containing:
// - `n`: graph order (number of nodes in graph)
// - `bits`: a vector of 0s and 1s encoding the upper diagonal of the graphs adjacency matrix.
//           This will be normalized to have a length divisible by 6.
fn get_adj_matrix_upper_diagonal_as_bits<G>(graph: G) -> (usize, Vec<usize>)
where
    G: GetAdjacencyMatrix + IntoNodeIdentifiers,
{
    let mut node_ids_iter = graph.node_identifiers();
    let mut node_ids_vec = vec![];

    let adj_matrix = graph.adjacency_matrix();
    let mut bits = vec![];
    let mut n = 0;
    while let Some(node_id) = node_ids_iter.next() {
        node_ids_vec.push(node_id);

        for i in 1..=n {
            let is_adjacent = graph.is_adjacent(&adj_matrix, node_ids_vec[i - 1], node_ids_vec[n]);
            bits.push(if is_adjacent { 1 } else { 0 });
        }

        n += 1;
    }

    while bits.len() % 6 != 0 {
        bits.push(0);
    }

    return (n, bits);
}

fn get_graph_order_as_bits(order: usize) -> Vec<usize> {
    let to_convert_to_bits = if order < N {
        vec![(order, 6)]
    } else if order <= 258047 {
        vec![(N, 6), (order, 18)]
    } else if order <= 68719476735 {
        vec![(N, 6), (N, 6), (order, 36)]
    } else {
        panic!("Graph order not supported.")
    };

    to_convert_to_bits
        .iter()
        .flat_map(|&(n, n_of_bits)| get_number_as_bits(n, n_of_bits))
        .collect()
}

fn get_number_as_bits(n: usize, n_of_bits: usize) -> Vec<usize> {
    let mut bits = Vec::new();
    for i in (0..n_of_bits).rev() {
        bits.push((n >> i) & 1);
    }
    bits
}
