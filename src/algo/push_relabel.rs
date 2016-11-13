use prelude::*;
use std::cmp::{max, min};
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;
use visit::{
    EdgeFiltered,
    IntoEdgeReferences,
    IntoEdges,
    IntoNodeIdentifiers,
    Visitable,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum MaxFlowError {
    ArithmeticOverflow,
}

struct Node<N> {
    orig_id: N,
    excess: i64,
    label: usize,
}

impl<N> Node<N> {
    fn new(orig_id: N) -> Node<N> {
        Node {
            orig_id: orig_id,
            excess: 0,
            label: 0,
        }
    }
}

struct Edge {
    capacity: i64,
    flow: i64,
}

impl Edge {
    fn new(capacity: i64) -> Edge {
        Edge {
            capacity: capacity,
            flow: 0,
        }
    }
}

struct State<N> {
    graph: Graph<Node<N>, ()>,
    // We need random access to the edges, so it is faster to store them in a
    // hashmap rather than in `graph`.
    edges: HashMap<(NodeId, NodeId), Edge>,
    target: NodeId,
    active_queue: VecDeque<NodeId>,
}

type PRGraph<N> = Graph<Node<N>, ()>;
type NodeId = NodeIndex<u32>;

impl<N: Copy> State<N> {
    fn push(&mut self, u: NodeId, v: NodeId) -> Result<(), MaxFlowError> {
        let new_flow = {
            let u_data = self.graph.node_weight(u).unwrap();
            let v_data = self.graph.node_weight(v).unwrap();
            let e_data = self.edges.get(&(u, v)).unwrap();

            debug_assert!(u_data.excess > 0);
            debug_assert!(u_data.label == v_data.label + 1);

            min(u_data.excess, e_data.capacity - e_data.flow)
        };
        try!(self.add_excess(u, -new_flow));
        try!(self.add_excess(v, new_flow));
        self.edges.get_mut(&(u, v)).unwrap().flow += new_flow;
        self.edges.get_mut(&(v, u)).unwrap().flow -= new_flow;
        Ok(())
    }

    fn has_capacity(&self, u: NodeId, v: NodeId) -> bool {
        let e = self.edges.get(&(u, v)).unwrap();
        e.capacity > e.flow
    }

    fn can_push(&self, u: NodeId, v: NodeId) -> bool {
        self.has_capacity(u, v) && self.graph[u].label == self.graph[v].label + 1
    }

    fn add_excess(&mut self, u: NodeId, amount: i64) -> Result<(), MaxFlowError> {
        debug_assert!(amount != 0);

        // The target node never has any excess inflow, since it can just gobble it up.
        if u == self.target {
            return Ok(());
        }

        let node = self.graph.node_weight_mut(u).unwrap();
        // We should never try to push more flow than the node has available.
        // There is one special case: the start node always has non-positive
        // excess flow.
        debug_assert!(node.excess <= 0 || node.excess >= -amount);
        if node.excess == 0 {
            // We weren't active before, but we are now.
            self.active_queue.push_back(u);
        }
        node.excess =
            try!(node.excess.checked_add(amount).ok_or(MaxFlowError::ArithmeticOverflow));
        Ok(())
    }

    // Keep pushing excess flow to neighbors until we can't any more.
    fn discharge(&mut self, u: NodeId) -> Result<(), MaxFlowError> {
        let mut nbrs = self.graph.neighbors(u).detach();
        while self.graph[u].excess > 0 {
            if let Some(v) = nbrs.next_node(&self.graph) {
                if self.can_push(u, v) {
                    try!(self.push(u, v));
                }
            } else {
                self.relabel(u);
                nbrs = self.graph.neighbors(u).detach();
            }
        }
        Ok(())
    }

    fn relabel(&mut self, u: NodeId) {
        let min_nbr_label = self.graph.neighbors(u)
            .filter(|v| self.has_capacity(u, *v))
            .map(|v| self.graph.node_weight(v).unwrap().label)
            .min()
            .expect("bug: tried to relabel a node with no outgoing edges");
        self.graph.node_weight_mut(u).unwrap().label = min_nbr_label + 1;
    }

    fn new<G>(g: G, source: G::NodeId, target: G::NodeId)
        -> State<G::NodeId>
        where G: IntoEdgeReferences<EdgeWeight=i64, NodeId=N> + IntoNodeIdentifiers,
              G::NodeId: Hash + Eq,
    {
        // Map from nodes of `g` to nodes in `pr_graph`.
        let mut node_map = HashMap::new();
        let mut pr_graph = PRGraph::new();
        let mut edges = HashMap::new();

        for n in g.node_identifiers() {
            let pr_id = pr_graph.add_node(Node::new(n.clone()));
            node_map.insert(n, pr_id);
        }
        for e in g.edge_references() {
            let u = node_map[&e.source()];
            let v = node_map[&e.target()];
            pr_graph.add_edge(u, v, ());
            edges.insert((u, v), Edge::new(max(*e.weight(), 0)));
        }

        // The algorithm requires that every edge has its reversal present.
        for e in g.edge_references() {
            let u = node_map[&e.source()];
            let v = node_map[&e.target()];
            if !edges.contains_key(&(v, u)) {
                edges.insert((v, u), Edge::new(0));
                pr_graph.add_edge(v, u, ());
            }
        }

        let pr_source = *node_map.get(&source).expect("source node isn't in the graph");
        pr_graph[pr_source].label = pr_graph.node_count();
        let mut nbrs = pr_graph.neighbors(pr_source).detach();
        let mut active = VecDeque::new();

        while let Some(v) = nbrs.next_node(&pr_graph) {
            let cap = edges[&(pr_source, v)].capacity;
            edges.get_mut(&(pr_source, v)).unwrap().flow = cap;
            edges.get_mut(&(v, pr_source)).unwrap().flow = -cap;
            pr_graph[v].excess += cap;
            pr_graph[pr_source].excess -= cap;
            active.push_back(v);
        }

        State {
            edges: edges,
            graph: pr_graph,
            target: *node_map.get(&target).expect("target node isn't in the graph"),
            active_queue: active,
        }
    }

    fn run(&mut self) -> Result<(), MaxFlowError> {
        while let Some(u) = self.active_queue.pop_front() {
            try!(self.discharge(u));
        }
        Ok(())
    }
}

/// Computes a max flow from `source` to `target` in the weighted graph `g` using the push-relabel
/// algorithm.
///
/// The edge weights in `g` are interpreted as edge capacities -- negative weights are treated the
/// same as zero weights.
///
/// Returns `HashMap` that maps ordered pairs of vertices to the flow between them. The map only
/// contains pairs of vertices with a strictly positive flow. Returns an error if an arithmetic
/// overflow occurred.
///
/// Panics if `source` or `target` is an invalid node index for `g`.
pub fn push_relabel_max_flow<G>(g: G, source: G::NodeId, target: G::NodeId)
    -> Result<HashMap<(G::NodeId, G::NodeId), i64>, MaxFlowError>
    where G: IntoEdgeReferences<EdgeWeight=i64> + IntoNodeIdentifiers,
          G::NodeId: Clone + Hash + Eq,
{
    let mut state = State::new(g, source, target);
    try!(state.run());

    let graph = state.graph;
    let flow = state.edges.into_iter()
        .filter(|&(_, ref data)|  data.flow > 0)
        .map(|((u, v), data)| ((graph[u].orig_id, graph[v].orig_id), data.flow))
        .collect::<HashMap<_,_>>();

    Ok(flow)
}

/// Computes a (directed) min cut of `g` separating `source` and `target` using the push-relabel
/// algorithm.
///
/// The edge weights in `g` should be non-negative. Any negative weights are treated as though they
/// were zero.
///
/// Panics if `source` or `target` is an invalid node index for `g`.
pub fn push_relabel_min_cut<G>(g: G, source: G::NodeId, target: G::NodeId)
    -> Result<Vec<G::EdgeRef>, MaxFlowError>
    where G: IntoEdges<EdgeWeight=i64> + IntoNodeIdentifiers + Visitable,
          G::NodeId: Clone + Hash + Eq,
{
    let flow = try!(push_relabel_max_flow(g, source, target));
    let spare_capacity = |e: G::EdgeRef| {
        flow.get(&(e.source(), e.target())).unwrap_or(&0) < e.weight()
    };
    let residual_g = EdgeFiltered::from_fn(g, spare_capacity);

    // Find the connected component of `source` in the residual graph.
    let mut source_cc = HashSet::new();
    let mut dfs = Dfs::new(&residual_g, source);
    while let Some(u) = dfs.next(&residual_g) {
        source_cc.insert(u);
    }

    // Every edge connecting `source_cc` with its complement belongs to the min cut.
    let mut ret = Vec::new();
    for &u in &source_cc {
        for e in g.edges(u) {
            if !source_cc.contains(&e.target()) {
                ret.push(e);
            }
        }
    }
    Ok(ret)
}

