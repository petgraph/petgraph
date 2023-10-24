mod r#impl;
mod matrix;

use core::{marker::PhantomData, ops::Add};

use num_traits::{CheckedAdd, Zero};
use petgraph_core::{
    edge::marker::{Directed, Undirected},
    id::LinearGraphId,
    Graph, GraphStorage,
};

use crate::shortest_paths::{
    common::cost::{DefaultCost, GraphCost},
    floyd_warshall::matrix::Matrix,
};

struct FloydWarshall<D, E> {
    edge_cost: E,

    direction: PhantomData<fn() -> *const D>,
}

impl FloydWarshall<Directed, DefaultCost> {
    fn directed() -> Self {
        Self {
            edge_cost: DefaultCost,
            direction: PhantomData,
        }
    }
}

impl FloydWarshall<Undirected, DefaultCost> {
    fn undirected() -> Self {
        Self {
            edge_cost: DefaultCost,
            direction: PhantomData,
        }
    }
}

impl<D, E> FloydWarshall<D, E> {
    fn with_edge_cost<S, F>(self, edge_cost: F) -> FloydWarshall<D, F>
    where
        F: GraphCost<S>,
    {
        FloydWarshall {
            edge_cost,
            direction: self.direction,
        }
    }
}

fn eval<S>(graph: &Graph<S>)
where
    S: GraphStorage,
    S::NodeId: LinearGraphId<S>,
    S::EdgeWeight: CheckedAdd + Zero,
    for<'a> &'a S::EdgeWeight: Add<Output = S::EdgeWeight>,
{
    let mut distance = Matrix::new_from_option(graph);
    let mut previous = Matrix::new_from_default(graph);

    // TODO: GraphCost

    for edge in graph.edges() {
        let (u, v) = edge.endpoints();
        // in a directed graph we would need to assign to both
        // distance[(u, v)] and distance[(v, u)]
        distance.set(u.id(), v.id(), Some(edge.weight()));
        // distance.set(v.id(), u.id(), Some(edge.weight()))

        previous.set(u.id(), v.id(), Some(u.id()))
        // previous.set(v.id(), u.id(), Some(v.id()))
    }

    for node in graph.nodes() {
        distance.set(node.id(), node.id(), Some(S::EdgeWeight::zero()));
        previous.set(node.id(), node.id(), Some(node.id()));
    }

    for k in graph.nodes() {
        let k = k.id();

        for i in graph.nodes() {
            let i = i.id();

            for j in graph.nodes() {
                let j = j.id();

                let Some(ik) = distance.get(i, k) else {
                    continue;
                };

                let Some(kj) = distance.get(k, j) else {
                    continue;
                };

                // Floyd-Warshall has a tendency to overflow on negative cycles, as large as
                // `Ω(⋅6^{n-1} w_max)`.
                // Where `w_max` is the largest absolute value of a negative edge weight.
                // see: https://doi.org/10.1016/j.ipl.2010.02.001
                let Some(alternative) = ik.checked_add(*kj) else {
                    continue;
                };

                if let Some(Some(current)) = distance.get(i, j) {
                    if alternative >= *current {
                        continue;
                    }
                }

                distance.set(i, j, Some(alternative));
                // distance.set(j, i, Some(alternative));
                previous.set(i, j, *previous.get(k, j));
                // previous.set(j, i, *previous.get(k, i));
            }
        }
    }

    if distance.diagonal().any(|d| d < S::EdgeWeight::zero()) {
        todo!("negative cycle")
    }
}
