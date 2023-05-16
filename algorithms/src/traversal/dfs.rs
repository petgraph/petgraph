use alloc::vec::Vec;
use core::ops::ControlFlow;

use petgraph_core::visit::{GraphBase, GraphRef, IntoNeighbors, VisitMap, Visitable};

/// Visit nodes of a graph in a depth-first-search (DFS) emitting nodes in
/// preorder (when they are first discovered).
///
/// The traversal starts at a given node and only traverses nodes reachable
/// from it.
///
/// `Dfs` is not recursive.
///
/// `Dfs` does not itself borrow the graph, and because of this you can run
/// a traversal over a graph while still retaining mutable access to it, if you
/// use it like the following example:
///
/// ```
/// use petgraph_graph::Graph;
///
/// let mut graph = Graph::<_, ()>::new();
/// let a = graph.add_node(0);
///
/// let mut dfs = Dfs::new(&graph, a);
/// while let Some(nx) = dfs.next(&graph) {
///     // we can access `graph` mutably here still
///     graph[nx] += 1;
/// }
///
/// assert_eq!(graph[a], 1);
/// ```
///
/// **Note:** The algorithm may not behave correctly if nodes are removed
/// during iteration. It may not necessarily visit added nodes or edges.
#[derive(Clone, Debug)]
pub struct Dfs<N, VM> {
    /// The stack of nodes to visit
    pub stack: Vec<N>,
    /// The map of discovered nodes
    pub discovered: VM,
}

impl<N, VM> Default for Dfs<N, VM>
where
    VM: Default,
{
    fn default() -> Self {
        Dfs {
            stack: Vec::new(),
            discovered: VM::default(),
        }
    }
}

impl<N, VM> Dfs<N, VM>
where
    N: Copy + PartialEq,
    VM: VisitMap<N>,
{
    /// Create a new **Dfs**, using the graph's visitor map, and put **start**
    /// in the stack of nodes to visit.
    pub fn new<G>(graph: G, start: N) -> Self
    where
        G: GraphRef + Visitable<NodeId = N, Map = VM>,
    {
        let mut dfs = Dfs::empty(graph);
        dfs.move_to(start);
        dfs
    }

    /// Create a `Dfs` from a vector and a visit map
    pub fn from_parts(stack: Vec<N>, discovered: VM) -> Self {
        Dfs { stack, discovered }
    }

    /// Clear the visit state
    pub fn reset<G>(&mut self, graph: G)
    where
        G: GraphRef + Visitable<NodeId = N, Map = VM>,
    {
        graph.reset_map(&mut self.discovered);
        self.stack.clear();
    }

    /// Create a new **Dfs** using the graph's visitor map, and no stack.
    pub fn empty<G>(graph: G) -> Self
    where
        G: GraphRef + Visitable<NodeId = N, Map = VM>,
    {
        Dfs {
            stack: Vec::new(),
            discovered: graph.visit_map(),
        }
    }

    /// Keep the discovered map, but clear the visit stack and restart
    /// the dfs from a particular node.
    pub fn move_to(&mut self, start: N) {
        self.stack.clear();
        self.stack.push(start);
    }

    /// Return the next node in the dfs, or **None** if the traversal is done.
    pub fn next<G>(&mut self, graph: G) -> Option<N>
    where
        G: IntoNeighbors<NodeId = N>,
    {
        while let Some(node) = self.stack.pop() {
            if self.discovered.visit(node) {
                for succ in graph.neighbors(node) {
                    if !self.discovered.is_visited(&succ) {
                        self.stack.push(succ);
                    }
                }
                return Some(node);
            }
        }
        None
    }
}

type DfsSpaceType<G> = DfsSpace<<G as GraphBase>::NodeId, <G as Visitable>::Map>;

/// Workspace for a graph traversal.
#[derive(Clone, Debug)]
pub struct DfsSpace<N, VM> {
    dfs: Dfs<N, VM>,
}

impl<N, VM> DfsSpace<N, VM>
where
    N: Copy + PartialEq,
    VM: VisitMap<N>,
{
    pub fn new<G>(g: G) -> Self
    where
        G: GraphRef + Visitable<NodeId = N, Map = VM>,
    {
        DfsSpace { dfs: Dfs::empty(g) }
    }
}

impl<N, VM> Default for DfsSpace<N, VM>
where
    VM: VisitMap<N> + Default,
{
    fn default() -> Self {
        DfsSpace {
            dfs: Dfs {
                stack: Default::default(),
                discovered: Default::default(),
            },
        }
    }
}

