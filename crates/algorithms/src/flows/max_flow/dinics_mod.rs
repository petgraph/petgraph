#[cfg(feature = "alloc")]
use alloc::{collections::VecDeque, vec::Vec};
use core::{borrow::Borrow, ops::Sub};

use petgraph_core::{
    edge::Edge,
    graph::{
        DirectedGraph, Graph,
        storable::{DataContainer, QueueContainer, StackContainer, StorableGraph, VisitContainer},
    },
};

use super::{other_endpoint, residual_capacity};
use crate::{
    flows::max_flow::{MaxFlowConfigError, MaxFlowOutput, adjusted_residual_flow},
    traits::{Bounded, Measure, MinMax, NonNegative, Zero},
};

/// Edge with its capacity copied out of the graph.
type FlowEdge<G, C> = Edge<<G as Graph>::EdgeId, C, <G as Graph>::NodeId>;

/// Struct to run Dinic's algorithm.
///
/// Uses the builder pattern to set parameters and can be run with the [`run`][Self::run] method.
/// Details of the algorithm can be found in [`run`][Self::run].
pub struct Dinics<'graph_ref, G: Graph, CapFn: Fn(G::EdgeId) -> C, C> {
    pub(super) network: &'graph_ref G,
    pub(super) source: Option<G::NodeId>,
    pub(super) sink: Option<G::NodeId>,
    pub(super) capacities: CapFn,
}

impl<'graph_ref, G: Graph, CapFn: Fn(G::EdgeId) -> C, C> Dinics<'graph_ref, G, CapFn, C> {
    /// Creates a new instance of Dinic's algorithm with the provided graph.
    ///
    /// Data of the graph's edges is interpreted as capacities of edges.
    /// The source and sink nodes can be set using a builder pattern with the
    /// [`with_source`][Self::with_source] and [`with_sink`][Self::with_sink] methods.
    ///
    /// To run the algorithm, call the [`run`][Self::run] method on the returned struct.
    #[must_use]
    pub const fn new(
        network: &'graph_ref G,
    ) -> Dinics<'graph_ref, G, impl Fn(G::EdgeId) -> G::EdgeData<'graph_ref>, G::EdgeData<'graph_ref>>
    where
        G: DirectedGraph,
        G::EdgeData<'graph_ref>: Clone,
    {
        Dinics::new_with_capacities(network, |edge_id| {
            #[expect(clippy::missing_panics_doc, reason = "infallible")]
            network.edge(edge_id).unwrap().data.borrow().clone()
        })
    }

