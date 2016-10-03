//! Graph algorithms.
//!
//! It is a goal to gradually migrate the algorithms to be based on graph traits
//! so that they are generally applicable. For now, some of these still require
//! the `Graph` type.

use std::collections::BinaryHeap;
use std::cmp::min;

use prelude::*;

use super::{
    EdgeType,
};
use scored::MinScored;
use super::visit::{
    GraphRef,
    Visitable,
    VisitMap,
    IntoNeighbors,
    IntoNeighborsDirected,
    IntoNodeIdentifiers,
    IntoExternals,
    NodeIndexable,
    NodeCompactIndexable,
    IntoEdgeReferences,
    EdgeRef,
    Reversed,
};
use super::unionfind::UnionFind;
use super::graph::{
    IndexType,
};

pub use super::isomorphism::{
    is_isomorphic,
    is_isomorphic_matching,
};
pub use super::dijkstra::dijkstra;

/// [Generic] Return the number of connected components of the graph.
///
/// For a directed graph, this is the *weakly* connected components.
pub fn connected_components<G>(g: G) -> usize
    where G: NodeCompactIndexable + IntoEdgeReferences,
{
    let mut vertex_sets = UnionFind::new(g.node_bound());
    for edge in g.edge_references() {
        let (a, b) = (edge.source(), edge.target());

        // union the two vertices of the edge
        vertex_sets.union(G::to_index(a), G::to_index(b));
    }
    let mut labels = vertex_sets.into_labeling();
    labels.sort();
    labels.dedup();
    labels.len()
}


/// [Generic] Return `true` if the input graph contains a cycle.
///
/// Always treats the input graph as if undirected.
pub fn is_cyclic_undirected<G>(g: G) -> bool
    where G: NodeIndexable + IntoEdgeReferences
{
    let mut edge_sets = UnionFind::new(g.node_bound());
    for edge in g.edge_references() {
        let (a, b) = (edge.source(), edge.target());

        // union the two vertices of the edge
        //  -- if they were already the same, then we have a cycle
        if !edge_sets.union(G::to_index(a), G::to_index(b)) {
            return true
        }
    }
    false
}

/// [Generic] Perform a topological sort of a directed graph `g`.
///
/// Visit each node in order (if it is part of a topological order).
///
/// If `space` is not `None`, it is reused instead of creating a temporary
/// workspace for graph traversal.
#[inline]
fn toposort_generic<G, F>(g: G,
                          space: Option<&mut DfsSpaceType<G>>,
                          mut visit: F)
    where G: IntoNeighborsDirected + IntoExternals + Visitable,
          F: FnMut(G, G::NodeId),
{
    with_dfs(g, space, |dfs| {
        dfs.reset(g);
        let mut ordered = &mut dfs.discovered;
        let mut tovisit = &mut dfs.stack;

        // find all initial nodes
        tovisit.extend(g.externals(Incoming));

        // Take an unvisited element and find which of its neighbors are next
        while let Some(nix) = tovisit.pop() {
            if ordered.is_visited(&nix) {
                continue;
            }
            visit(g, nix.clone());
            ordered.visit(nix.clone());
            for neigh in g.neighbors_directed(nix, Outgoing) {
                // Look at each neighbor, and those that only have incoming edges
                // from the already ordered list, they are the next to visit.
                if g.neighbors_directed(neigh.clone(), Incoming)
                    .all(|b| ordered.is_visited(&b)) {
                    tovisit.push(neigh);
                }
            }
        }
    })
}

/// [Generic] Return `true` if the input directed graph contains a cycle.
///
/// If `space` is not `None`, it is reused instead of creating a temporary
/// workspace for graph traversal.
pub fn is_cyclic_directed<G>(g: G, space: Option<&mut DfsSpaceType<G>>) -> bool
    where G: IntoNodeIdentifiers + IntoNeighborsDirected + IntoExternals + Visitable,
{
    let mut n_ordered = 0;
    toposort_generic(g, space, |_, _| n_ordered += 1);
    n_ordered != g.node_count()
}

type DfsSpaceType<G> where G: Visitable = DfsSpace<G::NodeId, G::Map>;

/// Workspace for a graph traversal.
#[derive(Clone, Debug)]
pub struct DfsSpace<N, VM> {
    dfs: Dfs<N, VM>,
}

impl<N, VM> DfsSpace<N, VM>
    where N: Copy,
          VM: VisitMap<N>,
{
    pub fn new<G>(g: G) -> Self
        where G: GraphRef + Visitable<NodeId=N, Map=VM>,
    {
        DfsSpace {
            dfs: Dfs::empty(g)
        }
    }
}

impl<N, VM> Default for DfsSpace<N, VM>
    where VM: VisitMap<N> + Default,
{
    fn default() -> Self {
        DfsSpace {
            dfs: Dfs {
                stack: <_>::default(),
                discovered: <_>::default(),
            }
        }
    }
}

