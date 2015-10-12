
extern crate petgraph;

use petgraph::dag::{Dag, WouldCycle};
use std::iter::once;

struct Weight;


#[test]
fn add_edges_ok() {

    let mut dag = Dag::<Weight, u32, u32>::new();
    let root = dag.add_node(Weight);
    let a = dag.add_node(Weight);
    let b = dag.add_node(Weight);
    let c = dag.add_node(Weight);

    let mut new_edges = dag.add_edges(once((root, a, 0))
        .chain(once((root, b, 1)))
        .chain(once((root, c, 2))))
        .unwrap();

    assert_eq!(new_edges.next(), dag.find_edge(root, a));
    assert_eq!(new_edges.next(), dag.find_edge(root, b));
    assert_eq!(new_edges.next(), dag.find_edge(root, c));

}


#[test]
fn add_edges_err() {

    let mut dag = Dag::<Weight, u32, u32>::new();
    let root = dag.add_node(Weight);
    let a = dag.add_node(Weight);
    let b = dag.add_node(Weight);
    let c = dag.add_node(Weight);

    let add_edges_result = dag.add_edges(once((root, a, 0))
        .chain(once((root, b, 1)))
        .chain(once((root, c, 2)))
        .chain(once((c, root, 3))));

    match add_edges_result {
        Err(WouldCycle(returned_weights)) => assert_eq!(returned_weights, vec![3, 2, 1, 0]),
        Ok(_) => panic!("Should have been an error"),
    }
}


#[test]
fn iter_children() {

    let mut dag = Dag::<Weight, Weight, u32>::new();
    let parent = dag.add_node(Weight);
    let (_, a) = dag.add_child(parent, Weight, Weight);
    let (_, b) = dag.add_child(parent, Weight, Weight);
    let (_, c) = dag.add_child(parent, Weight, Weight);

    {
        let mut children = dag.children(parent);
        assert_eq!(Some(c), children.next());
        assert_eq!(Some(b), children.next());
        assert_eq!(Some(a), children.next());
        assert_eq!(None, children.next());
    }

    let (_, d) = dag.add_child(b, Weight, Weight);
    let (_, e) = dag.add_child(b, Weight, Weight);
    let (_, f) = dag.add_child(b, Weight, Weight);
    {
        let mut children = dag.children(b);
        assert_eq!(Some(f), children.next());
        assert_eq!(Some(e), children.next());
        assert_eq!(Some(d), children.next());
        assert_eq!(None, children.next());
    }
}

#[test]
fn iter_parents() {

    let mut dag = Dag::<Weight, Weight, u32>::new();
    let child = dag.add_node(Weight);
    let (_, a) = dag.add_parent(child, Weight, Weight);
    let (_, b) = dag.add_parent(child, Weight, Weight);
    let (_, c) = dag.add_parent(child, Weight, Weight);
    let (_, d) = dag.add_parent(child, Weight, Weight);

    {
        let mut parents = dag.parents(child);
        assert_eq!(Some(d), parents.next());
        assert_eq!(Some(c), parents.next());
        assert_eq!(Some(b), parents.next());
        assert_eq!(Some(a), parents.next());
        assert_eq!(None, parents.next());
    }
}


#[test]
fn walk_children() {

    let mut dag = Dag::<Weight, Weight, u32>::new();
    let parent = dag.add_node(Weight);
    let (a_e, a_n) = dag.add_child(parent, Weight, Weight);
    let (b_e, b_n) = dag.add_child(parent, Weight, Weight);
    let (c_e, c_n) = dag.add_child(parent, Weight, Weight);

    let mut child_walker = dag.walk_children(parent);
    assert_eq!(Some((c_e, c_n)), child_walker.next_child(&dag));
    assert_eq!(Some((b_e, b_n)), child_walker.next_child(&dag));
    assert_eq!(Some((a_e, a_n)), child_walker.next_child(&dag));
    assert_eq!(None, child_walker.next(&dag));

    let (d_e, d_n) = dag.add_child(b_n, Weight, Weight);
    let (e_e, e_n) = dag.add_child(b_n, Weight, Weight);
    let (f_e, f_n) = dag.add_child(b_n, Weight, Weight);

    child_walker = dag.walk_children(b_n);
    assert_eq!(Some((f_e, f_n)), child_walker.next_child(&dag));
    assert_eq!(Some((e_e, e_n)), child_walker.next_child(&dag));
    assert_eq!(Some((d_e, d_n)), child_walker.next_child(&dag));
    assert_eq!(None, child_walker.next_child(&dag));
}

#[test]
fn walk_parents() {

    let mut dag = Dag::<Weight, Weight, u32>::new();
    let child = dag.add_node(Weight);
    let (a_e, a_n) = dag.add_parent(child, Weight, Weight);
    let (b_e, b_n) = dag.add_parent(child, Weight, Weight);
    let (c_e, c_n) = dag.add_parent(child, Weight, Weight);
    let (d_e, d_n) = dag.add_parent(child, Weight, Weight);

    let mut parent_walker = dag.walk_parents(child);
    assert_eq!(Some((d_e, d_n)), parent_walker.next_parent(&dag));
    assert_eq!(Some((c_e, c_n)), parent_walker.next_parent(&dag));
    assert_eq!(Some((b_e, b_n)), parent_walker.next_parent(&dag));
    assert_eq!(Some((a_e, a_n)), parent_walker.next_parent(&dag));
    assert_eq!(None, parent_walker.next_parent(&dag));
}

