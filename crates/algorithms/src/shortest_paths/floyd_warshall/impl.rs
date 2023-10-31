use alloc::vec::Vec;

use error_stack::{Report, Result};
use num_traits::{CheckedAdd, Zero};
use petgraph_core::{base::MaybeOwned, id::LinearGraphId, Graph, GraphStorage, Node};

use crate::shortest_paths::{
    common::{cost::GraphCost, transit::PredecessorMode},
    floyd_warshall::{error::FloydWarshallError, matrix::SlotMatrix},
    Cost, Path, Route,
};

pub(super) fn init_directed_edge_distance<'graph, S, E>(
    matrix: &mut SlotMatrix<'graph, S, MaybeOwned<'graph, E::Value>>,
    u: &S::NodeId,
    v: &S::NodeId,
    value: Option<MaybeOwned<'graph, E::Value>>,
) where
    S: GraphStorage,
    S::NodeId: LinearGraphId<S> + Clone,
    E: GraphCost<S>,
    E::Value: Clone,
{
    matrix.set(u, v, value);
}

pub(super) fn init_undirected_edge_distance<'graph, S, E>(
    matrix: &mut SlotMatrix<'graph, S, MaybeOwned<'graph, E::Value>>,
    u: &S::NodeId,
    v: &S::NodeId,
    value: Option<MaybeOwned<'graph, E::Value>>,
) where
    S: GraphStorage,
    S::NodeId: LinearGraphId<S> + Clone,
    E: GraphCost<S>,
    E::Value: Clone,
{
    matrix.set(u, v, value.clone());

    if u != v {
        matrix.set(v, u, value);
    }
}

pub(super) fn init_directed_edge_predecessor<'graph, S>(
    matrix: &mut SlotMatrix<'graph, S, &'graph S::NodeId>,
    u: &'graph S::NodeId,
    v: &'graph S::NodeId,
) where
    S: GraphStorage,
    S::NodeId: LinearGraphId<S> + Clone,
{
    matrix.set(u, v, Some(u));
}

pub(super) fn init_undirected_edge_predecessor<'graph, S>(
    matrix: &mut SlotMatrix<'graph, S, &'graph S::NodeId>,
    u: &'graph S::NodeId,
    v: &'graph S::NodeId,
) where
    S: GraphStorage,
    S::NodeId: LinearGraphId<S> + Clone,
{
    matrix.set(u, v, Some(u));
    matrix.set(v, u, Some(v));
}

fn reconstruct_path<'a, S>(
    matrix: &SlotMatrix<'_, S, &'a S::NodeId>,
    source: &S::NodeId,
    target: &'a S::NodeId,
) -> Vec<&'a S::NodeId>
where
    S: GraphStorage,
    S::NodeId: LinearGraphId<S> + Clone,
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

type InitEdgeDistanceFn<'graph, S, E> = fn(
    &mut SlotMatrix<'graph, S, MaybeOwned<'graph, <E as GraphCost<S>>::Value>>,
    &<S as GraphStorage>::NodeId,
    &<S as GraphStorage>::NodeId,
    Option<MaybeOwned<'graph, <E as GraphCost<S>>::Value>>,
);

type InitEdgePredecessorFn<'graph, S> = fn(
    &mut SlotMatrix<'graph, S, &'graph <S as GraphStorage>::NodeId>,
    &'graph <S as GraphStorage>::NodeId,
    &'graph <S as GraphStorage>::NodeId,
);

pub(super) struct FloydWarshallImpl<'graph: 'parent, 'parent, S, E>
where
    S: GraphStorage,
    S::NodeId: LinearGraphId<S>,
    E: GraphCost<S>,
{
    graph: &'graph Graph<S>,
    edge_cost: &'parent E,

    predecessor_mode: PredecessorMode,

    init_edge_distance: InitEdgeDistanceFn<'graph, S, E>,
    init_edge_predecessor: InitEdgePredecessorFn<'graph, S>,

    distances: SlotMatrix<'graph, S, MaybeOwned<'graph, E::Value>>,
    predecessors: SlotMatrix<'graph, S, &'graph S::NodeId>,
}

impl<'graph: 'parent, 'parent, S, E> FloydWarshallImpl<'graph, 'parent, S, E>
where
    S: GraphStorage,
    S::NodeId: LinearGraphId<S> + Clone + Send + Sync + 'static,
    E: GraphCost<S>,
    E::Value: PartialOrd + CheckedAdd + Zero + Clone + 'graph,
{
    pub(super) fn new(
        graph: &'graph Graph<S>,

        edge_cost: &'parent E,

        predecessor_mode: PredecessorMode,

        init_edge_distance: InitEdgeDistanceFn<'graph, S, E>,
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
            self.distances.set(
                node.id(),
                node.id(),
                Some(MaybeOwned::Owned(E::Value::zero())),
            );

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

                    self.distances
                        .set(i, j, Some(MaybeOwned::Owned(alternative)));

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
            let Some(node) = self.distances.resolve(index).map(MaybeOwned::into_owned) else {
                continue;
            };

            result = match result {
                Ok(()) => Err(Report::new(FloydWarshallError::NegativeCycle).attach(node)),
                Err(report) => Err(report.attach(node)),
            };
        }

        result
    }

    pub(super) fn filter(
        self,
        mut filter: impl FnMut(Node<S>, Node<S>) -> bool + 'parent,
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
                let path = match predecessor_mode {
                    PredecessorMode::Discard => Path {
                        source,
                        target,
                        transit: Vec::new(),
                    },
                    PredecessorMode::Record => Path {
                        source,
                        target,
                        transit: reconstruct_path(&predecessors, source.id(), target.id())
                            .into_iter()
                            .filter_map(|id| graph.node(id))
                            .collect(),
                    },
                };

                Route {
                    path,
                    cost: Cost(cost),
                }
            })
    }
}
