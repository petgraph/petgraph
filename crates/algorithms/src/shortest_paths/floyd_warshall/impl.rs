use alloc::vec::Vec;

use error_stack::{Report, Result};
use num_traits::{CheckedAdd, Zero};
use petgraph_core::{base::MaybeOwned, id::LinearGraphId, Graph, GraphStorage, Node};

use crate::shortest_paths::{
    common::{cost::GraphCost, intermediates::Intermediates},
    floyd_warshall::{error::FloydWarshallError, matrix::SlotMatrix},
    Cost, Path, Route,
};

fn set_directed_distance<'graph, S, E>(
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

fn set_undirected_distance<'graph, S, E>(
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

    if v != u {
        matrix.set(v, u, value);
    }
}

fn set_directed_predecessor<'graph, S>(
    matrix: &mut SlotMatrix<'graph, S, &'graph S::NodeId>,
    u: &S::NodeId,
    v: &S::NodeId,
    value: Option<&'graph S::NodeId>,
) where
    S: GraphStorage,
    S::NodeId: LinearGraphId<S> + Clone,
{
    matrix.set(u, v, value);
}

fn set_undirected_predecessor<'graph, S>(
    matrix: &mut SlotMatrix<'graph, S, &'graph S::NodeId>,
    u: &S::NodeId,
    v: &S::NodeId,
    value: Option<&'graph S::NodeId>,
) where
    S: GraphStorage,
    S::NodeId: LinearGraphId<S> + Clone,
{
    matrix.set(u, v, value);

    if v != u {
        matrix.set(v, u, value);
    }
}

fn reconstruct_path<'a, S>(
    matrix: &SlotMatrix<'_, S, &'a S::NodeId>,
    source: &S::NodeId,
    target: &S::NodeId,
) -> Vec<&'a S::NodeId>
where
    S: GraphStorage,
    S::NodeId: LinearGraphId<S> + Clone,
{
    let mut path = Vec::new();

    if matrix.get(source, target).is_none() {
        return path;
    }

    let mut current = target;
    // we do not need the target node in the path, therefore don't initially push here

    while source != current {
        let Some(node) = matrix.get(source, current).copied() else {
            return Vec::new();
        };

        current = node;
        path.push(node);
    }

    path.reverse();
    path
}

struct FloydWarshallImpl<'graph: 'parent, 'parent, S, E>
where
    S: GraphStorage,
    S::NodeId: LinearGraphId<S>,
    E: GraphCost<S>,
{
    graph: &'graph Graph<S>,
    edge_cost: &'parent E,

    intermediates: Intermediates,

    set_distance: fn(
        &mut SlotMatrix<'graph, S, MaybeOwned<'graph, E::Value>>,
        &S::NodeId,
        &S::NodeId,
        Option<MaybeOwned<'graph, E::Value>>,
    ),
    set_predecessor: fn(
        &mut SlotMatrix<'graph, S, &'graph S::NodeId>,
        &S::NodeId,
        &S::NodeId,
        Option<&'graph S::NodeId>,
    ),

    distances: SlotMatrix<'graph, S, MaybeOwned<'graph, E::Value>>,
    predecessors: SlotMatrix<'graph, S, &'graph S::NodeId>,
}

impl<'graph: 'parent, 'parent, S, E> FloydWarshallImpl<'graph, 'parent, S, E>
where
    S: GraphStorage + 'static,
    S::NodeId: LinearGraphId<S> + Clone + Send + Sync + 'static,
    E: GraphCost<S>,
    E::Value: PartialOrd + CheckedAdd + Zero + Clone,
{
    fn new(
        graph: &'graph Graph<S>,

        edge_cost: &'parent E,

        intermediates: Intermediates,

        set_distance: fn(
            &mut SlotMatrix<'graph, S, MaybeOwned<'graph, E::Value>>,
            &S::NodeId,
            &S::NodeId,
            Option<MaybeOwned<'graph, E::Value>>,
        ),
        set_predecessor: fn(
            &mut SlotMatrix<'graph, S, &'graph S::NodeId>,
            &S::NodeId,
            &S::NodeId,
            Option<&'graph S::NodeId>,
        ),
    ) -> Result<Self, FloydWarshallError> {
        let distances = SlotMatrix::new(graph);

        let predecessors = match intermediates {
            Intermediates::Discard => SlotMatrix::empty(),
            Intermediates::Record => SlotMatrix::new(graph),
        };

        let mut this = Self {
            graph,
            edge_cost,

            intermediates,

            set_distance,
            set_predecessor,

            distances,
            predecessors,
        };

        this.eval()?;

        Ok(this)
    }

    fn eval(&mut self) -> Result<(), FloydWarshallError> {
        for edge in self.graph.edges() {
            let (u, v) = edge.endpoints();
            // in a directed graph we would need to assign to both
            // distance[(u, v)] and distance[(v, u)]
            (self.set_distance)(
                &mut self.distances,
                u.id(),
                v.id(),
                Some(self.edge_cost.cost(edge)),
            );

            if self.intermediates == Intermediates::Record {
                (self.set_predecessor)(&mut self.predecessors, u.id(), v.id(), Some(u.id()));
            }
        }

        for node in self.graph.nodes() {
            (self.set_distance)(
                &mut self.distances,
                node.id(),
                node.id(),
                Some(MaybeOwned::Owned(E::Value::zero())),
            );

            if self.intermediates == Intermediates::Record {
                (self.set_predecessor)(
                    &mut self.predecessors,
                    node.id(),
                    node.id(),
                    Some(node.id()),
                );
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

                    (self.set_distance)(
                        &mut self.distances,
                        i,
                        j,
                        Some(MaybeOwned::Owned(alternative)),
                    );
                    if self.intermediates == Intermediates::Record {
                        let predecessor = self.predecessors.get(k, j).copied();
                        (self.set_predecessor)(&mut self.predecessors, i, j, predecessor);
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

    fn iter(self) -> impl Iterator<Item = Route<'graph, S, E::Value>> + 'parent {
        let Self {
            graph,

            intermediates,

            distances,
            predecessors,
            ..
        } = self;

        graph
            .nodes()
            .flat_map(move |source| graph.nodes().map(move |target| (source, target)))
            .filter_map(move |(source, target)| {
                let path = match intermediates {
                    Intermediates::Discard => Path {
                        source,
                        target,
                        intermediates: Vec::new(),
                    },
                    Intermediates::Record => Path {
                        source,
                        target,
                        intermediates: reconstruct_path(&predecessors, source.id(), target.id())
                            .into_iter()
                            .filter_map(|id| graph.node(id))
                            .collect(),
                    },
                };

                let cost = distances
                    .get(source.id(), target.id())
                    .cloned()
                    .map(MaybeOwned::into_owned)?;

                Some(Route {
                    path,
                    cost: Cost(cost),
                })
            })
    }
}
