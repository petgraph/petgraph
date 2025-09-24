use alloc::{collections::VecDeque, vec::Vec};

use super::{
    GraphRef, IntoNeighbors, IntoNeighborsDirected, IntoNodeIdentifiers, Reversed, VisitMap,
    Visitable,
};
use crate::Incoming;

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
/// use petgraph::Graph;
/// use petgraph::visit::Dfs;
///
/// let mut graph = Graph::<_,()>::new();
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

/// Visit nodes in a depth-first-search (DFS) emitting nodes in postorder
/// (each node after all its descendants have been emitted).
///
/// `DfsPostOrder` is not recursive.
///
/// The traversal starts at a given node and only traverses nodes reachable
/// from it.
#[derive(Clone, Debug)]
pub struct DfsPostOrder<N, VM> {
    /// The stack of nodes to visit
    pub stack: Vec<N>,
    /// The map of discovered nodes
    pub discovered: VM,
    /// The map of finished nodes
    pub finished: VM,
}

impl<N, VM> Default for DfsPostOrder<N, VM>
where
    VM: Default,
{
    fn default() -> Self {
        DfsPostOrder {
            stack: Vec::new(),
            discovered: VM::default(),
            finished: VM::default(),
        }
    }
}

impl<N, VM> DfsPostOrder<N, VM>
where
    N: Copy + PartialEq,
    VM: VisitMap<N>,
{
    /// Create a new `DfsPostOrder` using the graph's visitor map, and put
    /// `start` in the stack of nodes to visit.
    pub fn new<G>(graph: G, start: N) -> Self
    where
        G: GraphRef + Visitable<NodeId = N, Map = VM>,
    {
        let mut dfs = Self::empty(graph);
        dfs.move_to(start);
        dfs
    }

    /// Create a new `DfsPostOrder` using the graph's visitor map, and no stack.
    pub fn empty<G>(graph: G) -> Self
    where
        G: GraphRef + Visitable<NodeId = N, Map = VM>,
    {
        DfsPostOrder {
            stack: Vec::new(),
            discovered: graph.visit_map(),
            finished: graph.visit_map(),
        }
    }

    /// Clear the visit state
    pub fn reset<G>(&mut self, graph: G)
    where
        G: GraphRef + Visitable<NodeId = N, Map = VM>,
    {
        graph.reset_map(&mut self.discovered);
        graph.reset_map(&mut self.finished);
        self.stack.clear();
    }

    /// Keep the discovered and finished map, but clear the visit stack and restart
    /// the dfs from a particular node.
    pub fn move_to(&mut self, start: N) {
        self.stack.clear();
        self.stack.push(start);
    }

    /// Return the next node in the traversal, or `None` if the traversal is done.
    pub fn next<G>(&mut self, graph: G) -> Option<N>
    where
        G: IntoNeighbors<NodeId = N>,
    {
        while let Some(&nx) = self.stack.last() {
            if self.discovered.visit(nx) {
                // First time visiting `nx`: Push neighbors, don't pop `nx`
                for succ in graph.neighbors(nx) {
                    if !self.discovered.is_visited(&succ) {
                        self.stack.push(succ);
                    }
                }
            } else {
                self.stack.pop();
                if self.finished.visit(nx) {
                    // Second time: All reachable nodes must have been finished
                    return Some(nx);
                }
            }
        }
        None
    }
}

