use std::cmp::min;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use crate::visit::{EdgeRef, IntoEdges, IntoNodeReferences, NodeIndexable, NodeRef};

pub fn articulation_points<G>(g: G) -> HashSet<G::NodeId>
where
    G: IntoNodeReferences + IntoEdges + NodeIndexable,
    G::NodeWeight: Clone,
    G::EdgeWeight: Clone + PartialOrd,
    G::NodeId: Eq + Hash,
{
    let mut visited = HashSet::with_capacity(g.node_references().size_hint().0);
    let mut parent = HashMap::with_capacity(g.node_references().size_hint().0);
    let mut low = HashMap::with_capacity(g.node_references().size_hint().0);
    let mut disc = HashMap::with_capacity(g.node_references().size_hint().0);
    let mut ap = HashSet::with_capacity(g.node_references().size_hint().0);
    let mut time = 0;

    for node in g.node_references() {
        let node_id = g.to_index(node.id());
        if !visited.contains(&node_id) {
            dfs(
                &g,
                node_id,
                &mut visited,
                &mut parent,
                &mut low,
                &mut disc,
                &mut ap,
                &mut time,
            );
        }
    }

    ap.into_iter().map(|id| g.from_index(id)).collect()
}

fn dfs<G>(
    g: &G,
    u: usize,
    visited: &mut HashSet<usize>,
    parent: &mut HashMap<usize, usize>,
    low: &mut HashMap<usize, usize>,
    disc: &mut HashMap<usize, usize>,
    ap: &mut HashSet<usize>,
    time: &mut usize,
) where
    G: IntoEdges + NodeIndexable,
{
    visited.insert(u);
    *time += 1;
    disc.insert(u, *time);
    low.insert(u, *time);
    let mut children = 0;

    for edge in g.edges(g.from_index(u)) {
        let v = g.to_index(edge.target());
        if !visited.contains(&v) {
            children += 1;
            parent.insert(v, u);
            dfs(g, v, visited, parent, low, disc, ap, time);

            low.insert(u, min(low[&u], low[&v]));

            if parent.contains_key(&u) && low[&v] >= disc[&u] {
                ap.insert(u);
            }
        } else if v != parent.get(&u).cloned().unwrap_or(usize::MAX) {
            low.insert(u, min(low[&u], disc[&v]));
        }
    }

    if parent.get(&u).is_none() && children > 1 {
        ap.insert(u);
    }
}
