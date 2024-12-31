use alloc::vec::Vec;

use error_stack::{Report, Result};
use numi::{
    borrow::Moo,
    num::{checked::CheckedAdd, identity::Zero},
};
use petgraph_core::{Graph, Node, graph::SequentialGraphStorage, node::NodeId};

use crate::shortest_paths::{
    Path,
    common::{
        cost::{Cost, GraphCost},
        route::Route,
        transit::PredecessorMode,
    },
    floyd_warshall::{
        FloydWarshallMeasure, NegativeCycle, error::FloydWarshallError, matrix::SlotMatrix,
    },
};

pub(super) fn init_directed_edge_distance<'graph: 'this, 'this, S, E>(
    matrix: &mut SlotMatrix<'graph, S, Moo<'this, E::Value>>,
    u: NodeId,
    v: NodeId,
    value: Option<Moo<'this, E::Value>>,
) where
    S: Graph,
    E: GraphCost<S>,
    E::Value: Clone,
{
    matrix.set(u, v, value);
}

pub(super) fn init_undirected_edge_distance<'graph: 'this, 'this, S, E>(
    matrix: &mut SlotMatrix<'graph, S, Moo<'this, E::Value>>,
    u: NodeId,
    v: NodeId,
    value: Option<Moo<'this, E::Value>>,
) where
    S: Graph,
    E: GraphCost<S>,
    E::Value: Clone,
{
    matrix.set(u, v, value.clone());

    if u != v {
        matrix.set(v, u, value);
    }
}

pub(super) fn init_directed_edge_predecessor<S>(
    matrix: &mut SlotMatrix<S, NodeId>,
    u: NodeId,
    v: NodeId,
) where
    S: Graph,
{
    matrix.set(u, v, Some(u));
}

pub(super) fn init_undirected_edge_predecessor<S>(
    matrix: &mut SlotMatrix<S, NodeId>,
    u: NodeId,
    v: NodeId,
) where
    S: Graph,
{
    matrix.set(u, v, Some(u));
    matrix.set(v, u, Some(v));
}

fn reconstruct_path<S>(
    matrix: &SlotMatrix<'_, S, NodeId>,
    source: NodeId,
    target: NodeId,
) -> Vec<NodeId>
where
    S: Graph,
{
    let mut path = Vec::new();

    if source == target {
        return Vec::new();
    }

    if matrix.get(source, target).is_none() {
        return Vec::new();
    }

    let mut current = target;

    // eager loop termination here, so that we don't need to push and then pop the last element
    // again.
    loop {
        let Some(node) = matrix.get(source, current).copied() else {
            return Vec::new();
        };

        if node == source {
            break;
        }

        current = node;
        path.push(node);
    }

    path.reverse();
    path
}
type InitEdgeDistanceFn<'graph, 'this, S, E> = fn(
    &mut SlotMatrix<'graph, S, Moo<'this, <E as GraphCost<S>>::Value>>,
    NodeId,
    NodeId,
    Option<Moo<'this, <E as GraphCost<S>>::Value>>,
);

type InitEdgePredecessorFn<'graph, S> = fn(&mut SlotMatrix<'graph, S, NodeId>, NodeId, NodeId);

pub(super) struct FloydWarshallImpl<'graph: 'parent, 'parent, S, E>
where
    S: Graph,
    E: GraphCost<S>,
{
    graph: &'graph Graph<S>,
    edge_cost: &'parent E,

    predecessor_mode: PredecessorMode,

    init_edge_distance: InitEdgeDistanceFn<'graph, 'parent, S, E>,
    init_edge_predecessor: InitEdgePredecessorFn<'graph, S>,

    distances: SlotMatrix<'graph, S, Moo<'parent, E::Value>>,
    predecessors: SlotMatrix<'graph, S, NodeId>,
}

