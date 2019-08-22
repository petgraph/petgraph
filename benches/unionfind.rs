#![feature(test)]

extern crate test;
extern crate petgraph;

use petgraph::prelude::*;
use petgraph::{
    EdgeType,
};
use petgraph::graph::{
    node_index,
};

use petgraph::algo::{connected_components, is_cyclic_undirected, min_spanning_tree};

/// Petersen A and B are isomorphic
///
/// http://www.dharwadker.org/tevet/isomorphism/
const PETERSEN_A: &'static str = "
 0 1 0 0 1 0 1 0 0 0 
 1 0 1 0 0 0 0 1 0 0 
 0 1 0 1 0 0 0 0 1 0 
 0 0 1 0 1 0 0 0 0 1 
 1 0 0 1 0 1 0 0 0 0 
 0 0 0 0 1 0 0 1 1 0 
 1 0 0 0 0 0 0 0 1 1 
 0 1 0 0 0 1 0 0 0 1 
 0 0 1 0 0 1 1 0 0 0 
 0 0 0 1 0 0 1 1 0 0
";

const PETERSEN_B: &'static str = "
 0 0 0 1 0 1 0 0 0 1 
 0 0 0 1 1 0 1 0 0 0 
 0 0 0 0 0 0 1 1 0 1 
 1 1 0 0 0 0 0 1 0 0
 0 1 0 0 0 0 0 0 1 1 
 1 0 0 0 0 0 1 0 1 0 
 0 1 1 0 0 1 0 0 0 0 
 0 0 1 1 0 0 0 0 1 0 
 0 0 0 0 1 1 0 1 0 0 
 1 0 1 0 1 0 0 0 0 0
";

/// An almost full set, isomorphic
const FULL_A: &'static str = "
 1 1 1 1 1 1 1 1 1 1 
 1 1 1 1 1 1 1 1 1 1 
 1 1 1 1 1 1 1 1 1 1 
 1 1 1 1 1 1 1 1 1 1 
 1 1 1 1 1 1 1 1 1 1 
 1 1 1 1 1 1 1 1 1 1 
 1 1 1 1 1 1 1 1 1 1 
 1 1 1 1 1 1 1 1 1 1 
 1 1 1 1 0 1 1 1 0 1 
 1 1 1 1 1 1 1 1 1 1
";

const FULL_B: &'static str = "
 1 1 1 1 1 1 1 1 1 1 
 1 1 1 1 1 1 1 1 1 1 
 1 1 0 1 1 1 0 1 1 1 
 1 1 1 1 1 1 1 1 1 1
 1 1 1 1 1 1 1 1 1 1 
 1 1 1 1 1 1 1 1 1 1 
 1 1 1 1 1 1 1 1 1 1 
 1 1 1 1 1 1 1 1 1 1 
 1 1 1 1 1 1 1 1 1 1 
 1 1 1 1 1 1 1 1 1 1
";

/// Praust A and B are not isomorphic
const PRAUST_A: &'static str = "
 0 1 1 1 1 0 0 0 1 0 0 0 0 0 0 0 0 0 0 0 
 1 0 1 1 0 1 0 0 0 1 0 0 0 0 0 0 0 0 0 0 
 1 1 0 1 0 0 1 0 0 0 1 0 0 0 0 0 0 0 0 0 
 1 1 1 0 0 0 0 1 0 0 0 1 0 0 0 0 0 0 0 0 
 1 0 0 0 0 1 1 1 0 0 0 0 1 0 0 0 0 0 0 0 
 0 1 0 0 1 0 1 1 0 0 0 0 0 1 0 0 0 0 0 0 
 0 0 1 0 1 1 0 1 0 0 0 0 0 0 1 0 0 0 0 0 
 0 0 0 1 1 1 1 0 0 0 0 0 0 0 0 1 0 0 0 0 
 1 0 0 0 0 0 0 0 0 1 1 1 0 0 0 0 1 0 0 0 
 0 1 0 0 0 0 0 0 1 0 1 1 0 0 0 0 0 1 0 0 
 0 0 1 0 0 0 0 0 1 1 0 1 0 0 0 0 0 0 1 0 
 0 0 0 1 0 0 0 0 1 1 1 0 0 0 0 0 0 0 0 1 
 0 0 0 0 1 0 0 0 0 0 0 0 0 1 1 1 0 1 0 0 
 0 0 0 0 0 1 0 0 0 0 0 0 1 0 1 1 1 0 0 0 
 0 0 0 0 0 0 1 0 0 0 0 0 1 1 0 1 0 0 0 1 
 0 0 0 0 0 0 0 1 0 0 0 0 1 1 1 0 0 0 1 0 
 0 0 0 0 0 0 0 0 1 0 0 0 0 1 0 0 0 1 1 1 
 0 0 0 0 0 0 0 0 0 1 0 0 1 0 0 0 1 0 1 1 
 0 0 0 0 0 0 0 0 0 0 1 0 0 0 0 1 1 1 0 1 
 0 0 0 0 0 0 0 0 0 0 0 1 0 0 1 0 1 1 1 0