/// [Generic] Perform a topological sort of a directed graph.
///
/// Return a vector of nodes in topological order: each node is ordered
/// before its successors.
///
/// NOTE: If the returned vec contains less than all the nodes of the graph,
/// then the graph had a cycle.
///
/// If `space` is not `None`, it is reused instead of creating a temporary
/// workspace for graph traversal.
pub fn toposort<G>(g: G, space: Option<&mut DfsSpaceType<G>>) -> Vec<G::NodeId>
    where G: IntoNodeIdentifiers + IntoNeighborsDirected + IntoExternals + Visitable,
{
    let mut order = Vec::with_capacity(g.node_count());
    toposort_generic(g, space, |_, ix| order.push(ix));
    order
}

/// Create a Dfs if it's needed
fn with_dfs<G, F, R>(g: G, space: Option<&mut DfsSpaceType<G>>, f: F) -> R
    where G: GraphRef + Visitable,
          F: FnOnce(&mut Dfs<G::NodeId, G::Map>) -> R
{
    let mut local_visitor;
    let dfs = if let Some(v) = space { &mut v.dfs } else {
        local_visitor = Dfs::empty(g);
        &mut local_visitor
    };
    f(dfs)
}

/// [Generic] Check if there exists a path starting at `from` and reaching `to`.
///
/// `from` and `to` are equal, this function returns true.
///
/// If `space` is not `None`, it is reused instead of creating a temporary
/// workspace for graph traversal.
pub fn has_path_connecting<G>(g: G, from: G::NodeId, to: G::NodeId,
                              space: Option<&mut DfsSpaceType<G>>)
    -> bool
    where G: IntoNeighbors + Visitable,
          G::NodeId: PartialEq,
{
    with_dfs(g, space, |dfs| {
        dfs.reset(g);
        dfs.move_to(from);
        while let Some(x) = dfs.next(g) {
            if x == to {
                return true;
            }
        }
        false
    })
}


/// [Generic] Compute the *strongly connected components* using Kosaraju's algorithm.
///
/// Return a vector where each element is a strongly connected component (scc).
///
/// The order of `NodeId` within each scc is arbitrary, but the order of
/// the sccs is their postorder (reverse topological sort).
///
/// This implementation is iterative and does two passes over the nodes.
///
/// For an undirected graph, the sccs are simply the connected components.
pub fn scc<G>(g: G) -> Vec<Vec<G::NodeId>>
    where G: IntoNeighborsDirected + Visitable + IntoNodeIdentifiers,
{
    let mut dfs = DfsPostOrder::empty(g);

    // First phase, reverse dfs pass, compute finishing times.
    // http://stackoverflow.com/a/26780899/161659
    let mut finish_order = Vec::with_capacity(g.node_count());
    for i in g.node_identifiers() {
        if dfs.discovered.is_visited(&i) {
            continue
        }

        dfs.move_to(i);
        while let Some(nx) = dfs.next(Reversed(g)) {
            finish_order.push(nx);
        }
    }

    let mut dfs = Dfs::from_parts(dfs.stack, dfs.discovered);
    dfs.reset(g);
    let mut sccs = Vec::new();

    // Second phase
    // Process in decreasing finishing time order
    for i in finish_order.into_iter().rev() {
        if dfs.discovered.is_visited(&i) {
            continue;
        }
        // Move to the leader node `i`.
        dfs.move_to(i);
        let mut scc = Vec::new();
        while let Some(nx) = dfs.next(g) {
            scc.push(nx);
        }
        sccs.push(scc);
    }
    sccs
}

/// [Generic] Compute the *strongly connected components* using Tarjan's algorithm.
///
/// Return a vector where each element is a strongly connected component (scc).
///
/// The order of `NodeId` within each scc is arbitrary, but the order of
/// the sccs is their postorder (reverse topological sort).
///
/// This implementation is recursive and does one pass over the nodes.
///
/// For an undirected graph, the sccs are simply the connected components.
pub fn tarjan_scc<G>(g: G) -> Vec<Vec<G::NodeId>>
    where G: IntoNodeIdentifiers + IntoNeighbors + NodeIndexable
{
    #[derive(Copy, Clone)]
    #[derive(Debug)]
    struct NodeData {
        index: Option<usize>,
        lowlink: usize,
        on_stack: bool,
    }

    #[derive(Debug)]
    struct Data<'a, G>
        where G: NodeIndexable, 
          G::NodeId: 'a
    {
        index: usize,
        nodes: Vec<NodeData>,
        stack: Vec<G::NodeId>,
        sccs: &'a mut Vec<Vec<G::NodeId>>,
    }

    fn scc_visit<G>(v: G::NodeId, g: G, data: &mut Data<G>) 
        where G: IntoNeighbors + NodeIndexable
    {
        macro_rules! node {
            ($node:expr) => (data.nodes[G::to_index($node)])
        }

        if node![v].index.is_some() {
            // already visited
            return;
        }

        let v_index = data.index;
        node![v].index = Some(v_index);
        node![v].lowlink = v_index;
        node![v].on_stack = true;
        data.stack.push(v);
        data.index += 1;

        for w in g.neighbors(v) {
            match node![w].index {
                None => {
                    scc_visit(w, g, data);
                    node![v].lowlink = min(node![v].lowlink, node![w].lowlink);
                }
                Some(w_index) => {
                    if node![w].on_stack {
                        // Successor w is in stack S and hence in the current SCC
                        let v_lowlink = &mut node![v].lowlink;
                        *v_lowlink = min(*v_lowlink, w_index);
                    }
                }
            }
        }

        // If v is a root node, pop the stack and generate an SCC
        if let Some(v_index) = node![v].index {
            if node![v].lowlink == v_index {
                let mut cur_scc = Vec::new();
                loop {
                    let w = data.stack.pop().unwrap();
                    node![w].on_stack = false;
                    cur_scc.push(w);
                    if G::to_index(w) == G::to_index(v) { break; }
                }
                data.sccs.push(cur_scc);
            }
        }
    }

    let mut sccs = Vec::new();
    {
        let map = vec![NodeData { index: None, lowlink: !0, on_stack: false }; g.node_bound()];

        let mut data = Data {
            index: 0,
            nodes: map,
            stack: Vec::new(),
            sccs: &mut sccs,
        };

        for n in g.node_identifiers() {
            scc_visit(n, g, &mut data);
        }
    }
    sccs
}

