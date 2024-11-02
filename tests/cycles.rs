use petgraph::prelude::*;
use petgraph::algo::cycle_basis;
use petgraph::{Graph, Undirected};

#[test]
fn test_cycle_basis() {
    let mut graph: Graph<(), u16, Undirected> = Graph::from_edges(&[
        (0,1),(1,2),(2,3),(3,4),(4,5),(3,0),(5,0),(1,6),(6,7),(7,8),(8,0),(8,9),
    ]);
    let mut cy = cycle_basis(&graph, Some(0.into())).unwrap();
    cy.sort();
    let mut res: Vec<Vec<NodeIndex>> = vec![
        vec![1,2,3,0].into_iter().map(NodeIndex::new).collect(),
        vec![1,6,7,8,0].into_iter().map(NodeIndex::new).collect(),
        vec![5,4,3,0].into_iter().map(NodeIndex::new).collect(),
    ];
    assert_eq!(cy, res);
    //test disconnected subgraph
    graph.extend_with_edges(&[(10,11),(11,12),(12,10)]);
    cy = cycle_basis(&graph, Some(0.into())).unwrap();
    cy.sort();
    res = vec![
        vec![1,2,3,0].into_iter().map(NodeIndex::new).collect(),
        vec![1,6,7,8,0].into_iter().map(NodeIndex::new).collect(),
        vec![5,4,3,0].into_iter().map(NodeIndex::new).collect(),
        vec![11,12,10].into_iter().map(NodeIndex::new).collect(),
    ];
    assert_eq!(cy, res);
    graph.clear();
    graph.extend_with_edges(&[(0,1),(1,2),(2,3),(3,0),(0,2)]);
    cy = cycle_basis(&graph, Some(3.into())).unwrap();
    cy.sort();
    res = vec![
        vec![0,1,2,3].into_iter().map(NodeIndex::new).collect(),
        vec![0,2,3].into_iter().map(NodeIndex::new).collect(),
    ];
    assert_eq!(cy, res);
     // test self loop
    graph.add_edge(0.into(), 0.into(), 0);
    cy = cycle_basis(&graph, Some(0.into())).unwrap();
    cy.sort();
    res = vec![
        vec![0].into_iter().map(NodeIndex::new).collect(),
        vec![2,1,0].into_iter().map(NodeIndex::new).collect(),
        vec![2,3,0].into_iter().map(NodeIndex::new).collect(),
    ];
    assert_eq!(cy, res);
}