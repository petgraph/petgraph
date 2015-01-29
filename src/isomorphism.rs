use super::{
    EdgeType,
    Incoming,
    Directed,
};
use super::graph::{
    Graph,
    IndexType,
    NodeIndex,
};

#[derive(Debug)]
struct Vf2State<Ix> {
    /// The current mapping M(s) of nodes from G0 → G1 and G1 → G0,
    /// NodeIndex::end() for no mapping.
    core: Vec<NodeIndex<Ix>>,
    /// ins[i] is non-zero if i is in either M_0(s) or Tin_0(s)
    ins: Vec<usize>,
    /// out[i] is non-zero if i is in either M_0(s) or Tout_0(s)
    out: Vec<usize>,
}

impl<Ix> Vf2State<Ix> where Ix: IndexType
{
    fn new(c0: usize) -> Self
    {
        let mut state = Vf2State {
            core: Vec::with_capacity(c0),
            ins: Vec::with_capacity(c0),
            out: Vec::with_capacity(c0),
        };
        for _ in (0..c0) {
            state.core.push(NodeIndex::end());
            state.ins.push(0);
            state.out.push(0);
        }
        state
    }
}


pub fn is_isomorphic<N, E, Ix>(g0: &Graph<N, E, Directed, Ix>,
                               g1: &Graph<N, E, Directed, Ix>) -> bool where
    Ix: IndexType,
{
    if g0.node_count() != g1.node_count() || g0.edge_count() != g1.edge_count() {
        return false
    }
    fn try_match<N, E, Ix: IndexType>(s: usize,
                                      st: &mut [Vf2State<Ix>; 2],
                                      g0: &Graph<N, E, Directed, Ix>,
                                      g1: &Graph<N, E, Directed, Ix>) -> (bool, bool)
    {
        let g = [g0, g1];
        let graph_indices = 0..2us;
        let end = NodeIndex::end();

        // if all are mapped -- we are done and have an iso
        if s - 1 == g0.node_count() {
            return (true, true);
        }

        // A "depth first" search of a valid mapping from graph 1 to graph 2

        // F(s, n, m) -- evaluate state s and add mapping n <-> m

        // Find least T1out node (in st.out[1] but not in M[1])
        #[derive(Copy, Clone, PartialEq, Debug)]
        enum OpenList {
            Out,
            In,
            Other,
        }
        let mut open_list = OpenList::Out;

        // Find the next (least) node in the Tout or Tin set.
        let next_open_index = |&: st_inout: &[usize], st_core: &[NodeIndex<Ix>]| {
            st_inout.iter()
                    .enumerate()
                    .filter(|&(index, elt)| *elt > 0 && st_core[index] == end)
                    .next()
                    .map(|(index, _)| index)
        };

        // Find the next (least) node in the N - M set.
        let next_rest_index = |&: st_core: &[NodeIndex<Ix>]| {
            st_core.iter()
                   .enumerate()
                   .filter(|&(_, elt)| *elt == end)
                   .next()
                   .map(|(index, _)| index)
        };

        let mut to_index;
        let mut from_index = None;
        // Try the out list
        to_index = next_open_index(&st[1].out[0..], &st[1].core[0..]);

        if to_index.is_some() {
            from_index = next_open_index(&st[0].out[0..], &st[0].core[0..]);
            open_list = OpenList::Out;
        }

        // Try the in list
        if to_index.is_none() || from_index.is_none() {
            to_index = next_open_index(&st[1].ins[0..], &st[1].core[0..]);

            if to_index.is_some() {
                from_index = next_open_index(&st[0].ins[0..], &st[0].core[0..]);
                open_list = OpenList::In;
            }
        }

        // Try the other list -- disconnected graph
        if to_index.is_none() || from_index.is_none() {
            to_index = next_rest_index(&st[1].core[0..]);
            if to_index.is_some() {
                from_index = next_rest_index(&st[0].core[0..]);
                open_list = OpenList::Other;
            }
        }

        let (cand0, cand1) = match (from_index, to_index) {
            (Some(n), Some(m)) => (n, m),
            // No more candidates
            _ => return (false, false)
        };

        let mut nx = NodeIndex::new(cand0);
        let mx = NodeIndex::new(cand1);

        let mut first = true;

        'candidates: loop {
            if !first {
                // Find the next node index to try on the `from` side of the mapping
                let start = nx.index() + 1;
                let cand0 = match open_list {
                    OpenList::Out => next_open_index(&st[0].out[start..], &st[0].core[start..]),
                    OpenList::In => next_open_index(&st[0].ins[start..], &st[0].core[start..]),
                    OpenList::Other => next_rest_index(&st[0].core[start..]),
                }.map(|c| c + start); // compensate for start offset.
                nx = match cand0 {
                    None => break, // no more candidates
                    Some(ix) => NodeIndex::new(ix),
                };
                debug_assert!(nx.index() >= start);
            }
            first = false;

            let nodes = [nx, mx];

            /*
            print!("Mapping state: ");
            for (index, nmap) in st.core[0].iter().enumerate() {
                if *nmap == end {
                    continue;
                }
                print!("{} => {}, ", index, nmap.index());
            }
            println!("{} => {}", nx.index(), mx.index());
            */
        
            // Check syntactic feasibility of mapping by ensuring adjacencies
            // of nx map to adjacencies of mx.
            //
            // nx == map to => mx
            //
            // R_succ
            //
            // Check that every neighbor of nx is mapped to a neighbor of mx,
            // then check the reverse, from mx to nx. Check that they have the same
            // count of edges.
            let mut succ_count = [0, 0];
            for j in graph_indices.clone() {
                for n_neigh in g[j].neighbors(nodes[j]) {
                    succ_count[j] += 1;
                    let m_neigh = st[j].core[n_neigh.index()];
                    if m_neigh == end {
                        continue;
                    }
                    let has_edge = g[1-j].find_edge(nodes[1-j], m_neigh).is_some();
                    if !has_edge {
                        continue 'candidates;
                    }
                }
            }
            if succ_count[0] != succ_count[1] {
                continue 'candidates;
            }

            // R_pred
            let mut pred_count = [0, 0];
            for j in graph_indices.clone() {
                for n_neigh in g[j].neighbors_directed(nodes[j], Incoming) {
                    pred_count[j] += 1;
                    let m_neigh = st[j].core[n_neigh.index()];
                    if m_neigh == end {
                        continue;
                    }
                    let has_edge = g[1-j].find_edge(m_neigh, nodes[1-j]).is_some();
                    if !has_edge {
                        continue 'candidates;
                    }
                }
            }
            if pred_count[0] != pred_count[1] {
                continue 'candidates;
            }

            // Check cardinalities of open sets, important to prune
            // the search tree.
            //
            // counts in Tin/Tout
            let t0in = st[0].ins.iter().filter(|&&x| x > 0).count();
            let t1in = st[1].ins.iter().filter(|&&x| x > 0).count();
            let t0out = st[0].out.iter().filter(|&&x| x > 0).count();
            let t1out = st[1].out.iter().filter(|&&x| x > 0).count();
            if t0in != t1in || t0out != t1out {
                continue 'candidates;
            }

            // counts in N0 - M0 - Tin - Tout
            // equal to count in N - ins - outs
            let n0 = st[0].ins.iter().zip(st[0].out.iter())
                        .filter(|&(&a, &b)| a == 0 && b == 0)
                        .count();
            let n1 = st[1].ins.iter().zip(st[1].out.iter())
                        .filter(|&(&a, &b)| a == 0 && b == 0)
                        .count();
                        
            if n0 != n1 {
                continue 'candidates;
            }

            // Add mapping nx <-> mx to the state
            for j in graph_indices.clone() {
                let node_index = nodes[j];
                // map nx -> mx and mx -> nx
                st[j].core[node_index.index()] = nodes[1 - j];

                // update T0 & T1 ins/outs
                // T0out: Node in G0 not in M0 but successor of a node in M0.
                // st.out[0]: Node either in M0 or successor of M0
                for ix in g[j].neighbors(node_index) {
                    if st[j].out[ix.index()] == 0 {
                        st[j].out[ix.index()] = s;
                    }
                }
                for ix in g[j].neighbors_directed(node_index, Incoming) {
                    if st[j].ins[ix.index()] == 0 {
                        st[j].ins[ix.index()] = s;
                    }
                }
            }

            // recurse
            let (is_done, ans) = try_match(s + 1, st, g0, g1);
            if is_done {
                return (true, ans);
            }

            for j in graph_indices.clone() {
                let node_index = nodes[j];

                // undo (n, m) mapping
                st[j].core[node_index.index()] = end;

                // unmark in ins and outs
                for ix in g[j].neighbors(node_index) {
                    if st[j].out[ix.index()] == s {
                        st[j].out[ix.index()] = 0;
                    }
                }
                for ix in g[j].neighbors_directed(node_index, Incoming) {
                    if st[j].ins[ix.index()] == s {
                        st[j].ins[ix.index()] = 0;
                    }
                }
            }
        }
        (false, false)
    }
    let mut st = [Vf2State::<Ix>::new(g0.node_count()),
                  Vf2State::<Ix>::new(g1.node_count())];
    let (_, is_iso) = try_match(1, &mut st, g0, g1);
    is_iso
}