    /// Creates a new instance of Dinic's algorithm with the provided graph and capacity function.
    ///
    /// The source and sink nodes can be set using a builder pattern with the
    /// [`with_source`][Self::with_source] and [`with_sink`][Self::with_sink] methods.
    ///
    /// To run the algorithm, call the [`run`][Self::run] method on the returned struct.
    #[must_use]
    pub const fn new_with_capacities(network: &'graph_ref G, capacities: CapFn) -> Self
    where
        G: DirectedGraph,
        CapFn: Fn(G::EdgeId) -> C,
    {
        Self {
            network,
            source: None,
            sink: None,
            capacities,
        }
    }

    /// Sets the source node for the flow.
    #[must_use]
    pub const fn with_source(mut self, source: G::NodeId) -> Self {
        self.source = Some(source);
        self
    }

    /// Sets the sink node for the flow.
    #[must_use]
    pub const fn with_sink(mut self, sink: G::NodeId) -> Self {
        self.sink = Some(sink);
        self
    }

    /// Creates a new instance of Dinic's algorithm with the provided graph and capacity function.
    ///
    /// The source and sink nodes can be set using a builder pattern with the
    /// [`with_source`][Self::with_source] and [`with_sink`][Self::with_sink] methods.
    ///
    /// To run the algorithm, call the [`run`][Self::run] method on the returned struct.
    #[must_use]
    pub const fn with_capacities<NewCapFn, NewC>(
        network: &'graph_ref G,
        capacities: NewCapFn,
    ) -> Dinics<'graph_ref, G, NewCapFn, NewC>
    where
        G: DirectedGraph,
        NewCapFn: Fn(G::EdgeId) -> NewC,
    {
        Dinics {
            network,
            source: None,
            sink: None,
            capacities,
        }
    }
}

impl<G, CapFn: Fn(G::EdgeId) -> C, C> Dinics<'_, G, CapFn, C>
where
    G: DirectedGraph + StorableGraph,
{
    /// Run [Dinic's (or Dinitz's) algorithm][dinics_link] to find a [Maximum Flow][maximum_flow]
    /// from `source` to `sink`.
    ///
    /// The algorithm works by building successive level graphs using breadth-first search and finds
    /// blocking flows within them through depth-first searches.
    ///
    /// Edge Data of the provided graph is interpreted as capacities of edges.
    ///
    /// See also the [`max_flow`](../index.html) module for other maximum flow algorithms.
    ///
    /// # Arguments
    /// The following arguments need to be set in the algorithm struct before calling this method:
    /// - `source`: Source node for the flow.
    /// - `destination`: Sink node for the flow.
    /// - `capacities` (optional): Function that returns the non-negative capacity of an edge given
    ///   its ID. If not set, the edge data of the graph is used as capacity.
    ///
    /// # Returns
    /// Returns a struct wrapping the maximum flow value and the flow of each edge.
    ///
    /// # Complexity
    /// - Time complexity:
    ///   - In general: **O(|V|²|E|)**
    ///   - In networks with only unit capacities: **O(min{|V|²ᐟ³, |E|¹ᐟ²} |E|)**
    /// - Auxiliary space: **O(|V| + |E|)**.
    ///
    /// Where **|V|** is the number of nodes and **|E|** is the number of edges in the graph.
    /// The exact complexity depends on the graph backend. That is, an algorithm with runtime O(|V|)
    /// may have effective running time O(|V|^2) if accessing the graph is O(|V|) instead of O(1).
    ///
    /// # Errors
    /// Returns an error if the source or sink node is not set.
    ///
    /// # Example
    ///
    ///
    /// [maximum_flow]: https://en.wikipedia.org/wiki/Maximum_flow_problem
    /// [dinics_link]: https://en.wikipedia.org/wiki/Dinic%27s_algorithm
    #[cfg(feature = "alloc")]
    pub fn run(&self) -> Result<MaxFlowOutput<G, C>, MaxFlowConfigError>
    where
        C: Sub<Output = C> + Measure + NonNegative + Zero + Bounded + MinMax + Default,
    {
        self.run_with_stack_and_queue_container(|| Vec::new(), || Vec::new(), || VecDeque::new())
    }

    /// Run [Dinic's (or Dinitz's) algorithm][dinics_link] to find a [Maximum Flow][maximum_flow]
    /// from `source` to `sink` using a custom stack and queue containers.
    ///
    /// For further details, see the [`run`](Dinics::run) method of the [Dinics] struct.
    ///
    /// [maximum_flow]: https://en.wikipedia.org/wiki/Maximum_flow_problem
    /// [dinics_link]: https://en.wikipedia.org/wiki/Dinic%27s_algorithm
    pub fn run_with_stack_and_queue_container<SContainer, SContainerFlowEdge, QContainer>(
        &self,
        stack_container_node_id_constructor: impl Fn() -> SContainer,
        stack_container_flow_edge_constructor: impl Fn() -> SContainerFlowEdge,
        queue_container_node_id_constructor: impl Fn() -> QContainer,
    ) -> Result<MaxFlowOutput<G, C>, MaxFlowConfigError>
    where
        C: Sub<Output = C> + Measure + NonNegative + Zero + Bounded + MinMax + Default,
        SContainer: StackContainer<G::NodeId> + Clone,
        SContainerFlowEdge: StackContainer<FlowEdge<G, C>> + Clone,
        QContainer: QueueContainer<G::NodeId> + Clone,
    {
        let source = self.source.ok_or(MaxFlowConfigError::SourceNodeNotSet)?;
        let sink = self.sink.ok_or(MaxFlowConfigError::SinkNodeNotSet)?;
        Ok(dinics_inner(
            self.network,
            source,
            sink,
            &self.capacities,
            stack_container_node_id_constructor,
            stack_container_flow_edge_constructor,
            queue_container_node_id_constructor,
        ))
    }
}

/// Run [Dinic's (or Dinitz's) algorithm][dinics_link] to find a [Maximum Flow][maximum_flow] from
/// `source` to `sink`.
///
/// For further details, see the [`run`](Dinics::run) method of the [Dinics] struct.
///
/// [maximum_flow]: https://en.wikipedia.org/wiki/Maximum_flow_problem
/// [dinics_link]: https://en.wikipedia.org/wiki/Dinic%27s_algorithm
#[cfg(feature = "alloc")]
pub fn dinics<'graph_ref, G, C>(
    network: &'graph_ref G,
    source: G::NodeId,
    sink: G::NodeId,
    capacities: impl Fn(G::EdgeId) -> C,
) -> MaxFlowOutput<G, C>
where
    G: DirectedGraph + StorableGraph,
    C: Sub<Output = C> + Measure + NonNegative + Zero + Bounded + MinMax + Default,
{
    Dinics::new_with_capacities(network, capacities)
        .with_source(source)
        .with_sink(sink)
        .run()
        .expect("All required parameters should be set.")
}

