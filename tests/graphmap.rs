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
