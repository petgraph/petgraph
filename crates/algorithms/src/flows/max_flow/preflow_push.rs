#[cfg(feature = "alloc")]
use alloc::{collections::VecDeque, vec::Vec};
use core::{borrow::Borrow, ops::Sub};

use petgraph_core::{
    edge::Edge,
    graph::{
        DirectedGraph, Graph,
        storable::{DataContainer, QueueContainer, StackContainer, StorableGraph},
    },
};

use crate::{
    flows::max_flow::{
        FlowEdge, MaxFlowConfigError, MaxFlowOutput, adjusted_residual_flow, other_endpoint,
        residual_capacity,
    },
    traits::{Measure, MinMax, NonNegative, Zero},
};

/// Struct to run the Preflow-Push algorithm.
///
/// Uses the builder pattern to set parameters and can be run with the [`run`][Self::run] method.
/// Details of the algorithm can be found in [`run`][Self::run].
pub struct PreflowPush<'graph_ref, G: Graph, CapFn: Fn(G::EdgeId) -> C, C> {
    network: &'graph_ref G,
    source: Option<G::NodeId>,
    sink: Option<G::NodeId>,
    capacities: CapFn,
}

impl<'graph_ref, G>
    PreflowPush<'graph_ref, G, fn(G::EdgeId) -> G::EdgeData<'graph_ref>, G::EdgeData<'graph_ref>>
where
    G: DirectedGraph,
    G::EdgeData<'graph_ref>: Clone,
{
    /// Creates a new instance of the Preflow-Push algorithm with the provided graph.
    ///
    /// Data of the graph's edges is interpreted as capacities of edges.
    /// The source and sink nodes can be set using a builder pattern with the
    /// [`with_source`][Self::with_source] and [`with_sink`][Self::with_sink] methods.
    ///
    /// To run the algorithm, call the [`run`][Self::run] method on the returned struct.
    #[must_use]
    pub const fn new(
        network: &'graph_ref G,
    ) -> PreflowPush<
        'graph_ref,
        G,
        impl Fn(G::EdgeId) -> G::EdgeData<'graph_ref>,
        G::EdgeData<'graph_ref>,
    > {
        PreflowPush::new_with_capacities(network, |edge_id| {
            #[expect(clippy::missing_panics_doc, reason = "infallible")]
            network.edge(edge_id).unwrap().data.borrow().clone()
        })
    }
}

impl<'graph_ref, G: Graph, CapFn: Fn(G::EdgeId) -> C, C> PreflowPush<'graph_ref, G, CapFn, C> {
    /// Creates a new instance of the Preflow-Push algorithm with the provided graph and capacity
    /// function.
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
}

