use petgraph::algo::cycle_basis;
use petgraph::{Graph, Undirected};
use petgraph::visit::NodeIndexable;

#[test]
fn test_cycle_basis() {
    //Borrowing the tests from Networkx
    let mut graph: Graph<(), u16, Undirected> = Graph::from_edges(&[
        (0,1),(1,2),(2,3),(3,4),(4,5),(3,0),(5,0),(1,6),(6,7),(7,8),(8,0),(8,9),
    ]);
    let mut cy = cycle_basis(&graph, Some(graph.to_index(0.into()))).unwrap();
    cy.sort();
    assert_eq!(cy, vec![vec![1,2,3,0], vec![1,6,7,8,0], vec![5,4,3,0]]);
    //test disconnected subgraph
    graph.extend_with_edges(&[(10,11),(11,12),(12,10)]);
    cy = cycle_basis(&graph, Some(graph.to_index(0.into()))).unwrap();
    cy.sort();
    assert_eq!(cy, vec![vec![1,2,3,0], vec![1,6,7,8,0], vec![5,4,3,0], vec![11,12,10]]);
    graph.clear();
    graph.extend_with_edges(&[(0,1),(1,2),(2,3),(3,0),(0,2)]);
    cy = cycle_basis(&graph, Some(graph.to_index(3.into()))).unwrap();
    cy.sort();
    assert_eq!(cy, vec![vec![0,1,2,3], vec![0,2,3]]);
}