/// Create a Dfs if it's needed
fn with_dfs<G, F, R>(g: G, space: Option<&mut DfsSpaceType<G>>, f: F) -> R
where
    G: GraphRef + Visitable,
    F: FnOnce(&mut Dfs<G::NodeId, G::Map>) -> R,
{
    let mut local_visitor;
    let dfs = if let Some(v) = space {
        &mut v.dfs
    } else {
        local_visitor = Dfs::empty(g);
        &mut local_visitor
    };
    f(dfs)
}

// TODO: fn depth_first_search() -> impl Iterator<Item = NodeIndex>

/// Strictly monotonically increasing event time for a depth first search.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Default, Hash)]
pub struct Time(pub usize);

/// A depth first search (DFS) visitor event.
#[derive(Copy, Clone, Debug)]
pub enum DfsEvent<N> {
    Discover(N, Time),
    /// An edge of the tree formed by the traversal.
    TreeEdge(N, N),
    /// An edge to an already visited node.
    BackEdge(N, N),
    /// A cross or forward edge.
    ///
    /// For an edge *(u, v)*, if the discover time of *v* is greater than *u*,
    /// then it is a forward edge, else a cross edge.
    CrossForwardEdge(N, N),
    /// All edges from a node have been reported.
    Finish(N, Time),
}

pub struct DepthFirstSearchIter<G>
where
    G: IntoNeighbors + Visitable,
{
    graph: G,
    dfs: Dfs<G::NodeId, G::Map>,
}

impl<G> DepthFirstSearchIter<G>
where
    G: IntoNeighbors + Visitable,
{
    pub fn new(graph: G, starts: impl IntoIterator<Item = G::NodeId>) -> Self {
        let visit = graph.visit_map();

        Self {
            graph,
            dfs: Dfs::from_parts(starts.into_iter().collect(), visit),
        }
    }

    pub fn into_dfs(self) -> Dfs<G::NodeId, G::Map> {
        self.dfs
    }

    fn visit(&mut self) -> DfsEvent<G::NodeId> {
        todo!();

        if !self.dfs.discovered.is_visited() {
            return C::continuing();
        }

        try_control!(
            visitor(DfsEvent::Discover(u, time_post_inc(time))),
            {},
            for v in graph.neighbors(u) {
                if !discovered.is_visited(&v) {
                    try_control!(visitor(DfsEvent::TreeEdge(u, v)), continue);
                    try_control!(
                        dfs_visitor(graph, v, visitor, discovered, finished, time),
                        unreachable!()
                    );
                } else if !finished.is_visited(&v) {
                    try_control!(visitor(DfsEvent::BackEdge(u, v)), continue);
                } else {
                    try_control!(visitor(DfsEvent::CrossForwardEdge(u, v)), continue);
                }
            }
        );
        let first_finish = finished.visit(u);
        debug_assert!(first_finish);
        try_control!(
            visitor(DfsEvent::Finish(u, time_post_inc(time))),
            panic!("Pruning on the `DfsEvent::Finish` is not supported!")
        );
        C::continuing()
    }
}

impl<G> Iterator for DepthFirstSearchIter<G>
where
    G: IntoNeighbors + Visitable,
{
    type Item = DfsEvent<G::NodeId>;

    fn next(&mut self) -> Option<Self::Item> {
        self.dfs.next(self.graph)
    }
}

