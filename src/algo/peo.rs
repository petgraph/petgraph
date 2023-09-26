use crate::algo::IntoNeighbors;
use crate::algo::IntoNodeIdentifiers;
use crate::algo::NodeIndexable;
use crate::algo::Visitable;
use crate::visit::VisitMap;
use std::collections::HashSet;
use std::collections::LinkedList;
use std::hash::Hash;

/// try to find a peo
/// The input graph is treated as if undirected.
/// if graph is chordal, return Some(_), else return None
pub fn peo<G>(graph: G) -> Option<Vec<G::NodeId>>
where
    G: Visitable + NodeIndexable + IntoNodeIdentifiers + IntoNeighbors,
    G::NodeId: Hash + Eq,
{
    let lbfs_order = lbfs(graph);
    let peo: Vec<G::NodeId> = lbfs_order.into_iter().rev().collect();
    if check_peo(graph, peo.clone()) {
        Some(peo)
    } else {
        None
    }
}

pub fn is_chordal<G>(graph: G) -> bool
where
    G: Visitable + NodeIndexable + IntoNodeIdentifiers + IntoNeighbors,
    G::NodeId: Hash + Eq,
{
    peo(graph).is_some()
}

pub fn check_peo<G>(graph: G, peo: Vec<G::NodeId>) -> bool
where
    G: Visitable + NodeIndexable + IntoNodeIdentifiers + IntoNeighbors,
    G::NodeId: Hash + Eq,
{
    let mut eliminated = graph.visit_map();
    for node in peo {
        if is_clique(
            graph,
            graph
                .neighbors(node)
                .filter(|x| !eliminated.is_visited(x))
                .collect(),
        ) {
            let true = eliminated.visit(node) else {
                unreachable!()
            };
        } else {
            return false;
        }
    }
    true
}

/// The input graph is treated as if undirected.
fn is_clique<G>(graph: G, nodes: HashSet<G::NodeId>) -> bool
where
    G: Visitable + NodeIndexable + IntoNodeIdentifiers + IntoNeighbors,
    G::NodeId: Hash + Eq,
{
    for a in &nodes {
        let mut y = nodes.clone();
        y.remove(a);
        if !y.is_subset(&graph.neighbors(*a).collect()) {
            return false;
        }
    }
    true
}

/// https://en.wikipedia.org/wiki/Lexicographic_breadth-first_search#CITEREFRoseTarjanLueker1976
/// graph is treated as undirected graph
/// use a random start node
pub fn lbfs<G>(graph: G) -> Vec<G::NodeId>
where
    G: Visitable + NodeIndexable + IntoNodeIdentifiers + IntoNeighbors,
    G::NodeId: Hash + Eq,
{
    let mut res: Vec<G::NodeId> = Vec::new();
    let mut l: LinkedList<Vec<G::NodeId>> = LinkedList::from([graph.node_identifiers().collect()]);

    for _ in 0..graph.node_identifiers().count() {
        let v = l.front_mut().unwrap();

        let pivot = v.pop().unwrap();
        res.push(pivot);

        if v.is_empty() {
            let Some(_) = l.pop_front() else {
                unreachable!()
            };
        }

        let mut cursor = l.cursor_front_mut();
        let neighbour: HashSet<G::NodeId> = graph.neighbors(pivot).collect();

        while let Some(x) = cursor.current() {
            let (a, b): (Vec<_>, Vec<_>) = x.iter().partition(|&y| neighbour.contains(y));
            if a.is_empty() || b.is_empty() {
            } else {
                *x = b;
                cursor.insert_before(a);
            }
            cursor.move_next();
        }
    }
    assert!(l.is_empty());

    res
}
