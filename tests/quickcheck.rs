#![cfg(feature = "quickcheck")]
#[macro_use]
extern crate quickcheck;
extern crate petgraph;
extern crate rand;
#[macro_use]
extern crate defmac;

extern crate itertools;
extern crate odds;

mod utils;

use std::{collections::HashSet, fmt, hash::Hash};

use itertools::{assert_equal, cloned};
use odds::prelude::*;
use petgraph::{
    algo::{
        bellman_ford, condensation, dijkstra, find_negative_cycle, floyd_warshall,
        greedy_feedback_arc_set, greedy_matching, is_cyclic_directed, is_cyclic_undirected,
        is_isomorphic, is_isomorphic_matching, k_shortest_path_length, kosaraju_scc,
        maximum_matching, min_spanning_tree, tarjan_scc, toposort, Matching,
    },
    data::FromElements,
    dot::{Config, Dot},
    graph::{edge_index, node_index, IndexType},
    graphmap::NodeTrait,
    operator::complement,
    prelude::*,
    visit::{
        EdgeFiltered, EdgeRef, IntoEdgeReferences, IntoEdges, IntoNeighbors, IntoNodeIdentifiers,
        IntoNodeReferences, NodeCount, NodeIndexable, Reversed, Topo, VisitMap, Visitable,
    },
    EdgeType,
};
use quickcheck::{Arbitrary, Gen};
use rand::Rng;
use utils::{Small, Tournament};

fn sort_sccs<T: Ord>(v: &mut [Vec<T>]) {
    for scc in &mut *v {
        scc.sort();
    }
    v.sort();
}

quickcheck! {
    fn kosaraju_scc_is_topo_sort(g: Graph<(), ()>) -> bool {
        let tsccs = kosaraju_scc(&g);
        let firsts = tsccs.iter().rev().map(|v| v[0]).collect::<Vec<_>>();
        subset_is_topo_order(&g, &firsts)
    }
}

quickcheck! {
    fn tarjan_scc_is_topo_sort(g: Graph<(), ()>) -> bool {
        let tsccs = tarjan_scc(&g);
        let firsts = tsccs.iter().rev().map(|v| v[0]).collect::<Vec<_>>();
        subset_is_topo_order(&g, &firsts)
    }
}

#[derive(Debug, Clone)]
struct DAG<N: Default + Clone + Send + 'static>(Graph<N, ()>);

impl<N: Default + Clone + Send + 'static> Arbitrary for DAG<N> {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let nodes = usize::arbitrary(g);
        if nodes == 0 {
            return DAG(Graph::with_capacity(0, 0));
        }
        let split = g.gen_range(0., 1.);
        let max_width = f64::sqrt(nodes as f64) as usize;
        let tall = (max_width as f64 * split) as usize;
        let fat = max_width - tall;

        let edge_prob = 1. - (1. - g.gen_range(0., 1.)) * (1. - g.gen_range(0., 1.));
        let edges = ((nodes as f64).powi(2) * edge_prob) as usize;
        let mut gr = Graph::with_capacity(nodes, edges);
        let mut nodes = 0;
        for _ in 0..tall {
            let cur_nodes = g.gen_range(0, fat);
            for _ in 0..cur_nodes {
                gr.add_node(N::default());
            }
            for j in 0..nodes {
                for k in 0..cur_nodes {
                    if g.gen_range(0., 1.) < edge_prob {
                        gr.add_edge(NodeIndex::new(j), NodeIndex::new(k + nodes), ());
                    }
                }
            }
            nodes += cur_nodes;
        }
        DAG(gr)
    }

    // shrink the graph by splitting it in two by a very
    // simple algorithm, just even and odd node indices
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let self_ = self.clone();
        Box::new((0..2).filter_map(move |x| {
            let gr = self_.0.filter_map(
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
            if gr.node_count() < self_.0.node_count() {
                Some(DAG(gr))
            } else {
                None
            }
        }))
    }
}

fn subset_is_topo_order<N>(gr: &Graph<N, (), Directed>, order: &[NodeIndex]) -> bool {
    if gr.node_count() < order.len() {
        println!(
            "Graph (len={}) had less nodes than order (len={})",
            gr.node_count(),
            order.len()
        );
        return false;
    }
    // check all the edges of the graph
    for edge in gr.raw_edges() {
        let a = edge.source();
        let b = edge.target();
        if a == b {
            continue;
        }
        // skip those that are not in the subset
        let ai = match order.find(&a) {
            Some(i) => i,
            None => continue,
        };
        let bi = match order.find(&b) {
            Some(i) => i,
            None => continue,
        };
        if ai >= bi {
            println!("{:?} > {:?} ", a, b);
            return false;
        }
    }
    true
}

// TODO: move to algo ?!?!?
quickcheck! {
    // checks that the distances computed by dijkstra satisfy the triangle
    // inequality.
    fn dijkstra_triangle_ineq(g: Graph<u32, u32>, node: usize) -> bool {
        if g.node_count() == 0 {
            return true;
        }
        let v = node_index(node % g.node_count());
        let distances = dijkstra(&g, v, None, |e| *e.weight());
        for v2 in distances.keys() {
            let dv2 = distances[v2];
            // triangle inequality:
            // d(v,u) <= d(v,v2) + w(v2,u)
            for edge in g.edges(*v2) {
                let u = edge.target();
                let w = edge.weight();
                if distances.contains_key(&u) && distances[&u] > dv2 + w {
                    return false;
                }
            }
        }
        true
    }
}

