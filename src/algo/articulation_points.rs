use crate::visit::{EdgeRef, IntoEdges, IntoNodeReferences, NodeIndexable, NodeRef};
use std::cmp::min;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

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
    let mut stack: Vec<(usize, Option<usize>)> = vec![(u, None)];

    while let Some((current_node, maybe_current_child)) = stack.pop() {
        if let Some(current_child) = maybe_current_child {
            low.insert(current_node, min(low[&current_node], low[&current_child]));

            if parent.contains_key(&current_node) && low[&current_child] >= disc[&current_node] {
                ap.insert(current_node);
            }
        } else {
            visited.insert(current_node);
            *time += 1;
            disc.insert(current_node, *time);
            low.insert(current_node, *time);
            let mut children: usize = 0;

            for edge in g.edges(g.from_index(current_node)) {
                let current_child = g.to_index(edge.target());
                if !visited.contains(&current_child) {
                    children += 1;
                    parent.insert(current_child, current_node);
                    stack.push((current_node, Some(current_child)));
                    stack.push((current_child, None));
                } else if current_child != parent.get(&current_node).cloned().unwrap_or(usize::MAX)
                {
                    low.insert(current_node, min(low[&current_node], disc[&current_child]));
                }
            }
            if parent.get(&current_node).is_none() && children > 1 {
                ap.insert(current_node);
            }
        }
    }
}