/// A breadth first search (BFS) of a graph.
///
/// The traversal starts at a given node and only traverses nodes reachable
/// from it.
///
/// `Bfs` is not recursive.
///
/// `Bfs` does not itself borrow the graph, and because of this you can run
/// a traversal over a graph while still retaining mutable access to it, if you
/// use it like the following example:
///
/// **Note:** The algorithm may not behave correctly if nodes are removed
/// during iteration. It may not necessarily visit added nodes or edges.
///
/// ```
/// use petgraph::Graph;
/// use petgraph::visit::Bfs;
///
/// let mut graph = Graph::<_,()>::new();
/// let a = graph.add_node(0);
///
/// let mut bfs = Bfs::new(&graph, a);
/// while let Some(nx) = bfs.next(&graph) {
///     // we can access `graph` mutably here still
///     graph[nx] += 1;
/// }
///
/// assert_eq!(graph[a], 1);
/// ```
///
/// It is also possible to receive the distances from the starting point:
///
/// ```
/// use petgraph::graph::UnGraph;
/// use petgraph::visit::Bfs;
///
/// // Create an undirected graph with associated data
/// // of type `i32` for the nodes and `()` for the edges.
/// let g = UnGraph::<i32, ()>::from_edges(&[
///     (0, 1), (1, 2), (2, 3), (0, 3)
/// ]);
/// // The graph looks like this:
/// // 0 -- 1
/// // |    |
/// // 3 -- 2
///
/// // only nodes without distances
/// {
///     let mut bfs = Bfs::new(&g, 0.into());
///     assert_eq!(Some(0.into()), bfs.next(&g));
///     let second = bfs.next(&g).unwrap().index();
///     let third = bfs.next(&g).unwrap().index();
///     assert!(
///         (1 == second && 3 == third) ||
///         (1 == third && 3 == second)
///     );
///     assert_eq!(Some(2.into()), bfs.next(&g));
///     assert_eq!(None, bfs.next(&g));
/// }
///
/// // nodes with distances (u32 is enough for all distances here)
/// {
///     let mut dist_bfs = Bfs::new_with_distances::<u32>(&g, 0.into());
///     assert_eq!(Some((0, 0.into())), dist_bfs.next(&g));
///     let second = dist_bfs.next(&g).unwrap();
///     let third = dist_bfs.next(&g).unwrap();
///     assert!(
///         ((1, 1.into()) == second && (1, 3.into()) == third) ||
///         (((1, 3.into()) == second) && (1, 1.into()) == third)
///     );
///     assert_eq!(Some((2, 2.into())), dist_bfs.next(&g));
///     assert_eq!(None, dist_bfs.next(&g));
/// }
/// ```
///
#[derive(Clone)]
pub struct Bfs<N, VM, D = ()> {
    /// The queue of nodes to visit
    pub stack: VecDeque<N>,
    /// The map of discovered nodes
    pub discovered: VisitMapPhantom<VM, D>,
}

pub struct WithDistancesMarker<D>(core::marker::PhantomData<D>);

