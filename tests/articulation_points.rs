use petgraph::{
    algo::articulation_points::articulation_points,
    graph::{NodeIndex, UnGraph},
};

use std::collections::HashSet;

#[test]
fn art_single_node() {
    let mut gr = UnGraph::<&str, ()>::new_undirected();
    let _a = gr.add_node("A");

    let set: HashSet<NodeIndex> = HashSet::new();

    assert_eq!(articulation_points(&gr), set);
}

#[test]
fn art_two_connected_components() {
    let mut gr = UnGraph::<&str, ()>::new_undirected();
    let a = gr.add_node("A");
    let b = gr.add_node("B");
    let c = gr.add_node("C");
    let d = gr.add_node("D");
    let e = gr.add_node("E");
    let f = gr.add_node("F");

    gr.add_edge(a, b, ());
    gr.add_edge(b, c, ());

    gr.add_edge(d, e, ());
    gr.add_edge(e, f, ());

    let articulation_lables: Vec<&str> = articulation_points(&gr)
        .into_iter()
        .map(|node_idx| gr[node_idx])
        .collect();
    println!("{:?}", articulation_lables);

    let set: HashSet<NodeIndex> = [b, e].iter().cloned().collect();
    assert_eq!(articulation_points(&gr), set);
}

#[test]
fn art_linear_chain() {
    let mut gr = UnGraph::<&str, ()>::new_undirected();
    let a = gr.add_node("A");
    let b = gr.add_node("B");
    let c = gr.add_node("C");
    let d = gr.add_node("D");

    gr.add_edge(a, b, ());
    gr.add_edge(b, c, ());
    gr.add_edge(c, d, ());

    let set: HashSet<NodeIndex> = [b, c].iter().cloned().collect();

    assert_eq!(articulation_points(&gr), set);
}

#[test]
fn art_star_graph() {
    let mut gr = UnGraph::<&str, ()>::new_undirected();
    let center = gr.add_node("Center");
    let a = gr.add_node("A");
    let b = gr.add_node("B");
    let c = gr.add_node("C");
    let d = gr.add_node("D");

    gr.add_edge(center, a, ());
    gr.add_edge(center, b, ());
    gr.add_edge(center, c, ());
    gr.add_edge(center, d, ());

    let set: HashSet<NodeIndex> = [center].iter().cloned().collect();

    assert_eq!(articulation_points(&gr), set);
}

#[test]
fn art_clique() {
    let mut gr = UnGraph::<&str, ()>::new_undirected();
    let a = gr.add_node("A");
    let b = gr.add_node("B");
    let c = gr.add_node("C");

    gr.add_edge(a, b, ());
    gr.add_edge(b, c, ());
    gr.add_edge(c, a, ());

    let set: HashSet<NodeIndex> = HashSet::new();

    assert_eq!(articulation_points(&gr), set);
}

#[test]
fn art_simple1() {
    let mut gr = UnGraph::<&str, ()>::new_undirected();
    let a = gr.add_node("A");
    let b = gr.add_node("B");
    let c = gr.add_node("C");
    let d = gr.add_node("D");
    gr.add_edge(a, b, ());
    gr.add_edge(b, c, ());
    gr.add_edge(b, d, ());

    let set: HashSet<NodeIndex> = [b].iter().cloned().collect();

    assert_eq!(articulation_points(&gr), set);
}

#[test]
fn art_disconnected_graph() {
    let mut gr = UnGraph::<&str, ()>::new_undirected();
    let a = gr.add_node("A");
    let b = gr.add_node("B");
    let c = gr.add_node("C");
    let d = gr.add_node("D");
    let e = gr.add_node("E");

    gr.add_edge(a, b, ());
    gr.add_edge(b, c, ());
    gr.add_edge(d, e, ());

    let set: HashSet<NodeIndex> = [b].iter().cloned().collect();

    assert_eq!(articulation_points(&gr), set);
}

#[test]
fn art_3x3_grid() {
    let mut gr = UnGraph::<&str, ()>::new_undirected();
    let a = gr.add_node("A");
    let b = gr.add_node("B");
    let c = gr.add_node("C");
    let d = gr.add_node("D");
    let e = gr.add_node("E");
    let f = gr.add_node("F");
    let g = gr.add_node("G");
    let h = gr.add_node("H");
    let i = gr.add_node("I");

    gr.add_edge(a, b, ());
    gr.add_edge(b, c, ());

    gr.add_edge(a, d, ());
    gr.add_edge(b, e, ());
    gr.add_edge(c, f, ());

    gr.add_edge(d, e, ());
    gr.add_edge(e, f, ());

    gr.add_edge(d, g, ());
    gr.add_edge(e, h, ());
    gr.add_edge(f, i, ());

    gr.add_edge(g, h, ());
    gr.add_edge(h, i, ());

    let set: HashSet<NodeIndex> = HashSet::new();

    assert_eq!(articulation_points(&gr), set);
}

#[test]
fn art_simple2() {
    let mut gr = UnGraph::<&str, ()>::new_undirected();
    let a = gr.add_node("A");
    let b = gr.add_node("B");
    let c = gr.add_node("C");
    let d = gr.add_node("D");
    let e = gr.add_node("E");

    gr.add_edge(a, b, ());
    gr.add_edge(b, c, ());
    gr.add_edge(b, d, ());
    gr.add_edge(d, e, ());

    let set: HashSet<NodeIndex> = [b, d].iter().cloned().collect();

    assert_eq!(articulation_points(&gr), set);
}
