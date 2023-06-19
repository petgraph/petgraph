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