impl<G, CapFn: Fn(G::EdgeId) -> C, C> PreflowPush<'_, G, CapFn, C>
where
    G: DirectedGraph + StorableGraph,
{
    /// Run the [Preflow-Push (or Push-Relabel) algorithm][preflow_push_link] to find a
    /// [Maximum Flow][maximum_flow] from `source` to `sink`.
    ///
    /// The algorithm maintains a preflow, which allows nodes to receive more flow than they send,
    /// and a label for each node that estimates its distance to the sink. Nodes with excess flow
    /// push it to neighbors with a smaller label and are relabeled if no such neighbor is left.
    /// Nodes with excess are processed in FIFO order and the labels are periodically recomputed by
    /// a breadth-first search from the sink.
    ///
    /// Edge Data of the provided graph is interpreted as capacities of edges. As nodes temporarily
    /// hold excess flow, the total capacity of the edges entering any node other than the source
    /// has to fit into the capacity type.
    ///
    /// See also the [`max_flow`](../index.html) module for other maximum flow algorithms.
    ///
    /// # Arguments
    /// The following arguments need to be set in the algorithm struct before calling this method:
    /// - `source`: Source node for the flow.
    /// - `sink`: Sink node for the flow.
    /// - `capacities` (optional): Function that returns the non-negative capacity of an edge given
    ///   its ID. If not set, the edge data of the graph is used as capacity.
    ///
    /// # Returns
    /// Returns a struct wrapping the maximum flow value and the flow of each edge.
    ///
    /// # Complexity
    /// - Time complexity: **O(|V|³ + |V||E|)**, which is **O(|V|³)** for graphs without parallel
    ///   edges.
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
    /// // TODO: Add example
    ///
    /// [maximum_flow]: https://en.wikipedia.org/wiki/Maximum_flow_problem
    /// [preflow_push_link]: https://en.wikipedia.org/wiki/Push%E2%80%93relabel_maximum_flow_algorithm
    #[cfg(feature = "alloc")]
    pub fn run(&self) -> Result<MaxFlowOutput<G, C>, MaxFlowConfigError>
    where
        C: Sub<Output = C> + Measure + NonNegative + Zero + MinMax,
    {
        self.run_with_stack_and_queue_container(Vec::new, VecDeque::new)
    }

    /// Run the [Preflow-Push (or Push-Relabel) algorithm][preflow_push_link] to find a
    /// [Maximum Flow][maximum_flow] from `source` to `sink` using custom stack and queue
    /// containers.
    ///
    /// For further details, see the [`run`](PreflowPush::run) method of the [`PreflowPush`]
    /// struct.
    ///
    /// [maximum_flow]: https://en.wikipedia.org/wiki/Maximum_flow_problem
    /// [preflow_push_link]: https://en.wikipedia.org/wiki/Push%E2%80%93relabel_maximum_flow_algorithm
    #[allow(
        clippy::missing_errors_doc,
        reason = "errors are documented in the run() method"
    )]
    pub fn run_with_stack_and_queue_container<SContainer, QContainer>(
        &self,
        stack_container_flow_edge_constructor: impl Fn() -> SContainer,
        queue_container_node_id_constructor: impl Fn() -> QContainer,
    ) -> Result<MaxFlowOutput<G, C>, MaxFlowConfigError>
    where
        C: Sub<Output = C> + Measure + NonNegative + Zero + MinMax,
        SContainer: StackContainer<FlowEdge<G, C>> + Clone,
        QContainer: QueueContainer<G::NodeId>,
    {
        let source = self.source.ok_or(MaxFlowConfigError::SourceNodeNotSet)?;
        let sink = self.sink.ok_or(MaxFlowConfigError::SinkNodeNotSet)?;
        Ok(preflow_push_inner(
            self.network,
            source,
            sink,
            &self.capacities,
            stack_container_flow_edge_constructor,
            queue_container_node_id_constructor,
        ))
    }
}

/// Run the [Preflow-Push (or Push-Relabel) algorithm][preflow_push_link] to find a
/// [Maximum Flow][maximum_flow] from `source` to `sink`.
///
/// For further details, see the [`run`](PreflowPush::run) method of the [`PreflowPush`] struct.
///
/// [maximum_flow]: https://en.wikipedia.org/wiki/Maximum_flow_problem
/// [preflow_push_link]: https://en.wikipedia.org/wiki/Push%E2%80%93relabel_maximum_flow_algorithm
#[cfg(feature = "alloc")]
#[expect(clippy::missing_panics_doc, reason = "source and sink are always set")]
pub fn preflow_push<G, C>(
    network: &G,
    source: G::NodeId,
    sink: G::NodeId,
    capacities: impl Fn(G::EdgeId) -> C,
) -> MaxFlowOutput<G, C>
where
    G: DirectedGraph + StorableGraph,
    C: Sub<Output = C> + Measure + NonNegative + Zero + MinMax,
{
    PreflowPush::new_with_capacities(network, capacities)
        .with_source(source)
        .with_sink(sink)
        .run()
        .expect("All required parameters should be set.")
}

/// Work of relabeling a node in addition to scanning its incident edges, in scanned edges.
const RELABEL_WORK: usize = 12;

