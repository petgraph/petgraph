use std::{
    collections::{HashMap, HashSet, VecDeque},
    vec::Vec,
};

use crate::visit::{IntoNeighborsDirected, IntoNodeIdentifiers, NodeCount, NodeIndexable};

pub fn cycle_basis<G>(
    g: G, 
    root_choice_index: Option<usize>,
) -> Option<Vec<Vec<usize>>>
where
    G: IntoNeighborsDirected + IntoNodeIdentifiers + NodeCount + NodeIndexable
{
    let g_node_count: usize = g.node_count();
    if g_node_count == 0 {
        return None  //Handle the trivial case of an empty graph
    }
    let mut processed_nodes: HashSet<usize> = HashSet::with_capacity(g_node_count);
    let mut cycles: Vec<Vec<usize>> = Vec::new();

    let node_vec: VecDeque<G::NodeId> = match root_choice_index {
        Some(n) => {
            let mut v: VecDeque<G::NodeId> = g.node_identifiers().collect();
            v.swap(0,n);
            v
        }
        None => {
            let w: VecDeque<G::NodeId> = g.node_identifiers().collect();
            w
        }
    };
    let mut node_iter = node_vec.iter();

    while let Some(root) = node_iter.next() {
        let rooti = g.to_index(*root);
        if processed_nodes.contains(&rooti) {
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
            for nbr in g.neighbors(g.from_index(z)) {
                let nbri = g.to_index(nbr);
                if !used.contains_key(&nbri) {
                    pred.insert(nbri, z);
                    stack.push(nbri);
                    used.insert(nbri, HashSet::from([z,]));
                } 
                else if nbri == z {
                    cycles.push(vec![z]);
                } 
                else if !((used.get(&z).unwrap()).contains(&nbri)) {
                    let pn: &HashSet<usize> = used.get(&nbri).unwrap();
                    let mut cycle: Vec<usize> = vec![nbri, z];
                    let mut p = pred.get(&z).unwrap();
                    loop {
                        cycle.push(*p);
                        p = pred.get(&p).unwrap();
                        if pn.contains(&p) {
                            break
                        }
                    }
                    cycle.push(*p);
                    cycle.dedup(); //As we have an explicit self-loop conditional, this is ok
                    cycles.push(cycle);
                    used.get_mut(&nbri).unwrap().insert(z);
                }
            }
            let mut iter_pred = pred.iter();
            while let Some((key, _value)) = iter_pred.next() {
                processed_nodes.insert(*key);
            }
        }
        processed_nodes.insert(rooti);
    }
    if !cycles.is_empty() {
        return Some(cycles)
    }
    None
}