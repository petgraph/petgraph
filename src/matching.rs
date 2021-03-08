use std::collections::{hash_set, HashSet, VecDeque};
use std::hash::Hash;
use std::iter::Copied;

use super::visit::{
    EdgeRef, GraphBase, IntoEdges, IntoNodeIdentifiers, NodeCompactIndexable, NodeCount,
    NodeIndexable, VisitMap, Visitable,
};

pub struct Matching<G: GraphBase> {
    edges: HashSet<G::EdgeId>,
    nodes: HashSet<G::NodeId>,
    is_perfect: bool,
}

impl<G: GraphBase> Matching<G>
where
    G::NodeId: Eq + Hash,
    G::EdgeId: Eq + Hash,
{
    fn empty() -> Self {
        Self {
            edges: HashSet::new(),
            nodes: HashSet::new(),
            is_perfect: false,
        }
    }

    pub fn is_perfect(&self) -> bool {
        self.is_perfect
    }

    pub fn edges(&self) -> Matched<G::EdgeId> {
        Matched {
            inner: self.edges.iter().copied(),
        }
    }

    pub fn contains_edge(&self, edge: G::EdgeId) -> bool {
        self.edges.contains(&edge)
    }

    pub fn nodes(&self) -> Matched<G::NodeId> {
        Matched {
            inner: self.nodes.iter().copied(),
        }
    }

    pub fn contains_node(&self, node: G::NodeId) -> bool {
        self.nodes.contains(&node)
    }
}

pub struct Matched<'a, T> {
    inner: Copied<hash_set::Iter<'a, T>>,
}

impl<T: Copy> Iterator for Matched<'_, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

/// \[Generic\] Compute a
/// [*matching*](https://en.wikipedia.org/wiki/Matching_(graph_theory)) using a
/// greedy heuristic.
///
/// The input graph is treated as if undirected. The underlying heuristic is
/// unspecified, but is guaranteed to be bounded by *O(|V| + |E|)*. No
/// guarantees are provided other than the output is a valid matching.
pub fn greedy_matching<G>(graph: G) -> Matching<G>
where
    G: Visitable + IntoNodeIdentifiers + IntoEdges + NodeCount,
    G::NodeId: Eq + Hash,
    G::EdgeId: Eq + Hash,
{
    let mut matching = Matching::empty();
    let visited = &mut graph.visit_map();

    for start in graph.node_identifiers() {
        let mut last = Some(start);
        non_backtracking_dfs(graph, start, visited, |next, edge| {
            // Alternate matched and unmatched edges.
            if let Some(pred) = last.take() {
                matching.edges.insert(edge);
                matching.nodes.insert(pred);
                matching.nodes.insert(next);
            } else {
                last = Some(next);
            }
        });
    }

    let n = graph.node_count();
    matching.is_perfect = n % 2 == 0 && matching.edges.len() == n / 2;
    matching
}

fn non_backtracking_dfs<G, F>(graph: G, source: G::NodeId, visited: &mut G::Map, mut visitor: F)
where
    G: Visitable + IntoEdges,
    F: FnMut(G::NodeId, G::EdgeId),
{
    if visited.visit(source) {
        for edge in graph.edges(source) {
            let target = edge.target();

            if !visited.is_visited(&target) {
                visitor(target, edge.id());
                non_backtracking_dfs(graph, target, visited, visitor);

                // Non-backtracking traversal, stop iterating over the
                // neighbors.
                break;
            }
        }
    }
}

trait WithDummy: NodeIndexable {
    fn dummy(&self) -> Self::NodeId;
    fn dummy_idx(&self) -> usize;
    fn node_bound_with_dummy(&self) -> usize;
}

impl<G: NodeIndexable> WithDummy for G {
    fn dummy(&self) -> Self::NodeId {
        self.from_index(self.dummy_idx())
    }

    fn dummy_idx(&self) -> usize {
        // Gabow numbers the vertices from 1 to n, and uses 0 as the dummy
        // vertex. Our vertex indices are zero-based and so we use the node
        // bound as the dummy node.
        self.node_bound()
    }

    fn node_bound_with_dummy(&self) -> usize {
        self.node_bound() + 1
    }
}

#[derive(Clone, Copy)]
enum Label<G: GraphBase> {
    None,
    Start,
    // If node v is outer node, then label(v) = w is another outer node on path
    // from v to start u.
    Vertex(G::NodeId),
    // If node v is outer node, then label(v) = (r, s) are two outer vertices
    // (connected by an edge)
    Edge(G::EdgeId, [G::NodeId; 2]),
    // Flag is a special label used in searching for the join vertex of two
    // paths.
    Flag(G::EdgeId),
}

impl<G: GraphBase> Label<G> {
    fn is_outer(&self) -> bool {
        self != &Label::None
            && !match self {
                Label::Flag(_) => true,
                _ => false,
            }
    }

    fn is_inner(&self) -> bool {
        !self.is_outer()
    }

