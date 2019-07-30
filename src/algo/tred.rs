//! Transitive reduction and closure

use crate::Direction;
use adj::List;
use data::{NodeIndexMap, NodeIndexMappable};
use fixedbitset::FixedBitSet;
use graph::IndexType;
use visit::{GraphBase, IntoNeighbors, IntoNeighborsDirected, NodeCount};

pub fn to_toposorted_adjacency_list<G, Ix: IndexType>(g: G, toposort: &[G::NodeId]) -> List<(), Ix>
where
    G: GraphBase + IntoNeighborsDirected + NodeIndexMappable<Ix> + NodeCount,
{
    let mut res = List::with_capacity(g.node_count());
    let mut revmap = g.new_node_index_map();
    for (ix, &old_ix) in toposort.iter().enumerate() {
        let ix = Ix::new(ix);
        revmap.set(old_ix, ix);
        let mut n = 0;
        for old_pre in g.neighbors_directed(old_ix, Direction::Incoming) {
            let pre: Ix = revmap.get(old_pre).unwrap().clone();
            res.add_edge(pre, ix, ());
            n += 1;
        }
        let new_ix = res.add_node_with_capacity(n);
        debug_assert_eq!(new_ix.index(), ix.index());
    }
    res
}

pub fn transitive_reduction_closure<Ix: IndexType>(
    g: &List<(), Ix>,
) -> (List<(), Ix>, List<(), Ix>) {
    let mut tred = List::with_capacity(g.node_count());
    let mut tclos = List::with_capacity(g.node_count());
    let mut mark = FixedBitSet::with_capacity(g.node_count());
    for i in g.node_indices() {
        tred.add_node();
        tclos.add_node_with_capacity(g.neighbors(i).len());
    }
    for i in g.node_indices().rev() {
        for x in g.neighbors(i) {
            tred.add_edge(i, x, ());
            tclos.add_edge(i, x, ());
            for e in tclos.edge_indices_from(x) {
                let y = tclos.edge_endpoints(e).unwrap().1;
                if !mark[y.index()] {
                    mark.insert(y.index());
                    tclos.add_edge(i, y, ());
                }
            }
        }
        for y in tclos.neighbors(i) {
            mark.set(y.index(), false);
        }
    }
    (tred, tclos)
}
