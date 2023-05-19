use alloc::{vec, vec::Vec};

use petgraph_core::{
    edge::Direction,
    visit::{GetAdjacencyMatrix, GraphProp, IntoNeighborsDirected, NodeCompactIndexable},
};

#[derive(Debug)]
// TODO: make mapping generic over the index type of the other graph.
pub struct Vf2State<'a, G: GetAdjacencyMatrix> {
    /// A reference to the graph this state was built from.
    pub graph: &'a G,
    /// The current mapping M(s) of nodes from G0 → G1 and G1 → G0,
    /// `usize::MAX` for no mapping.
    pub mapping: Vec<usize>,
    /// out[i] is non-zero if i is in either M_0(s) or Tout_0(s)
    /// These are all the next vertices that are not mapped yet, but
    /// have an outgoing edge from the mapping.
    out: Vec<usize>,
    /// ins[i] is non-zero if i is in either M_0(s) or Tin_0(s)
    /// These are all the incoming vertices, those not mapped yet, but
    /// have an edge from them into the mapping.
    /// Unused if graph is undirected -- it's identical with out in that case.
    ins: Vec<usize>,
    pub out_size: usize,
    pub ins_size: usize,
    pub adjacency_matrix: G::AdjMatrix,
    generation: usize,
}

impl<'a, G> Vf2State<'a, G>
where
    G: GetAdjacencyMatrix + GraphProp + NodeCompactIndexable + IntoNeighborsDirected,
{
    pub fn new(g: &'a G) -> Self {
        let c0 = g.node_count();
        Vf2State {
            graph: g,
            mapping: vec![usize::MAX; c0],
            out: vec![0; c0],
            ins: vec![0; c0 * (g.is_directed() as usize)],
            out_size: 0,
            ins_size: 0,
            adjacency_matrix: g.adjacency_matrix(),
            generation: 0,
        }
    }

    /// Return **true** if we have a complete mapping
    pub fn is_complete(&self) -> bool {
        self.generation == self.mapping.len()
    }

    /// Add mapping **from** <-> **to** to the state.
    pub fn push_mapping(&mut self, from: G::NodeId, to: usize) {
        self.generation += 1;
        self.mapping[self.graph.to_index(from)] = to;
        // update T0 & T1 ins/outs
        // T0out: Node in G0 not in M0 but successor of a node in M0.
        // st.out[0]: Node either in M0 or successor of M0
        for ix in self.graph.neighbors_directed(from, Direction::Outgoing) {
            if self.out[self.graph.to_index(ix)] == 0 {
                self.out[self.graph.to_index(ix)] = self.generation;
                self.out_size += 1;
            }
        }
        if self.graph.is_directed() {
            for ix in self.graph.neighbors_directed(from, Direction::Incoming) {
                if self.ins[self.graph.to_index(ix)] == 0 {
                    self.ins[self.graph.to_index(ix)] = self.generation;
                    self.ins_size += 1;
                }
            }
        }
    }

    /// Restore the state to before the last added mapping
    pub fn pop_mapping(&mut self, from: G::NodeId) {
        // undo (n, m) mapping
        self.mapping[self.graph.to_index(from)] = usize::MAX;

        // unmark in ins and outs
        for ix in self.graph.neighbors_directed(from, Direction::Outgoing) {
            if self.out[self.graph.to_index(ix)] == self.generation {
                self.out[self.graph.to_index(ix)] = 0;
                self.out_size -= 1;
            }
        }
        if self.graph.is_directed() {
            for ix in self.graph.neighbors_directed(from, Direction::Incoming) {
                if self.ins[self.graph.to_index(ix)] == self.generation {
                    self.ins[self.graph.to_index(ix)] = 0;
                    self.ins_size -= 1;
                }
            }
        }

        self.generation -= 1;
    }

    /// Find the next (least) node in the Tout set.
    pub fn next_out_index(&self, from_index: usize) -> Option<usize> {
        self.out[from_index..]
            .iter()
            .enumerate()
            .find(move |&(index, &elt)| elt > 0 && self.mapping[from_index + index] == usize::MAX)
            .map(|(index, _)| index)
    }

    /// Find the next (least) node in the Tin set.
    pub fn next_in_index(&self, from_index: usize) -> Option<usize> {
        if !self.graph.is_directed() {
            return None;
        }
        self.ins[from_index..]
            .iter()
            .enumerate()
            .find(move |&(index, &elt)| elt > 0 && self.mapping[from_index + index] == usize::MAX)
            .map(|(index, _)| index)
    }

    /// Find the next (least) node in the N - M set.
    pub fn next_rest_index(&self, from_index: usize) -> Option<usize> {
        self.mapping[from_index..]
            .iter()
            .enumerate()
            .find(|&(_, &elt)| elt == usize::MAX)
            .map(|(index, _)| index)
    }
}