impl<'graph: 'parent, 'parent, S, E> FloydWarshallImpl<'graph, 'parent, S, E>
where
    S: Graph,
    E: GraphCost<S>,
    E::Value: FloydWarshallMeasure,
{
    pub(super) fn new(
        graph: &'graph Graph<S>,

        edge_cost: &'parent E,

        predecessor_mode: PredecessorMode,

        init_edge_distance: InitEdgeDistanceFn<'graph, 'parent, S, E>,
        init_edge_predecessor: InitEdgePredecessorFn<'graph, S>,
    ) -> Result<Self, FloydWarshallError> {
        let distances = SlotMatrix::new(graph);

        let predecessors = match predecessor_mode {
            PredecessorMode::Discard => SlotMatrix::empty(),
            PredecessorMode::Record => SlotMatrix::new(graph),
        };

        let mut this = Self {
            graph,
            edge_cost,

            predecessor_mode,

            init_edge_distance,
            init_edge_predecessor,

            distances,
            predecessors,
        };

        this.eval()?;

        Ok(this)
    }

    fn eval(&mut self) -> Result<(), FloydWarshallError> {
        for edge in self.graph.edges() {
            let (u, v) = edge.endpoints();

            (self.init_edge_distance)(
                &mut self.distances,
                u.id(),
                v.id(),
                Some(self.edge_cost.cost(edge)),
            );

            if self.predecessor_mode == PredecessorMode::Record {
                (self.init_edge_predecessor)(&mut self.predecessors, u.id(), v.id());
            }
        }

        for node in self.graph.nodes() {
            self.distances
                .set(node.id(), node.id(), Some(Moo::Owned(E::Value::zero())));

            if self.predecessor_mode == PredecessorMode::Record {
                self.predecessors.set(node.id(), node.id(), Some(node.id()));
            }
        }

        for k in self.graph.nodes() {
            let k = k.id();

            for i in self.graph.nodes() {
                let i = i.id();

                for j in self.graph.nodes() {
                    let j = j.id();

                    let Some(ik) = self.distances.get(i, k) else {
                        continue;
                    };

                    let Some(kj) = self.distances.get(k, j) else {
                        continue;
                    };

                    // Floyd-Warshall has a tendency to overflow on negative cycles, as large as
                    // `Ω(⋅6^{n-1} w_max)`.
                    // Where `w_max` is the largest absolute value of a negative edge weight.
                    // see: https://doi.org/10.1016/j.ipl.2010.02.001
                    let Some(alternative) = ik.as_ref().checked_add(kj.as_ref()) else {
                        continue;
                    };

                    if let Some(current) = self.distances.get(i, j) {
                        if alternative >= *current.as_ref() {
                            continue;
                        }
                    }

                    // TODO: check for diagonal here and apply suggestion from paper

                    self.distances.set(i, j, Some(Moo::Owned(alternative)));

                    if self.predecessor_mode == PredecessorMode::Record {
                        let predecessor = self.predecessors.get(k, j).copied();
                        self.predecessors.set(i, j, predecessor);
                    }
                }
            }
        }

        let negative_cycles = self
            .distances
            .diagonal()
            .enumerate()
            .filter_map(|(index, value)| value.map(|value| (index, value)))
            .filter(|(_, value)| *value.as_ref() < E::Value::zero())
            .map(|(index, _)| index);

        let mut result: Result<(), FloydWarshallError> = Ok(());

        for index in negative_cycles {
            let Some(node) = self.distances.resolve(index) else {
                continue;
            };

            let cycle = NegativeCycle::new(node);

            result = match result {
                Ok(()) => Err(Report::new(FloydWarshallError::NegativeCycle).attach(cycle)),
                Err(report) => Err(report.attach(cycle)),
            };
        }

        result
    }

    pub(super) fn filter(
        self,
        mut filter: impl FnMut(Node<'graph, S>, Node<'graph, S>) -> bool + 'parent,
    ) -> impl Iterator<Item = Route<'graph, S, E::Value>> + 'parent {
        let Self {
            graph,
            predecessor_mode,

            distances,
            predecessors,
            ..
        } = self;

        graph
            .nodes()
            .flat_map(move |source| graph.nodes().map(move |target| (source, target)))
            .filter(move |(source, target)| filter(*source, *target))
            .filter_map(move |(source, target)| {
                // filter out before so that we don't have to reconstruct the path for
                // unreachable nodes
                distances
                    .get(source.id(), target.id())
                    .map(|cost| (source, target, cost.clone().into_owned()))
            })
            .map(move |(source, target, cost)| {
                let transit = match predecessor_mode {
                    PredecessorMode::Discard => Vec::new(),
                    PredecessorMode::Record => {
                        reconstruct_path(&predecessors, source.id(), target.id())
                            .into_iter()
                            .filter_map(|id| graph.node(id))
                            .collect()
                    }
                };

                Route::new(Path::new(source, transit, target), Cost::new(cost))
            })
    }

    pub(super) fn pick(self, source: NodeId, target: NodeId) -> Option<Route<'graph, S, E::Value>> {
        let Self {
            graph,
            distances,
            predecessors,
            predecessor_mode,
            ..
        } = self;

        let source_node = graph.node(source)?;
        let target_node = graph.node(target)?;

        let cost = distances.get(source, target)?;
        let transit = match predecessor_mode {
            PredecessorMode::Discard => Vec::new(),
            PredecessorMode::Record => reconstruct_path(&predecessors, source, target)
                .into_iter()
                .filter_map(|id| graph.node(id))
                .collect(),
        };

        let cost = Cost::new(cost.clone().into_owned());

        Some(Route::new(
            Path::new(source_node, transit, target_node),
            cost,
        ))
    }
}