fn dinics_inner<G, C, SContainerNodeId, SContainerFlowEdge, QContainer>(
    network: &G,
    source: G::NodeId,
    sink: G::NodeId,
    capacities: impl Fn(G::EdgeId) -> C,
    stack_container_node_id_constructor: impl Fn() -> SContainerNodeId,
    stack_container_flow_edge_constructor: impl Fn() -> SContainerFlowEdge,
    queue_container_node_id_constructor: impl Fn() -> QContainer,
) -> MaxFlowOutput<G, C>
where
    G: DirectedGraph + StorableGraph,
    C: Sub<Output = C> + Measure + NonNegative + Zero + Bounded + MinMax + Default,
    SContainerNodeId: StackContainer<G::NodeId> + Clone,
    SContainerFlowEdge: StackContainer<FlowEdge<G, C>> + Clone,
    QContainer: QueueContainer<G::NodeId> + Clone,
{
    if source == sink {
        return MaxFlowOutput {
            max_flow: C::zero(),
            flows: network.edge_data_container::<C>(C::zero()),
        };
    }

    let mut max_flow = C::zero();
    let mut flows = network.edge_data_container::<C>(C::zero());
    let mut visited = network.node_visit_container();
    let mut level_edges =
        network.node_data_container::<SContainerFlowEdge>(stack_container_flow_edge_constructor());

    while *build_level_graph(
        network,
        source,
        sink,
        &capacities,
        &flows,
        &mut level_edges,
        &queue_container_node_id_constructor,
    )
    .get(sink)
        > 0
    {
        let flow_increase = find_blocking_flow(
            network,
            source,
            sink,
            &mut flows,
            &mut level_edges,
            &mut visited,
            &stack_container_node_id_constructor,
        );
        max_flow = max_flow + flow_increase;
    }
    MaxFlowOutput { max_flow, flows }
}

/// Makes a BFS that labels network vertices with levels representing
/// their distance to the source vertex, considering only edges with
/// positive residual capacity.
///
/// The source vertex is labeled as 1, and vertices not reachable are
/// labeled as 0.
///
/// Aggregates in `level_edges` the edges that connects each
/// vertex to its neighbours in the next level.
///
/// Returns the computed level graph.
fn build_level_graph<G, C, SContainer, QContainer>(
    network: &G,
    source: G::NodeId,
    sink: G::NodeId,
    capacities: impl Fn(G::EdgeId) -> C,
    flows: &impl DataContainer<G::EdgeId, C>,
    level_edges: &mut impl DataContainer<G::NodeId, SContainer>,
    queue_container_node_id_constructor: impl Fn() -> QContainer,
) -> impl DataContainer<G::NodeId, usize>
where
    G: DirectedGraph + StorableGraph,
    C: Sub<Output = C> + Measure + NonNegative + Zero,
    SContainer: StackContainer<FlowEdge<G, C>>,
    QContainer: QueueContainer<G::NodeId> + Clone,
{
    let mut level_graph = network.node_data_container::<usize>(0);
    let mut bfs_queue = queue_container_node_id_constructor();
    bfs_queue.push_back(source);

    *level_graph.get_mut(source) = 1;
    while let Some(vertex) = bfs_queue.pop_front() {
        let incident_edges = network.incident_edges(vertex);
        level_edges.get_mut(vertex).clear();
        for edge in incident_edges {
            let capacity = (capacities)(edge.id);
            let edge = Edge {
                id: edge.id,
                source: edge.source,
                target: edge.target,
                data: capacity,
            };
            let next_vertex = other_endpoint::<G, _>(&edge, vertex);
            let residual_cap = residual_capacity::<G, C>(edge, next_vertex, *flows.get(edge.id));
            if residual_cap == C::zero() {
                continue;
            }
            if *level_graph.get(next_vertex) == 0 {
                *level_graph.get_mut(next_vertex) = *level_graph.get(vertex) + 1;
                level_edges.get_mut(vertex).push(edge);
                if next_vertex != sink {
                    bfs_queue.push_back(next_vertex);
                }
            } else if *level_graph.get(next_vertex) == *level_graph.get(vertex) + 1 {
                level_edges.get_mut(vertex).push(edge);
            }
        }
    }

    level_graph
}