    fn to_vertex(self) -> Option<G::NodeId> {
        match self {
            Label::Vertex(v) => Some(v),
            _ => None,
        }
    }

    fn is_flagged(&self, edge: G::EdgeId) -> bool {
        match self {
            Label::Flag(flag) if flag == &edge => true,
            _ => false,
        }
    }
}

impl<G: GraphBase> Default for Label<G> {
    fn default() -> Self {
        Label::None
    }
}

impl<G: GraphBase> PartialEq for Label<G> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Label::None, Label::None) => true,
            (Label::Start, Label::Start) => true,
            (Label::Vertex(v1), Label::Vertex(v2)) => v1 == v2,
            (Label::Edge(e1, _), Label::Edge(e2, _)) => e1 == e2,
            (Label::Flag(e1), Label::Flag(e2)) => e1 == e2,
            _ => false,
        }
    }
}

/// \[Generic\] Compute the [*maximum
/// matching*](https://en.wikipedia.org/wiki/Matching_(graph_theory)) using
/// [Gabow's algorithm][1].
///
/// [1]: https://dl.acm.org/doi/10.1145/321941.321942
///
/// The input graph is treated as if undirected. The algorithm runs in
/// *O(|V|³)*. An algorithm with a better time complexity might be used in the
/// future.
pub fn maximum_matching<G>(graph: G) -> Matching<G>
where
    G: Visitable + NodeCompactIndexable + IntoNodeIdentifiers + IntoEdges,
    G::NodeId: Eq + Hash,
    G::EdgeId: Eq + Hash,
{
    let len = graph.node_bound_with_dummy();

    macro_rules! array {
        ($default:expr) => {{
            let mut array = Vec::with_capacity(len);
            array.resize(len, $default);
            array
        }};
    }

    let mut matching = Matching::empty();

    let mut mate = array!(graph.dummy_idx());
    let mut label: Vec<Label<G>> = array!(Label::None);
    let mut first_inner = array!(std::usize::MAX);
    let visited = &mut graph.visit_map();

    // Greedy algorithm should create a fairly good initial matching. The hope
    // is that it speeds up the computation by doing les work in the complex
    // algorithm.
    for start in graph.node_identifiers() {
        let mut last = Some(start);
        non_backtracking_dfs(graph, start, visited, |next, _| {
            // Alternate matched and unmatched edges.
            if let Some(pred) = last.take() {
                let (pred, next) = (graph.to_index(pred), graph.to_index(next));
                mate[pred] = next;
                mate[next] = pred;
            } else {
                last = Some(next);
            }
        });
    }

    for start in 0..graph.node_bound() {
        if mate[start] != graph.dummy_idx() {
            // The vertex is already matched. A start must be a free vertex.
            continue;
        }

        // Begin search from the node.
        label[start] = Label::Start;
        first_inner[start] = graph.dummy_idx();
        graph.reset_map(visited);

        let start = graph.from_index(start);

        // Queue will contain outer vertices that should be processed next. The
        // start vertex is considered an outer vertex.
        let mut queue = VecDeque::new();
        queue.push_back(start);
        // Mark the start vertex so it is not processed repeatedly.
        visited.visit(start);

        'search: while let Some(outer_vertex) = queue.pop_front() {
            let outer_idx = graph.to_index(outer_vertex);

            for edge in graph.edges(outer_vertex) {
                if edge.source() == edge.target() {
                    // Ignore self-loops.
                    continue;
                }

                let other_vertex = edge.target();
                let other_idx = graph.to_index(other_vertex);

                if mate[other_idx] == graph.dummy_idx() && other_vertex != start {
                    // An augmenting path was found. Augment the matching. If
                    // `other` is actually the start node, then the augmentation
                    // must not be performed, because the start vertex would be
                    // incident to two edges, which violates the matching
                    // property.
                    mate[other_idx] = outer_idx;
                    augment_path(&graph, outer_vertex, other_vertex, &mut mate, &label);

                    // The path is augmented, so the start is no longer free
                    // vertex. We need to begin with a new start.
                    break 'search;
                } else if label[other_idx].is_outer() {
                    // The `other` is an outer vertex (a label has been set to
                    // it). An odd cycle (blossom) was found. Assign this edge
                    // as a label to all inner vertices in paths P(outer) and
                    // P(other).
                    find_join(
                        &graph,
                        edge,
                        &mate,
                        &mut label,
                        &mut first_inner,
                        |labeled| {
                            if visited.visit(labeled) {
                                queue.push_back(labeled);
                            }
                        },
                    );
                } else {
                    let mate_idx = mate[other_idx];
                    let mate_vertex = graph.from_index(mate_idx);

                    if label[mate_idx].is_inner() {
                        // Mate of `other` vertex is inner (no label has been
                        // set to it so far). But it actually is an outer vertex
                        // (it is on a path to the start vertex that begins with
                        // a matched edge, since it is a mate of `other`).
                        // Assign the label of this mate to the `outer` vertex,
                        // so the path for it can be reconstructed using `mate`
                        // and this label.
                        label[mate_idx] = Label::Vertex(outer_vertex);
                        first_inner[mate_idx] = other_idx;
                    }

                    if visited.visit(mate_vertex) {
                        // Add the vertex to the queue only if this is its first
                        // discovery.
                        queue.push_back(mate_vertex);
                    }
                }
            }
        }

        // Reset the labels. All vertices are inner for the next search.
        for lbl in label.iter_mut() {
            *lbl = Label::None;
        }
    }

    // Transform the information from `mate` array to the output matching.
    for (vertex_idx, vertex_mate) in mate.into_iter().enumerate() {
        if vertex_idx != graph.dummy_idx() && vertex_mate != graph.dummy_idx() {
            let source = graph.from_index(vertex_idx);

            if !matching.nodes.contains(&source) {
                let target = graph.from_index(vertex_mate);
                let edge = graph
                    .edges(source)
                    .find(|edge| edge.target() == target)
                    .unwrap();

                matching.edges.insert(edge.id());
                matching.nodes.insert(source);
                matching.nodes.insert(target);
            }
        }
    }

    let n = graph.node_count();
    matching.is_perfect = n % 2 == 0 && matching.edges.len() == n / 2;
    matching
}