fn preflow_push_inner<G, C, SContainer, QContainer>(
    network: &G,
    source: G::NodeId,
    sink: G::NodeId,
    capacities: impl Fn(G::EdgeId) -> C,
    stack_container_flow_edge_constructor: impl Fn() -> SContainer,
    queue_container_node_id_constructor: impl Fn() -> QContainer,
) -> MaxFlowOutput<G, C>
where
    G: DirectedGraph + StorableGraph,
    C: Sub<Output = C> + Measure + NonNegative + Zero + MinMax,
    SContainer: StackContainer<FlowEdge<G, C>> + Clone,
    QContainer: QueueContainer<G::NodeId>,
{
    if source == sink {
        return MaxFlowOutput {
            max_flow: C::zero(),
            flows: network.edge_data_container::<C>(C::zero()),
        };
    }

    let incident_edges =
        collect_incident_edges(network, capacities, stack_container_flow_edge_constructor);
    let mut preflow = Preflow::new(network, source, sink);
    let mut active = queue_container_node_id_constructor();
    let mut bfs_queue = queue_container_node_id_constructor();

    preflow.saturate_source_edges(incident_edges.get(source), &mut active);
    preflow.global_relabel(&incident_edges, &mut bfs_queue);

    // A global relabeling scans every edge twice, so it is run once relabeling has done about as
    // much work, which bounds its total cost by the cost of relabeling.
    let global_relabel_work = network.node_count() + 2 * network.edge_count();
    let mut relabel_work = 0;
    while let Some(node) = active.pop_front() {
        relabel_work += preflow.discharge(node, incident_edges.get(node), &mut active);
        if relabel_work > global_relabel_work {
            preflow.global_relabel(&incident_edges, &mut bfs_queue);
            relabel_work = 0;
        }
    }

    preflow.into_output()
}

/// Collects the incident edges of each node together with their capacities.
///
/// Self-loops and edges without capacity are skipped, as they never carry flow.
fn collect_incident_edges<G, C, SContainer>(
    network: &G,
    capacities: impl Fn(G::EdgeId) -> C,
    stack_container_flow_edge_constructor: impl Fn() -> SContainer,
) -> G::NodeDataContainer<SContainer>
where
    G: DirectedGraph + StorableGraph,
    C: Measure + Zero,
    SContainer: StackContainer<FlowEdge<G, C>> + Clone,
{
    let mut incident_edges = network.node_data_container(stack_container_flow_edge_constructor());
    for edge in network.edges() {
        let capacity = capacities(edge.id);
        if edge.source == edge.target || capacity <= C::zero() {
            continue;
        }
        let edge = Edge {
            id: edge.id,
            source: edge.source,
            target: edge.target,
            data: capacity,
        };
        incident_edges.get_mut(edge.source).push(edge);
        incident_edges.get_mut(edge.target).push(edge);
    }
    incident_edges
}

/// Preflow together with the node labels of the algorithm.
///
/// The label of a node is a lower bound on its distance to the sink in the residual network or,
/// if the sink is unreachable, on `|V|` plus its distance to the source. Label `2|V|` marks nodes
/// that can reach neither.
struct Preflow<G: StorableGraph, C: Clone> {
    source: G::NodeId,
    sink: G::NodeId,
    node_count: usize,
    flows: G::EdgeDataContainer<C>,
    excess: G::NodeDataContainer<C>,
    labels: G::NodeDataContainer<usize>,
    /// Index of the first incident edge of each node that may still be admissible.
    current_edge: G::NodeDataContainer<usize>,
}