/// A recursive depth first search.
///
/// Starting points are the nodes in the iterator `starts` (specify just one
/// start vertex *x* by using `Some(x)`).
///
/// The traversal emits discovery and finish events for each reachable vertex,
/// and edge classification of each reachable edge. `visitor` is called for each
/// event, see [`DfsEvent`][de] for possible values.
///
/// The return value should implement the trait `ControlFlow`, and can be used to change
/// the control flow of the search.
///
/// `Control` Implements `ControlFlow` such that `Control::Continue` resumes the search.
/// `Control::Break` will stop the visit early, returning the contained value.
/// `Control::Prune` will stop traversing any additional edges from the current
/// node and proceed immediately to the `Finish` event.
///
/// There are implementations of `ControlFlow` for `()`, and `Result<C, E>` where
/// `C: ControlFlow`. The implementation for `()` will continue until finished.
/// For `Result`, upon encountering an `E` it will break, otherwise acting the same as `C`.
///
/// ***Panics** if you attempt to prune a node from its `Finish` event.
///
/// [de]: enum.DfsEvent.html
///
/// # Example returning `Control`.
///
/// Find a path from vertex 0 to 5, and exit the visit as soon as we reach
/// the goal vertex.
///
/// ```
/// use petgraph_core::visit::{depth_first_search, Control, DfsEvent};
/// use petgraph_graph::{Graph, NodeIndex};
///
/// let gr: Graph<(), ()> = Graph::from_edges(&[
///     (0, 1),
///     (0, 2),
///     (0, 3),
///     (1, 3),
///     (2, 3),
///     (2, 4),
///     (4, 0),
///     (4, 5),
/// ]);
///
/// // record each predecessor, mapping node → node
/// let mut predecessor = vec![NodeIndex::end(); gr.node_count()];
/// let start = n(0);
/// let goal = n(5);
/// depth_first_search(&gr, Some(start), |event| {
///     if let DfsEvent::TreeEdge(u, v) = event {
///         predecessor[v.index()] = u;
///         if v == goal {
///             return Control::Break(v);
///         }
///     }
///     Control::Continue
/// });
///
/// let mut next = goal;
/// let mut path = vec![next];
/// while next != start {
///     let pred = predecessor[next.index()];
///     path.push(pred);
///     next = pred;
/// }
/// path.reverse();
/// assert_eq!(&path, &[n(0), n(2), n(4), n(5)]);
/// ```
///
/// # Example returning a `Result`.
/// ```
/// use petgraph_core::visit::{depth_first_search, DfsEvent, Time};
/// use petgraph_graph::Graph;
///
/// let gr: Graph<(), ()> = Graph::from_edges(&[(0, 1), (1, 2), (1, 1), (2, 1)]);
/// let start = n(0);
/// let mut back_edges = 0;
/// let mut discover_time = 0;
/// // Stop the search, the first time a BackEdge is encountered.
/// let result = depth_first_search(&gr, Some(start), |event| {
///     match event {
///         // In the cases where Ok(()) is returned,
///         // Result falls back to the implementation of Control on the value ().
///         // In the case of (), this is to always return Control::Continue.
///         // continuing the search.
///         DfsEvent::Discover(_, Time(t)) => {
///             discover_time = t;
///             Ok(())
///         }
///         DfsEvent::BackEdge(..) => {
///             back_edges += 1;
///             // the implementation of ControlFlow for Result,
///             // treats this Err value as Continue::Break
///             Err(event)
///         }
///         _ => Ok(()),
///     }
/// });
///
/// // Even though the graph has more than one cycle,
/// // The number of back_edges visited by the search should always be 1.
/// assert_eq!(back_edges, 1);
/// println!("discover time:{:?}", discover_time);
/// println!("number of backedges encountered: {}", back_edges);
/// println!("back edge: {:?}", result);
/// ```
pub fn depth_first_search<G>(
    graph: G,
    starts: impl IntoIterator<Item = G::NodeId>,
) -> impl Iterator<Item = DfsEvent<G::NodeId>>
where
    G: IntoNeighbors + Visitable,
{
    let time = &mut Time(0);
    let discovered = &mut graph.visit_map();
    let finished = &mut graph.visit_map();

    DepthFirstSearchIter::new(graph, starts)
}

fn dfs_visitor<G, F, C>(
    graph: G,
    u: G::NodeId,
    visitor: &mut F,
    discovered: &mut G::Map,
    finished: &mut G::Map,
    time: &mut Time,
) -> C
where
    G: IntoNeighbors + Visitable,
    F: FnMut(DfsEvent<G::NodeId>) -> C,
    C: ControlFlow,
{
    if !discovered.visit(u) {
        return C::continuing();
    }

    try_control!(
        visitor(DfsEvent::Discover(u, time_post_inc(time))),
        {},
        for v in graph.neighbors(u) {
            if !discovered.is_visited(&v) {
                try_control!(visitor(DfsEvent::TreeEdge(u, v)), continue);
                try_control!(
                    dfs_visitor(graph, v, visitor, discovered, finished, time),
                    unreachable!()
                );
            } else if !finished.is_visited(&v) {
                try_control!(visitor(DfsEvent::BackEdge(u, v)), continue);
            } else {
                try_control!(visitor(DfsEvent::CrossForwardEdge(u, v)), continue);
            }
        }
    );
    let first_finish = finished.visit(u);
    debug_assert!(first_finish);
    try_control!(
        visitor(DfsEvent::Finish(u, time_post_inc(time))),
        panic!("Pruning on the `DfsEvent::Finish` is not supported!")
    );
    C::continuing()
}

fn time_post_inc(x: &mut Time) -> Time {
    let v = *x;
    x.0 += 1;
    v
}
