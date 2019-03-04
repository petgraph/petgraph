extern crate petgraph;

use petgraph::Graph;
use petgraph::algo::ordered_list;
use petgraph::algo::topology_sorting::TopologySorting;

#[test]
fn topology_cyclic() {

    let mut g = Graph::new();

    let a = g.add_node("A");
    let b = g.add_node("B");
    let c = g.add_node("C");
    let d = g.add_node("D");
    let e = g.add_node("E");

    let _ = g.add_edge(c, b, 0);
    let _ = g.add_edge(d, a, 0);
    let _ = g.add_edge(d, c, 0);
    let _ = g.add_edge(e, a, 0);
    let _ = g.add_edge(a, d, 0);

    match to_vec(ordered_list(&g), |x| **x) {
        Ok(_) => assert!(false),
        Err(_) => assert!(true),
    }
}

#[test]
fn topology_non_cyclic() {

    let mut g = Graph::new();

    let a = g.add_node("A");
    let b = g.add_node("B");
    let c = g.add_node("C");
    let d = g.add_node("D");
    let e = g.add_node("E");

    let _ = g.add_edge(c, b, 0);
    let _ = g.add_edge(d, a, 0);
    let _ = g.add_edge(d, c, 0);
    let _ = g.add_edge(e, a, 0);

    match to_vec(ordered_list(&g), |x| **x) {
        Ok(result) => assert_eq!(result, vec!["D", "E", "C", "A", "B"]),
        Err(_) => assert!(false),
    }
}

fn to_vec<'a, N, T, F>(result: TopologySorting<N>, f: F) -> Result<Vec<T>, TopologySorting<'a, N>>
    where
        F: FnMut(&&N) -> T
{
    match result {
        TopologySorting::Ordered(list) => Ok(list.iter().map(f).collect()),
        TopologySorting::Cyclic => Err(TopologySorting::Cyclic),
    }
}