#[derive(Clone)]
pub struct VisitMapPhantom<VM, D>(pub VM, core::marker::PhantomData<D>);
impl<VM, D> core::ops::Deref for VisitMapPhantom<VM, D> {
    type Target = VM;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<VM, D> core::ops::DerefMut for VisitMapPhantom<VM, D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl<VM, D> VisitMapPhantom<VM, D> {
    fn new(vm: VM) -> Self {
        Self(vm, Default::default())
    }
}
impl<VM, D> Default for VisitMapPhantom<VM, D>
where
    VM: Default,
{
    fn default() -> Self {
        Self(Default::default(), Default::default())
    }
}

impl<N, VM> Default for Bfs<N, VM>
where
    VM: Default,
{
    fn default() -> Self {
        Bfs {
            stack: VecDeque::new(),
            discovered: Default::default(),
        }
    }
}

impl<N, VM> Bfs<N, VM>
where
    N: Copy + PartialEq,
    VM: VisitMap<N>,
{
    /// Create a new **Bfs**, using the graph's visitor map, and put **start**
    /// in the stack of nodes to visit.
    pub fn new<G>(graph: G, start: N) -> Self
    where
        G: GraphRef + Visitable<NodeId = N, Map = VM>,
    {
        let mut discovered = graph.visit_map();
        discovered.visit(start);
        let mut stack = VecDeque::new();
        stack.push_front(start);
        Bfs {
            stack,
            discovered: VisitMapPhantom::new(discovered),
        }
    }

    /// Create a new **Bfs** that also returns the distances to the starting point,
    /// using the graph's visitor map, and put **start** with distance 0 in the stack of nodes to visit.
    ///
    /// (`D` is the distance type (unsigned integer), checked by enforcing `From<u8>` as bound)
    pub fn new_with_distances<D>(
        graph: impl GraphRef + Visitable<NodeId = N, Map = VM>,
        start: N,
    ) -> Bfs<(D, N), VM, WithDistancesMarker<D>>
    where
        D: Copy + From<u8>,
    {
        let mut discovered = graph.visit_map();
        discovered.visit(start);
        let mut stack = VecDeque::new();
        stack.push_front((D::from(0u8), start));
        Bfs {
            stack,
            discovered: VisitMapPhantom::new(discovered),
        }
    }
}

impl<N, VM> Bfs<N, VM, ()>
where
    N: Copy + PartialEq,
    VM: VisitMap<N>,
{
    /// Return the next node in the bfs, or **None** if the traversal is done.
    pub fn next<G>(&mut self, graph: G) -> Option<N>
    where
        G: IntoNeighbors<NodeId = N>,
    {
        if let Some(node) = self.stack.pop_front() {
            for succ in graph.neighbors(node) {
                if self.discovered.visit(succ) {
                    self.stack.push_back(succ);
                }
            }

            return Some(node);
        }
        None
    }
}
impl<N, VM, D> Bfs<(D, N), VM, WithDistancesMarker<D>>
where
    N: Copy + PartialEq,
    VM: VisitMap<N>,
    D: From<u8> + core::ops::Add<D, Output = D> + Copy,
{
    /// Return the distance to the next node in the bfs and the node itself, or **None** if the traversal is done.
    pub fn next<G>(&mut self, graph: G) -> Option<(D, N)>
    where
        G: IntoNeighbors<NodeId = N>,
    {
        if let Some((distance, node)) = self.stack.pop_front() {
            for succ in graph.neighbors(node) {
                if self.discovered.visit(succ) {
                    self.stack.push_back((distance + D::from(1u8), succ));
                }
            }

            return Some((distance, node));
        }
        None
    }
}

/// A topological order traversal for a graph.
///
/// **Note** that `Topo` only visits nodes that are not part of cycles,
/// i.e. nodes in a true DAG. Use other visitors like [`DfsPostOrder`] or
/// algorithms like [`kosaraju_scc`][crate::algo::kosaraju_scc()] to handle
/// graphs with possible cycles.
#[derive(Clone)]
pub struct Topo<N, VM> {
    tovisit: Vec<N>,
    ordered: VM,
}

impl<N, VM> Default for Topo<N, VM>
where
    VM: Default,
{
    fn default() -> Self {
        Topo {
            tovisit: Vec::new(),
            ordered: VM::default(),
        }
    }
}

impl<N, VM> Topo<N, VM>
where
    N: Copy + PartialEq,
    VM: VisitMap<N>,
{
    /// Create a new `Topo`, using the graph's visitor map, and put all
    /// initial nodes in the to visit list.
    pub fn new<G>(graph: G) -> Self
    where
        G: IntoNodeIdentifiers + IntoNeighborsDirected + Visitable<NodeId = N, Map = VM>,
    {
        let mut topo = Self::empty(graph);
        topo.extend_with_initials(graph);
        topo
    }

    /// Create a new `Topo` with initial nodes.
    ///
    /// Nodes with incoming edges are ignored.
    pub fn with_initials<G, I>(graph: G, initials: I) -> Self
    where
        G: IntoNeighborsDirected + Visitable<NodeId = N, Map = VM>,
        I: IntoIterator<Item = N>,
    {
        Topo {
            tovisit: initials
                .into_iter()
                .filter(|&n| graph.neighbors_directed(n, Incoming).next().is_none())
                .collect(),
            ordered: graph.visit_map(),
        }
    }

    fn extend_with_initials<G>(&mut self, g: G)
    where
        G: IntoNodeIdentifiers + IntoNeighborsDirected<NodeId = N>,
    {
        // find all initial nodes (nodes without incoming edges)
        self.tovisit.extend(
            g.node_identifiers()
                .filter(move |&a| g.neighbors_directed(a, Incoming).next().is_none()),
        );
    }

    /* Private until it has a use */
    /// Create a new `Topo`, using the graph's visitor map with *no* starting
    /// index specified.
    fn empty<G>(graph: G) -> Self
    where
        G: GraphRef + Visitable<NodeId = N, Map = VM>,
    {
        Topo {
            ordered: graph.visit_map(),
            tovisit: Vec::new(),
        }
    }

    /// Clear visited state, and put all initial nodes in the to visit list.
    pub fn reset<G>(&mut self, graph: G)
    where
        G: IntoNodeIdentifiers + IntoNeighborsDirected + Visitable<NodeId = N, Map = VM>,
    {
        graph.reset_map(&mut self.ordered);
        self.tovisit.clear();
        self.extend_with_initials(graph);
    }

    /// Return the next node in the current topological order traversal, or
    /// `None` if the traversal is at the end.
    ///
    /// *Note:* The graph may not have a complete topological order, and the only
    /// way to know is to run the whole traversal and make sure it visits every node.
    pub fn next<G>(&mut self, g: G) -> Option<N>
    where
        G: IntoNeighborsDirected + Visitable<NodeId = N, Map = VM>,
    {
        // Take an unvisited element and find which of its neighbors are next
        while let Some(nix) = self.tovisit.pop() {
            if self.ordered.is_visited(&nix) {
                continue;
            }
            self.ordered.visit(nix);
            for neigh in g.neighbors(nix) {
                // Look at each neighbor, and those that only have incoming edges
                // from the already ordered list, they are the next to visit.
                if Reversed(g)
                    .neighbors(neigh)
                    .all(|b| self.ordered.is_visited(&b))
                {
                    self.tovisit.push(neigh);
                }
            }
            return Some(nix);
        }
        None
    }
}

/// A walker is a traversal state, but where part of the traversal
/// information is supplied manually to each next call.
///
/// This for example allows graph traversals that don't hold a borrow of the
/// graph they are traversing.
pub trait Walker<Context> {
    type Item;
    /// Advance to the next item
    fn walk_next(&mut self, context: Context) -> Option<Self::Item>;