/// Find blocking flow for current level graph by repeatingly finding
/// augmenting paths in it.
///
/// Attach computed flows to `flows` and returns the total flow increase from
/// edges available in `level_edges` at this iteration.
fn find_blocking_flow<G, C, SContainerNodeId, SContainerFlowEdge>(
    network: &G,
    source: G::NodeId,
    sink: G::NodeId,
    flows: &mut G::EdgeDataContainer<C>,
    level_edges: &mut G::NodeDataContainer<SContainerFlowEdge>,
    visited: &mut G::NodeVisitContainer,
    stack_container_builder: impl Fn() -> SContainerNodeId,
) -> C
where
    G: DirectedGraph + StorableGraph,
    C: Sub<Output = C> + Measure + NonNegative + Zero + Bounded + MinMax + Default,
    SContainerNodeId: StackContainer<G::NodeId>,
    SContainerFlowEdge: StackContainer<FlowEdge<G, C>> + Clone,
{
    let mut flow_increase = C::zero();
    let mut edge_to = network.node_data_container::<Option<FlowEdge<G, C>>>(None);
    while find_augmenting_path(
        network,
        source,
        sink,
        flows,
        level_edges,
        visited,
        &mut edge_to,
        &stack_container_builder,
    ) {
        let mut path_flow = <C as Bounded>::max();

        // Find the bottleneck capacity of the path
        let mut vertex = sink;
        while let Some(edge) = edge_to.get(vertex) {
            let residual_capacity =
                residual_capacity::<G, C>(edge.to_owned_edge(), vertex, *flows.get(edge.id));
            path_flow = path_flow.min_cmp(&residual_capacity);
            vertex = other_endpoint::<G, _>(edge, vertex);
        }

        // Update the flow of each edge along the discovered path
        let mut vertex = sink;
        while let Some(edge) = edge_to.get(vertex) {
            *flows.get_mut(edge.id) =
                adjusted_residual_flow::<G, _, C>(edge, vertex, *flows.get(edge.id), path_flow);
            vertex = other_endpoint::<G, _>(edge, vertex);
        }
        flow_increase = flow_increase + path_flow;
    }
    flow_increase
}

/// Makes a DFS to find an augmenting path from source to sink vertex
/// using previously computed `edge_levels` from level graph.
///
/// Returns a boolean indicating if an augmenting path to sink was found.
fn find_augmenting_path<G, C, SContainerNodeId, SContainerFlowEdge>(
    network: &G,
    source: G::NodeId,
    sink: G::NodeId,
    flows: &impl DataContainer<G::EdgeId, C>,
    level_edges: &mut G::NodeDataContainer<SContainerFlowEdge>,
    visited: &mut G::NodeVisitContainer,
    edge_to: &mut G::NodeDataContainer<Option<FlowEdge<G, C>>>,
    stack_container_builder: impl Fn() -> SContainerNodeId,
) -> bool
where
    G: DirectedGraph + StorableGraph,
    C: Sub<Output = C> + Measure + NonNegative + Zero,
    SContainerNodeId: StackContainer<G::NodeId>,
    SContainerFlowEdge: StackContainer<FlowEdge<G, C>> + Clone,
{
    visited.clear();
    let mut level_edges_i = network.node_data_container::<usize>(0);

    let mut dfs_stack = stack_container_builder();
    dfs_stack.push(source);
    visited.mark_visited(source);
    while let Some(&vertex) = dfs_stack.peek_last() {
        let mut found_next = false;
        while *level_edges_i.get(vertex) < level_edges.get(vertex).len() {
            let curr_level_edges_i = level_edges_i.get(vertex);
            let edge = level_edges.get(vertex)[*curr_level_edges_i];
            let next_vertex = other_endpoint::<G, _>(&edge, vertex);

            let residual_cap = residual_capacity::<G, C>(edge, next_vertex, *flows.get(edge.id));
            if residual_cap == C::zero() {
                level_edges.get_mut(vertex).swap_remove(*curr_level_edges_i);
                continue;
            }

            if !visited.is_visited(next_vertex) {
                edge_to.get_mut(next_vertex).replace(edge);
                if sink == next_vertex {
                    return true;
                }
                dfs_stack.push(next_vertex);
                visited.mark_visited(next_vertex);
                found_next = true;
                break;
            }
            *level_edges_i.get_mut(vertex) += 1;
        }
        if !found_next {
            dfs_stack.pop();
        }
    }
    false
}