fn find_join<G, F>(
    graph: &G,
    edge: G::EdgeRef,
    mate: &Vec<usize>,
    label: &mut Vec<Label<G>>,
    first_inner: &mut Vec<usize>,
    mut visitor: F,
) where
    G: IntoEdges + NodeIndexable + Visitable,
    G::EdgeId: Eq + Hash,
    F: FnMut(G::NodeId),
{
    // Simultaneously traverse the inner vertices on paths P(source) and
    // P(target) to find a join vertex - an inner vertex that is shared by these
    // paths.
    let source = graph.to_index(edge.source());
    let target = graph.to_index(edge.target());

    let mut left = first_inner[source];
    let mut right = first_inner[target];

    if left == right {
        // No vertices can be labeled, since both paths already refer to a
        // common vertex - the join.
        return;
    }

    // Flag the (first) inner vertices. This ensures that they are assigned the
    // join as their first inner vertex.
    let flag = Label::Flag(edge.id());
    label[left] = flag;
    label[right] = flag;

    // Find the join.
    let join = loop {
        // Swap the sides. Do not swap if the right side is already finished.
        if right != graph.dummy_idx() {
            std::mem::swap(&mut left, &mut right);
        }

        // Set left to the next inner vertex in P(source) or P(target).
        left = first_inner[graph.to_index(label[mate[left]].to_vertex().unwrap())];

        if !label[left].is_flagged(edge.id()) {
            // The inner vertex is not flagged yet, so flag it.
            label[left] = flag;
        } else {
            // The inner vertex is already flagged. It means that the other side
            // had to visit it already. Therefore it is the join vertex.
            break left;
        }
    };

    // Label all inner vertices on P(source) and P(target) with the found join.
    for endpoint in [source, target].iter().copied() {
        let mut inner = first_inner[endpoint];
        while inner != join {
            // Notify the caller about labeling a vertex.
            visitor(graph.from_index(inner));

            label[inner] = Label::Edge(edge.id(), [edge.source(), edge.target()]);
            first_inner[inner] = join;
            inner = first_inner[graph.to_index(label[mate[inner]].to_vertex().unwrap())];
        }
    }

    for (vertex_idx, vertex_label) in label.iter().enumerate() {
        // To all outer vertices that are on paths P(source) and P(target) until
        // the join, se the join as their first inner vertex.
        if vertex_idx != graph.dummy_idx()
            && vertex_label.is_outer()
            && label[first_inner[vertex_idx]].is_outer()
        {
            first_inner[vertex_idx] = join;
        }
    }
}

fn augment_path<G>(
    graph: &G,
    outer: G::NodeId,
    other: G::NodeId,
    mate: &mut Vec<usize>,
    label: &Vec<Label<G>>,
) where
    G: NodeCompactIndexable,
{
    let outer_idx = graph.to_index(outer);
    let other_idx = graph.to_index(other);

    let temp = mate[outer_idx];
    mate[outer_idx] = other_idx;

    if mate[temp] != outer_idx {
        // We are at the end of the path and so the entire path is completely
        // rematched/augmented.
        return;
    } else if let Label::Vertex(vertex) = label[outer_idx] {
        // The outer vertex has a vertex label which refers to another outer
        // vertex on the path. So we set this another outer node as the mate for
        // the previous mate of the outer node.
        mate[temp] = graph.to_index(vertex);
        augment_path(graph, vertex, graph.from_index(temp), mate, label);
    } else if let Label::Edge(_, [source, target]) = label[outer_idx] {
        // The outer vertex has an edge label which refers to an edge in a
        // blossom. We need to augment both directions along the blossom.
        augment_path(graph, source, target, mate, label);
        augment_path(graph, target, source, mate, label);
    } else {
        panic!("Unexpected label when augmenting path");
    }
}