// TODO: move to algo ?!?!?
quickcheck! {
    // checks that the distances computed by k'th shortest path is always greater or equal compared to their dijkstra computation
    fn k_shortest_path_(g: Graph<u32, u32>, node: usize) -> bool {
        if g.node_count() == 0 {
            return true;
        }
        let v = node_index(node % g.node_count());
        let second_best_distances = k_shortest_path(&g, v, None, 2, |e| *e.weight());
        let dijkstra_distances = dijkstra(&g, v, None, |e| *e.weight());
        for v in second_best_distances.keys() {
            if second_best_distances[&v] < dijkstra_distances[&v] {
                return false;
            }
        }
        true
    }
}

// TODO: move to algo ?!?!?
quickcheck! {
    // checks floyd_warshall against dijkstra results
    fn floyd_warshall_(g: Graph<u32, u32>) -> bool {
        if g.node_count() == 0 {
            return true;
        }

        let fw_res = floyd_warshall(&g, |e| *e.weight()).unwrap();

        for node1 in g.node_identifiers() {
            let dijkstra_res = dijkstra(&g, node1, None, |e| *e.weight());

            for node2 in g.node_identifiers() {
                // if dijkstra found a path then the results must be same
                if let Some(distance) = dijkstra_res.get(&node2) {
                    let floyd_distance = fw_res.get(&(node1, node2)).unwrap();
                    if distance != floyd_distance {
                        return false;
                    }
                } else {
                    // if there are no path between two nodes then floyd_warshall will return maximum value possible
                    if *fw_res.get(&(node1, node2)).unwrap() != u32::MAX {
                        return false;
                    }
                }
            }
         }
        true
    }
}

// TODO: move to algo
quickcheck! {
    // checks that the complement of the complement is the same as the input if the input does not contain self-loops
    fn complement_(g: Graph<u32, u32>, _node: usize) -> bool {
        if g.node_count() == 0 {
            return true;
        }
        for x in g.node_indices() {
            if g.contains_edge(x, x) {
                return true;
            }
        }
        let mut complement_graph: Graph<u32, u32>  = Graph::new();
        let mut result: Graph<u32, u32> = Graph::new();
        complement(&g, &mut complement_graph, 0);
        complement(&complement_graph, &mut result, 0);

        for x in g.node_indices() {
            for y in g.node_indices() {
                if g.contains_edge(x, y) != result.contains_edge(x, y){
                    return false;
                }
            }
        }
        true
    }
}

fn naive_closure_foreach<G, F>(g: G, mut f: F)
where
    G: Visitable + IntoNeighbors + IntoNodeIdentifiers,
    F: FnMut(G::NodeId, G::NodeId),
{
    let mut dfs = Dfs::empty(&g);
    for i in g.node_identifiers() {
        dfs.reset(&g);
        dfs.move_to(i);
        while let Some(nx) = dfs.next(&g) {
            if i != nx {
                f(i, nx);
            }
        }
    }
}

fn naive_closure<G>(g: G) -> Vec<(G::NodeId, G::NodeId)>
where
    G: Visitable + IntoNodeIdentifiers + IntoNeighbors,
{
    let mut res = Vec::new();
    naive_closure_foreach(g, |a, b| res.push((a, b)));
    res
}

fn naive_closure_edgecount<G>(g: G) -> usize
where
    G: Visitable + IntoNodeIdentifiers + IntoNeighbors,
{
    let mut res = 0;
    naive_closure_foreach(g, |_, _| res += 1);
    res
}

quickcheck! {
    fn test_tred(g: DAG<()>) -> bool {
        let acyclic = g.0;
        println!("acyclic graph {:#?}", &acyclic);
        let toposort = toposort(&acyclic, None).unwrap();
        println!("Toposort:");
        for (new, old) in toposort.iter().enumerate() {
            println!("{} -> {}", old.index(), new);
        }
        let (toposorted, revtopo): (petgraph::adj::AdjacencyList<(), usize>, _) =
            petgraph::algo::tred::dag_to_toposorted_adjacency_list(&acyclic, &toposort);
        println!("checking revtopo");
        for (i, ix) in toposort.iter().enumerate() {
            assert_eq!(i, revtopo[ix.index()]);
        }
        println!("toposorted adjacency list: {:#?}", &toposorted);
        let (tred, tclos) = petgraph::algo::tred::dag_transitive_reduction_closure(&toposorted);
        println!("tred: {:#?}", &tred);
        println!("tclos: {:#?}", &tclos);
        if tred.node_count() != tclos.node_count() {
            println!("Different node count");
            return false;
        }
        if acyclic.node_count() != tclos.node_count() {
            println!("Different node count from original graph");
            return false;
        }
        // check the closure
        let mut clos_edges: Vec<(_, _)> = tclos.edge_references().map(|i| (i.source(), i.target())).collect();
        clos_edges.sort();
        let mut tred_closure = naive_closure(&tred);
        tred_closure.sort();
        if tred_closure != clos_edges {
            println!("tclos is not the transitive closure of tred");
            return false
        }
        // check the transitive reduction is a transitive reduction
        for i in tred.edge_references() {
            let filtered = EdgeFiltered::from_fn(&tred, |edge| {
                edge.source() !=i.source() || edge.target() != i.target()
            });
            let new = naive_closure_edgecount(&filtered);
            if new >= clos_edges.len() {
                println!("when removing ({} -> {}) the transitive closure does not shrink",
                         i.source().index(), i.target().index());
                return false
            }
        }
        // check that the transitive reduction is included in the original graph
        for i in tred.edge_references() {
            if acyclic.find_edge(toposort[i.source().index()], toposort[i.target().index()]).is_none() {
                println!("tred is not included in the original graph");
                return false
            }
        }
        println!("ok!");
        true
    }
}