/// [Graph] Condense every strongly connected component into a single node and return the result.
///
/// If `make_acyclic` is true, self-loops and multi edges are ignored, guaranteeing that
/// the output is acyclic.
pub fn condensation<N, E, Ty, Ix>(g: Graph<N, E, Ty, Ix>, make_acyclic: bool) -> Graph<Vec<N>, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{
    let sccs = scc(&g);
    let mut condensed: Graph<Vec<N>, E, Ty, Ix> = Graph::with_capacity(sccs.len(), g.edge_count());

    // Build a map from old indices to new ones.
    let mut node_map = vec![NodeIndex::end(); g.node_count()];
    for comp in sccs {
        let new_nix = condensed.add_node(Vec::new());
        for nix in comp {
            node_map[nix.index()] = new_nix;
        }
    }

    // Consume nodes and edges of the old graph and insert them into the new one.
    let (nodes, edges) = g.into_nodes_edges();
    for (nix, node) in nodes.into_iter().enumerate() {
        condensed[node_map[nix]].push(node.weight);
    }
    for edge in edges {
        let source = node_map[edge.source().index()];
        let target = node_map[edge.target().index()];
        if make_acyclic {
            if source != target {
                condensed.update_edge(source, target, edge.weight);
            }
        } else {
            condensed.add_edge(source, target, edge.weight);
        }
    }
    condensed
}

/// [Graph] Compute a *minimum spanning tree* of a graph.
///
/// Treat the input graph as undirected.
///
/// Using Kruskal's algorithm with runtime **O(|E| log |E|)**. We actually
/// return a minimum spanning forest, i.e. a minimum spanning tree for each connected
/// component of the graph.
///
/// The resulting graph has all the vertices of the input graph (with identical node indices),
/// and **|V| - c** edges, where **c** is the number of connected components in `g`.
pub fn min_spanning_tree<N, E, Ty, Ix>(g: &Graph<N, E, Ty, Ix>)
    -> Graph<N, E, Undirected, Ix>
    where N: Clone,
          E: Clone + PartialOrd,
          Ty: EdgeType,
          Ix: IndexType,
{
    if g.node_count() == 0 {
        return Graph::with_capacity(0, 0)
    }

    // Create a mst skeleton by copying all nodes
    let mut mst = Graph::with_capacity(g.node_count(), g.node_count() - 1);
    for node in g.raw_nodes() {
        mst.add_node(node.weight.clone());
    }

    // Initially each vertex is its own disjoint subgraph, track the connectedness
    // of the pre-MST with a union & find datastructure.
    let mut subgraphs = UnionFind::new(g.node_count());

    let mut sort_edges = BinaryHeap::with_capacity(g.edge_count());
    for edge in g.edge_references() {
        sort_edges.push(MinScored(edge.weight().clone(), (edge.source(), edge.target())));
    }

    // Kruskal's algorithm.
    // Algorithm is this:
    //
    // 1. Create a pre-MST with all the vertices and no edges.
    // 2. Repeat:
    //
    //  a. Remove the shortest edge from the original graph.
    //  b. If the edge connects two disjoint trees in the pre-MST,
    //     add the edge.
    while let Some(MinScored(score, (a, b))) = sort_edges.pop() {
        // check if the edge would connect two disjoint parts
        if subgraphs.union(a.index(), b.index()) {
            mst.add_edge(a, b, score);
        }
    }

    debug_assert!(mst.node_count() == g.node_count());
    debug_assert!(mst.edge_count() < g.node_count());
    mst
}