impl<G, C> Preflow<G, C>
where
    G: DirectedGraph + StorableGraph,
    C: Sub<Output = C> + Measure + NonNegative + Zero + MinMax,
{
    fn new(network: &G, source: G::NodeId, sink: G::NodeId) -> Self {
        let node_count = network.node_count();
        Self {
            source,
            sink,
            node_count,
            flows: network.edge_data_container(C::zero()),
            excess: network.node_data_container(C::zero()),
            labels: network.node_data_container(2 * node_count),
            current_edge: network.node_data_container(0),
        }
    }

    const fn unreachable_label(&self) -> usize {
        2 * self.node_count
    }

    /// Saturates all edges leaving the source.
    fn saturate_source_edges(
        &mut self,
        source_edges: &impl StackContainer<FlowEdge<G, C>>,
        active: &mut impl QueueContainer<G::NodeId>,
    ) {
        for index in 0..source_edges.len() {
            let edge = source_edges[index];
            if edge.source == self.source {
                self.push(&edge, edge.target, edge.data, active);
            }
        }
    }

    /// Pushes `amount` of flow along `edge` to `target` and activates `target` if it had no
    /// excess before. The excess of the pushing node has to be reduced by the caller.
    fn push(
        &mut self,
        edge: &FlowEdge<G, C>,
        target: G::NodeId,
        amount: C,
        active: &mut impl QueueContainer<G::NodeId>,
    ) {
        let flow = self.flows.get_mut(edge.id);
        *flow = adjusted_residual_flow::<G, _, C>(edge, target, *flow, amount);
        // The excess of the source is not needed and could overflow.
        if target != self.source {
            let excess = self.excess.get_mut(target);
            if *excess == C::zero() && target != self.sink {
                active.push_back(target);
            }
            *excess = *excess + amount;
        }
    }

    /// Pushes the excess of `node` to neighbors with a label smaller by one until either no
    /// excess or no such neighbor is left, in which case `node` is relabeled.
    ///
    /// Returns the work spent on relabeling, measured in scanned edges.
    fn discharge(
        &mut self,
        node: G::NodeId,
        edges: &impl StackContainer<FlowEdge<G, C>>,
        active: &mut impl QueueContainer<G::NodeId>,
    ) -> usize {
        let label = *self.labels.get(node);
        let mut excess = *self.excess.get(node);
        let mut index = *self.current_edge.get(node);
        while index < edges.len() {
            let edge = edges[index];
            let next = other_endpoint::<G, _>(&edge, node);
            let residual_cap = residual_capacity::<G, C>(edge, next, *self.flows.get(edge.id));
            if residual_cap > C::zero() && label == *self.labels.get(next) + 1 {
                let amount = excess.min_cmp(&residual_cap);
                self.push(&edge, next, amount, active);
                excess = excess - amount;
                if excess == C::zero() {
                    *self.excess.get_mut(node) = excess;
                    *self.current_edge.get_mut(node) = index;
                    return 0;
                }
            } else {
                index += 1;
            }
        }

        let (new_label, first_admissible) = self.min_residual_label(node, edges);
        *self.labels.get_mut(node) = new_label;
        *self.current_edge.get_mut(node) = first_admissible;
        if new_label < self.unreachable_label() {
            active.push_back(node);
        } else {
            // Nodes with excess can always reach the source, unless rounding errors of floating
            // point capacities left some excess behind, which is discarded.
            excess = C::zero();
        }
        *self.excess.get_mut(node) = excess;
        edges.len() + RELABEL_WORK
    }

    /// Returns the smallest label of a neighbor of `node` in the residual network plus one, capped
    /// at `2|V|`, and the index of the first edge to such a neighbor.
    fn min_residual_label(
        &self,
        node: G::NodeId,
        edges: &impl StackContainer<FlowEdge<G, C>>,
    ) -> (usize, usize) {
        let mut min_label = self.unreachable_label();
        let mut min_index = 0;
        for index in 0..edges.len() {
            let edge = edges[index];
            let next = other_endpoint::<G, _>(&edge, node);
            let label = *self.labels.get(next) + 1;
            if label < min_label
                && residual_capacity::<G, C>(edge, next, *self.flows.get(edge.id)) > C::zero()
            {
                min_label = label;
                min_index = index;
            }
        }
        (min_label, min_index)
    }

    /// Sets the label of each node to its distance to the sink in the residual network or, if the
    /// sink is unreachable, to `|V|` plus its distance to the source.
    fn global_relabel<SContainer>(
        &mut self,
        incident_edges: &G::NodeDataContainer<SContainer>,
        queue: &mut impl QueueContainer<G::NodeId>,
    ) where
        SContainer: StackContainer<FlowEdge<G, C>> + Clone,
    {
        let unreachable_label = self.unreachable_label();
        self.labels.clear();
        self.current_edge.clear();
        *self.labels.get_mut(self.sink) = 0;
        *self.labels.get_mut(self.source) = self.node_count;
        for root in [self.sink, self.source] {
            queue.push_back(root);
            while let Some(node) = queue.pop_front() {
                let label = *self.labels.get(node) + 1;
                let edges = incident_edges.get(node);
                for index in 0..edges.len() {
                    let edge = edges[index];
                    let previous = other_endpoint::<G, _>(&edge, node);
                    if *self.labels.get(previous) == unreachable_label
                        && residual_capacity::<G, C>(edge, node, *self.flows.get(edge.id))
                            > C::zero()
                    {
                        *self.labels.get_mut(previous) = label;
                        queue.push_back(previous);
                    }
                }
            }
        }
    }

    fn into_output(self) -> MaxFlowOutput<G, C> {
        MaxFlowOutput {
            max_flow: *self.excess.get(self.sink),
            flows: self.flows,
        }
    }
}