    /// Create an iterator out of the walker and given `context`.
    fn iter(self, context: Context) -> WalkerIter<Self, Context>
    where
        Self: Sized,
        Context: Clone,
    {
        WalkerIter {
            walker: self,
            context,
        }
    }
}

/// A walker and its context wrapped into an iterator.
#[derive(Clone, Debug)]
pub struct WalkerIter<W, C> {
    walker: W,
    context: C,
}

impl<W, C> WalkerIter<W, C>
where
    W: Walker<C>,
    C: Clone,
{
    pub fn context(&self) -> C {
        self.context.clone()
    }

    pub fn inner_ref(&self) -> &W {
        &self.walker
    }

    pub fn inner_mut(&mut self) -> &mut W {
        &mut self.walker
    }
}

impl<W, C> Iterator for WalkerIter<W, C>
where
    W: Walker<C>,
    C: Clone,
{
    type Item = W::Item;
    fn next(&mut self) -> Option<Self::Item> {
        self.walker.walk_next(self.context.clone())
    }
}

impl<C, W: ?Sized> Walker<C> for &mut W
where
    W: Walker<C>,
{
    type Item = W::Item;
    fn walk_next(&mut self, context: C) -> Option<Self::Item> {
        (**self).walk_next(context)
    }
}

impl<G> Walker<G> for Dfs<G::NodeId, G::Map>
where
    G: IntoNeighbors + Visitable,
{
    type Item = G::NodeId;
    fn walk_next(&mut self, context: G) -> Option<Self::Item> {
        self.next(context)
    }
}

impl<G> Walker<G> for DfsPostOrder<G::NodeId, G::Map>
where
    G: IntoNeighbors + Visitable,
{
    type Item = G::NodeId;
    fn walk_next(&mut self, context: G) -> Option<Self::Item> {
        self.next(context)
    }
}

impl<G> Walker<G> for Bfs<G::NodeId, G::Map>
where
    G: IntoNeighbors + Visitable,
{
    type Item = G::NodeId;
    fn walk_next(&mut self, context: G) -> Option<Self::Item> {
        self.next(context)
    }
}

impl<G> Walker<G> for Topo<G::NodeId, G::Map>
where
    G: IntoNeighborsDirected + Visitable,
{
    type Item = G::NodeId;
    fn walk_next(&mut self, context: G) -> Option<Self::Item> {
        self.next(context)
    }
}
