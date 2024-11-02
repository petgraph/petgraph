use std::{
    collections::{HashMap, HashSet, VecDeque},
    hash::Hash,
    vec::Vec,
};

use crate::visit::{IntoNeighborsDirected, IntoNodeIdentifiers, NodeCount, NodeIndexable};


/// \[Generic\] An algorithm for determining the cycle basis of a graph.
///
/// A set of basis for cycles of a graph is a minimal collection of cycles such that
/// any cycle in the graph can be derived as a sum of the cycles in the cycle basis.
/// 
/// Note that graphs may have multiple, correct cycle basis (see the example below). 
/// 
/// If no root is selected, then the first node from the 'node_identifiers' iterator is used
/// as the initial root.
/// 
/// This algorithm works for disconnected graphs.
///
/// Returns a `Vec` of 'Vec', each containing a cycle. Returns None if no cycles are present.
/// # Example
/// ```rust
/// use petgraph::prelude::*;
/// use petgraph::algo::cycle_basis;
/// use petgraph::{Graph, Undirected};
/// use petgraph::visit::NodeIndexable;
///
/// let mut graph: Graph<(), u16, Undirected> = Graph::from_edges(&[
/// (0,1),(1,2),(2,3),(3,0),(0,2),]);
/// 
/// // 0 ------ 1
/// // |  \     |
/// // |   \    |
/// // |    \   |
/// // |     \  |
/// // |      \ |
/// // 3 <----- 2
///
/// let expected_res: Vec<Vec<NodeIndex>> = vec![
///     vec![0,1,2,3].into_iter().map(NodeIndex::new).collect(),
///     vec![0,2,3].into_iter().map(NodeIndex::new).collect(),
///     ];
/// let mut res: Vec<Vec<NodeIndex>> = cycle_basis(&graph, Some(3.into())).unwrap();
/// res.sort();
/// assert_eq!(res, expected_res);
/// 
/// // Note that the cycle [0,1,2] is equal to the cycle [0,1,2,3] minus [0,2,3].
/// // Also note that [0,1,2] and [0,3,2] is an equally correct cycle basis,
/// // as [0,1,2,3] = [0,1,2] plus [0,3,2] (the edge between 0-2 cancels out).
/// // Which set is returned will depend on the choice of initial root node.
/// ```
pub fn cycle_basis<G>(
    g: G, 
    root_choice: Option<G::NodeId>,
) -> Option<Vec<Vec<G::NodeId>>>
where
    G: IntoNeighborsDirected + IntoNodeIdentifiers + NodeCount + NodeIndexable,
    G::NodeId: Eq + Hash + Copy,
{
    let g_node_count: usize = g.node_count();
    if g_node_count == 0 {
        return None  //Handle the trivial case of an empty graph
    }
    let mut processed_node_ix: HashSet<usize> = HashSet::with_capacity(g_node_count);
    let mut cycles: Vec<Vec<G::NodeId>> = Vec::new();

    let node_vec: VecDeque<G::NodeId> = match root_choice {
        Some(n) => {
            let mut v: VecDeque<G::NodeId> = g.node_identifiers().collect();
            let p = v.iter().position(|&x| x==n);
            v.swap(0,p?);
            v
        },
        None => {
            let w: VecDeque<G::NodeId> = g.node_identifiers().collect();
            w
        }
    };
    let mut node_iter = node_vec.iter();
    let ix = |i| g.to_index(i);
    let deix = |i| g.from_index(i);

    while let Some(root) = node_iter.next() {
        let rooti = ix(*root);
        if processed_node_ix.contains(&rooti) {
            continue
        }
        let mut stack: Vec<usize> = vec![rooti];
        let mut pred: HashMap<usize, usize> = HashMap::from([(rooti, rooti)]);
        let mut used:HashMap<usize, HashSet<usize>> = HashMap::from([(rooti, HashSet::new())]);
        loop {
            let z = match stack.pop() {
                None => break,
                Some(q) => q
            };
            for nbr in g.neighbors(deix(z)) {
                let nbri = ix(nbr);
                if !used.contains_key(&nbri) {
                    pred.insert(nbri, z);
                    stack.push(nbri);
                    used.insert(nbri, HashSet::from([z,]));
                } 
                else if nbri == z {
                    cycles.push(vec![deix(z)]);
                } 
                else if !((used.get(&z).unwrap()).contains(&nbri)) {
                    let pn: &HashSet<usize> = used.get(&nbri).unwrap();
                    let mut cycle: Vec<G::NodeId> = vec![deix(nbri), deix(z)];
                    let mut p = pred.get(&z).unwrap();
                    loop {
                        cycle.push(deix(*p));
                        p = pred.get(&p).unwrap();
                        if pn.contains(&p) {
                            break
                        }
                    }
                    cycle.push(deix(*p));
                    cycle.dedup(); //As we have an explicit self-loop conditional, this is ok
                    cycles.push(cycle);
                    used.get_mut(&nbri).unwrap().insert(z);
                }
            }
            let mut iter_pred = pred.iter();
            while let Some((key, _value)) = iter_pred.next() {
                processed_node_ix.insert(*key);
            }
        }
        processed_node_ix.insert(rooti);
    }
    if !cycles.is_empty() {
        return Some(cycles)
    }
    None
}