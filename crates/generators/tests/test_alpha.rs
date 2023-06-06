use petgraph::algo::is_cyclic_directed;
use petgraph_core::{
    edge::{Directed, Undirected},
    visit::GetAdjacencyMatrix,
};
use petgraph_generators::Generator;
use petgraph_graph::Graph;

#[test]
fn test_generate_undirected() {
    for size in 0..4 {
        let mut generator = Generator::<Undirected>::all(size, true);
        let expected_edges = (size * size - size) / 2 + size;

        let mut n = 0;
        while generator.next_ref().is_some() {
            n += 1;
        }

        let expected = 1 << expected_edges;

        assert_eq!(n, expected);
    }
}

#[test]
fn test_generate_directed() {
    // Number of DAG out of all graphs (all permutations) per node size
    //            0, 1, 2, 3,  4,   5 ..
    let n_dag = &[1, 1, 3, 25, 543, 29281, 3781503];
    for (size, &dags_exp) in (0..4).zip(n_dag) {
        let mut gen = Generator::<Directed>::all(size, true);
        let nedges = size * size;
        let mut n = 0;
        let mut dags = 0;
        while let Some(g) = gen.next_ref() {
            n += 1;
            if !is_cyclic_directed(g) {
                dags += 1;
            }
        }

        /*
        // check that all generated graphs have unique adjacency matrices
        let mut adjmats = graphs.iter().map(Graph::adjacency_matrix).collect::<Vec<_>>();
        adjmats.sort(); adjmats.dedup();
        assert_eq!(adjmats.len(), n);
        */
        assert_eq!(dags_exp, dags);
        assert_eq!(n, 1 << nedges);
    }
}

#[test]
fn test_generate_dag() {
    for size in 1..5 {
        let gen = Generator::directed_acyclic(size);
        let nedges = (size - 1) * size / 2;
        let graphs = gen.collect::<Vec<_>>();

        assert_eq!(graphs.len(), 1 << nedges);

        // check that all generated graphs have unique adjacency matrices
        let mut adjmats = graphs
            .iter()
            .map(Graph::adjacency_matrix)
            .collect::<Vec<_>>();

        adjmats.sort();
        adjmats.dedup();

        assert_eq!(adjmats.len(), graphs.len());

        for graph in &graphs {
            assert!(
                !is_cyclic_directed(graph),
                "Assertion failed: {graph:?} acyclic",
            );
        }
    }
}
