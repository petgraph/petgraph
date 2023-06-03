#![cfg(feature = "graphmap")]
extern crate petgraph;

use std::{collections::HashSet, fmt};

use petgraph::{
    algo::dijkstra,
    dot::{Config, Dot},
    prelude::*,
};
use petgraph_core::visit::Walker;

// TODO: algo
#[test]
fn dfs() {
    let mut gr = UnGraphMap::default();
    let h = gr.add_node("H");
    let i = gr.add_node("I");
    let j = gr.add_node("J");
    let k = gr.add_node("K");
    // Z is disconnected.
    let z = gr.add_node("Z");
    gr.add_edge(h, i, 1.);
    gr.add_edge(h, j, 3.);
    gr.add_edge(i, j, 1.);
    gr.add_edge(i, k, 2.);

    println!("{:?}", gr);

    {
        let mut cnt = 0;
        let mut dfs = Dfs::new(&gr, h);
        while let Some(_) = dfs.next(&gr) {
            cnt += 1;
        }
        assert_eq!(cnt, 4);
    }
    {
        let mut cnt = 0;
        let mut dfs = Dfs::new(&gr, z);
        while let Some(_) = dfs.next(&gr) {
            cnt += 1;
        }
        assert_eq!(cnt, 1);
    }

    assert_eq!(Dfs::new(&gr, h).iter(&gr).count(), 4);
    assert_eq!(Dfs::new(&gr, i).iter(&gr).count(), 4);
    assert_eq!(Dfs::new(&gr, z).iter(&gr).count(), 1);
}

fn assert_sccs_eq<N>(mut res: Vec<Vec<N>>, mut answer: Vec<Vec<N>>)
where
    N: Ord + fmt::Debug,
{
    // normalize the result and compare with the answer.
    for scc in &mut res {
        scc.sort();
    }
    res.sort();
    for scc in &mut answer {
        scc.sort();
    }
    answer.sort();
    assert_eq!(res, answer);
}

// TODO: algo
#[test]
fn scc() {
    let gr: GraphMap<_, u32, Directed> = GraphMap::from_edges(&[
        (6, 0, 0),
        (0, 3, 1),
        (3, 6, 2),
        (8, 6, 3),
        (8, 2, 4),
        (2, 5, 5),
        (5, 8, 6),
        (7, 5, 7),
        (1, 7, 8),
        (7, 4, 9),
        (4, 1, 10),
    ]);

    assert_sccs_eq(petgraph::algo::kosaraju_scc(&gr), vec![
        vec![0, 3, 6],
        vec![1, 4, 7],
        vec![2, 5, 8],
    ]);
}
