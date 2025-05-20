extern crate quickcheck;

use core::ops::Range;

use alloc::{boxed::Box, vec::Vec};

use self::quickcheck::{Arbitrary, Gen};
use crate::{
    graph::{node_index, IndexType},
    visit::NodeIndexable,
    EdgeType, Graph,
};

#[cfg(feature = "stable_graph")]
use crate::stable_graph::StableGraph;

#[cfg(feature = "graphmap")]
use crate::graphmap::{GraphMap, NodeTrait};

/// `Arbitrary` for `Graph` creates a graph by selecting a node count
/// and a probability for each possible edge to exist.
///
/// The result will be simple graph or digraph, self loops
/// possible, no parallel edges.
///
/// The exact properties of the produced graph is subject to change.
///
/// Requires crate feature `"quickcheck"`
impl<N, E, Ty, Ix> Arbitrary for Graph<N, E, Ty, Ix>
where
    N: Arbitrary,
    E: Arbitrary,
    Ty: EdgeType + Send + 'static,
    Ix: IndexType + Send,
{
    fn arbitrary(g: &mut Gen) -> Self {
        let nodes = gen_range(g, 0..g.size());
        if nodes == 0 {
            return Graph::with_capacity(0, 0);
        }
        // use X² for edge probability (bias towards lower)
        let edge_prob = gen_float(g, 1.) * gen_float(g, 1.);
        let edges = ((nodes as f64).powi(2) * edge_prob) as usize;
        let mut gr = Graph::with_capacity(nodes, edges);
        for _ in 0..nodes {
            gr.add_node(N::arbitrary(g));
        }
        for i in gr.node_indices() {
            for j in gr.node_indices() {
                if !gr.is_directed() && i > j {
                    continue;
                }
                let p: f64 = gen_float(g, 1.);
                if p <= edge_prob {
                    gr.add_edge(i, j, E::arbitrary(g));
                }
            }
        }
        gr
    }

    // shrink the graph by splitting it in two by a very
    // simple algorithm, just even and odd node indices
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let self_ = self.clone();
        Box::new((0..2).filter_map(move |x| {
            let gr = self_.filter_map(
                |i, w| {
                    if i.index() % 2 == x {
                        Some(w.clone())
                    } else {
                        None
                    }
                },
                |_, w| Some(w.clone()),
            );
            // make sure we shrink
            if gr.node_count() < self_.node_count() {
                Some(gr)
            } else {
                None
            }
        }))
    }
}

#[cfg(feature = "stable_graph")]
/// `Arbitrary` for `StableGraph` creates a graph by selecting a node count
/// and a probability for each possible edge to exist.
///
/// The result will be simple graph or digraph, with possible
/// self loops, no parallel edges.
///
/// The exact properties of the produced graph is subject to change.
///
/// Requires crate features `"quickcheck"` and `"stable_graph"`
impl<N, E, Ty, Ix> Arbitrary for StableGraph<N, E, Ty, Ix>
where
    N: Arbitrary,
    E: Arbitrary,
    Ty: EdgeType + Send + 'static,
    Ix: IndexType + Send,
{
    fn arbitrary(g: &mut Gen) -> Self {
        let nodes = gen_range(g, 0..g.size());
        if nodes == 0 {
            return StableGraph::with_capacity(0, 0);
        }
        // use X² for edge probability (bias towards lower)
        let edge_prob = gen_float(g, 1.) * gen_float(g, 1.);
        let edges = ((nodes as f64).powi(2) * edge_prob) as usize;
        let mut gr = StableGraph::with_capacity(nodes, edges);
        for _ in 0..nodes {
            gr.add_node(N::arbitrary(g));
        }
        for i in 0..gr.node_count() {
            for j in 0..gr.node_count() {
                let i = node_index(i);
                let j = node_index(j);
                if !gr.is_directed() && i > j {
                    continue;
                }
                let p: f64 = gen_float(g, 1.);
                if p <= edge_prob {
                    gr.add_edge(i, j, E::arbitrary(g));
                }
            }
        }
        if bool::arbitrary(g) {
            // potentially remove nodes to make holes in nodes & edge sets
            let n = u8::arbitrary(g) % (gr.node_count() as u8);
            for _ in 0..n {
                let ni = node_index(usize::arbitrary(g) % gr.node_bound());
                if gr.node_weight(ni).is_some() {
                    gr.remove_node(ni);
                }
            }
        }
        gr
    }

    // shrink the graph by splitting it in two by a very
    // simple algorithm, just even and odd node indices
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let self_ = self.clone();
        Box::new((0..2).filter_map(move |x| {
            let gr = self_.filter_map(
                |i, w| {
                    if i.index() % 2 == x {
                        Some(w.clone())
                    } else {
                        None
                    }
                },
                |_, w| Some(w.clone()),
            );
            // make sure we shrink
            if gr.node_count() < self_.node_count() {
                Some(gr)
            } else {
                None
            }
        }))
    }
}

/// `Arbitrary` for `GraphMap` creates a graph by selecting a node count
/// and a probability for each possible edge to exist.
///
/// The result will be simple graph or digraph, self loops
/// possible, no parallel edges.
///
/// The exact properties of the produced graph is subject to change.
///
/// Requires crate features `"quickcheck"` and `"graphmap"`
#[cfg(feature = "graphmap")]
impl<N, E, Ty> Arbitrary for GraphMap<N, E, Ty>
where
    N: NodeTrait + Arbitrary,
    E: Arbitrary,
    Ty: EdgeType + Clone + Send + 'static,
{
    fn arbitrary(g: &mut Gen) -> Self {
        let nodes = gen_range(g, 0..g.size());
        if nodes == 0 {
            return GraphMap::with_capacity(0, 0);
        }
        let mut nodes = (0..nodes).map(|_| N::arbitrary(g)).collect::<Vec<_>>();
        nodes.sort();
        nodes.dedup();

        // use X² for edge probability (bias towards lower)
        let edge_prob = gen_float(g, 1.) * gen_float(g, 1.);
        let edges = ((nodes.len() as f64).powi(2) * edge_prob) as usize;
        let mut gr = GraphMap::with_capacity(nodes.len(), edges);
        for &node in &nodes {
            gr.add_node(node);
        }
        for (index, &i) in nodes.iter().enumerate() {
            let js = if Ty::is_directed() {
                &nodes[..]
            } else {
                &nodes[index..]
            };
            for &j in js {
                let p: f64 = gen_float(g, 1.);
                if p <= edge_prob {
                    gr.add_edge(i, j, E::arbitrary(g));
                }
            }
        }
        gr
    }
}

/// Generate a random float in the range [0., max).
fn gen_float(g: &mut Gen, max: f64) -> f64 {
    // from rand
    let bits = 53;
    let scale = 1. / ((1u64 << bits) as f64);
    let x = u64::arbitrary(g);
    let normalized = (x >> (64 - bits)) as f64 * scale;
    normalized * max
}

/// Generate a random `usize` in the given range.
///
/// See <https://github.com/BurntSushi/quickcheck/issues/267>
fn gen_range(g: &mut Gen, range: Range<usize>) -> usize {
    let span = range.end - range.start;
    let bits = span.next_power_of_two().trailing_zeros();
    let mask = (1 << bits) - 1;
    let mut x = u64::arbitrary(g);
    x &= mask;
    range.start + (x as usize % span)
}