";

const PRAUST_B: &'static str = "
 0 1 1 1 1 0 0 0 1 0 0 0 0 0 0 0 0 0 0 0 
 1 0 1 1 0 1 0 0 0 1 0 0 0 0 0 0 0 0 0 0 
 1 1 0 1 0 0 1 0 0 0 1 0 0 0 0 0 0 0 0 0 
 1 1 1 0 0 0 0 1 0 0 0 1 0 0 0 0 0 0 0 0 
 1 0 0 0 0 1 1 1 0 0 0 0 1 0 0 0 0 0 0 0 
 0 1 0 0 1 0 1 1 0 0 0 0 0 0 0 0 0 0 0 1 
 0 0 1 0 1 1 0 1 0 0 0 0 0 0 1 0 0 0 0 0 
 0 0 0 1 1 1 1 0 0 0 0 0 0 0 0 0 0 1 0 0 
 1 0 0 0 0 0 0 0 0 1 1 1 0 0 0 0 1 0 0 0
 0 1 0 0 0 0 0 0 1 0 1 1 0 1 0 0 0 0 0 0 
 0 0 1 0 0 0 0 0 1 1 0 1 0 0 0 0 0 0 1 0 
 0 0 0 1 0 0 0 0 1 1 1 0 0 0 0 1 0 0 0 0 
 0 0 0 0 1 0 0 0 0 0 0 0 0 1 1 0 0 1 0 1 
 0 0 0 0 0 0 0 0 0 1 0 0 1 0 0 1 1 0 1 0 
 0 0 0 0 0 0 1 0 0 0 0 0 1 0 0 1 0 1 0 1 
 0 0 0 0 0 0 0 0 0 0 0 1 0 1 1 0 1 0 1 0 
 0 0 0 0 0 0 0 0 1 0 0 0 0 1 0 1 0 1 1 0 
 0 0 0 0 0 0 0 1 0 0 0 0 1 0 1 0 1 0 0 1 
 0 0 0 0 0 0 0 0 0 0 1 0 0 1 0 1 1 0 0 1 
 0 0 0 0 0 1 0 0 0 0 0 0 1 0 1 0 0 1 1 0 
";

/// Parse a text adjacency matrix format into a graph
fn parse_graph<Ty: EdgeType>(s: &str) -> Graph<(), (), Ty> {
    let mut gr = Graph::with_capacity(0, 0);
    let s = s.trim();
    let lines = s.lines().filter(|l| !l.is_empty());
    for (row, line) in lines.enumerate() {
        for (col, word) in line.split(' ')
                                .filter(|s| s.len() > 0)
                                .enumerate()
        {
            let has_edge = word.parse::<i32>().unwrap();
            assert!(has_edge == 0 || has_edge == 1);
            if has_edge == 0 {
                continue;
            }
            while col >= gr.node_count() || row >= gr.node_count() {
                gr.add_node(());
            }
            gr.update_edge(node_index(row), node_index(col), ());
        }
    }
    gr
}

/// Parse a text adjacency matrix format into a *undirected* graph
fn str_to_ungraph(s: &str) -> Graph<(), (), Undirected> {
    parse_graph(s)
}

/// Parse a text adjacency matrix format into a *directed* graph
fn str_to_digraph(s: &str) -> Graph<(), (), Directed> {
    parse_graph(s)
}

#[bench]
fn connected_components_praust_undir_bench(bench: &mut test::Bencher) {
    let a = str_to_ungraph(PRAUST_A);
    let b = str_to_ungraph(PRAUST_B);

    bench.iter(|| {
        (connected_components(&a), connected_components(&b))
    });
}

#[bench]
fn connected_components_praust_dir_bench(bench: &mut test::Bencher) {
    let a = str_to_digraph(PRAUST_A);
    let b = str_to_digraph(PRAUST_B);

    bench.iter(|| {
        (connected_components(&a), connected_components(&b))
    });
}

#[bench]
fn connected_components_full_undir_bench(bench: &mut test::Bencher) {
    let a = str_to_ungraph(FULL_A);
    let b = str_to_ungraph(FULL_B);

    bench.iter(|| {
        (connected_components(&a), connected_components(&b))
    });
}

#[bench]
fn connected_components_full_dir_bench(bench: &mut test::Bencher) {
    let a = str_to_digraph(FULL_A);
    let b = str_to_digraph(FULL_B);

    bench.iter(|| {
        (connected_components(&a), connected_components(&b))
    });
}

#[bench]
fn connected_components_petersen_undir_bench(bench: &mut test::Bencher) {
    let a = str_to_ungraph(PETERSEN_A);
    let b = str_to_ungraph(PETERSEN_B);

    bench.iter(|| {
        (connected_components(&a), connected_components(&b))
    });
}

#[bench]
fn connected_components_petersen_dir_bench(bench: &mut test::Bencher) {
    let a = str_to_digraph(PETERSEN_A);
    let b = str_to_digraph(PETERSEN_B);

    bench.iter(|| {
        (connected_components(&a), connected_components(&b))
    });
}

#[bench]
fn is_cyclic_undirected_praust_undir_bench(bench: &mut test::Bencher) {
    let a = str_to_ungraph(PRAUST_A);
    let b = str_to_ungraph(PRAUST_B);

    bench.iter(|| {
        (is_cyclic_undirected(&a), is_cyclic_undirected(&b))
    });
}

#[bench]
fn is_cyclic_undirected_praust_dir_bench(bench: &mut test::Bencher) {
    let a = str_to_digraph(PRAUST_A);
    let b = str_to_digraph(PRAUST_B);

    bench.iter(|| {
        (is_cyclic_undirected(&a), is_cyclic_undirected(&b))
    });
}

#[bench]
fn is_cyclic_undirected_full_undir_bench(bench: &mut test::Bencher) {
    let a = str_to_ungraph(FULL_A);
    let b = str_to_ungraph(FULL_B);

    bench.iter(|| {
        (is_cyclic_undirected(&a), is_cyclic_undirected(&b))
    });
}

#[bench]
fn is_cyclic_undirected_full_dir_bench(bench: &mut test::Bencher) {
    let a = str_to_digraph(FULL_A);
    let b = str_to_digraph(FULL_B);

    bench.iter(|| {
        (is_cyclic_undirected(&a), is_cyclic_undirected(&b))
    });
}

#[bench]
fn is_cyclic_undirected_petersen_undir_bench(bench: &mut test::Bencher) {
    let a = str_to_ungraph(PETERSEN_A);
    let b = str_to_ungraph(PETERSEN_B);

    bench.iter(|| {
        (is_cyclic_undirected(&a), is_cyclic_undirected(&b))
    });
}

#[bench]
fn is_cyclic_undirected_petersen_dir_bench(bench: &mut test::Bencher) {
    let a = str_to_digraph(PETERSEN_A);
    let b = str_to_digraph(PETERSEN_B);

    bench.iter(|| {
        (is_cyclic_undirected(&a), is_cyclic_undirected(&b))
    });
}

#[bench]
fn min_spanning_tree_praust_undir_bench(bench: &mut test::Bencher) {
    let a = str_to_ungraph(PRAUST_A);
    let b = str_to_ungraph(PRAUST_B);

    bench.iter(|| {
        (min_spanning_tree(&a), min_spanning_tree(&b))
    });
}

#[bench]
fn min_spanning_tree_praust_dir_bench(bench: &mut test::Bencher) {
    let a = str_to_digraph(PRAUST_A);
    let b = str_to_digraph(PRAUST_B);

    bench.iter(|| {
        (min_spanning_tree(&a), min_spanning_tree(&b))
    });
}

#[bench]
fn min_spanning_tree_full_undir_bench(bench: &mut test::Bencher) {
    let a = str_to_ungraph(FULL_A);
    let b = str_to_ungraph(FULL_B);

    bench.iter(|| {
        (min_spanning_tree(&a), min_spanning_tree(&b))
    });
}

#[bench]
fn min_spanning_tree_full_dir_bench(bench: &mut test::Bencher) {
    let a = str_to_digraph(FULL_A);
    let b = str_to_digraph(FULL_B);

    bench.iter(|| {
        (min_spanning_tree(&a), min_spanning_tree(&b))
    });
}

#[bench]
fn min_spanning_tree_petersen_undir_bench(bench: &mut test::Bencher) {
    let a = str_to_ungraph(PETERSEN_A);
    let b = str_to_ungraph(PETERSEN_B);

    bench.iter(|| {
        (min_spanning_tree(&a), min_spanning_tree(&b))
    });
}

#[bench]
fn min_spanning_tree_petersen_dir_bench(bench: &mut test::Bencher) {
    let a = str_to_digraph(PETERSEN_A);
    let b = str_to_digraph(PETERSEN_B);

    bench.iter(|| {
        (min_spanning_tree(&a), min_spanning_tree(&b))
    });
}
