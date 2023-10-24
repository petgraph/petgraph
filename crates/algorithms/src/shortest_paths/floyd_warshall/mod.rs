mod error;
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
    floyd_warshall::matrix::SlotMatrix,
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
        S: GraphStorage,
        F: GraphCost<S>,
    {
        FloydWarshall {
            edge_cost,
            direction: self.direction,
        }
    }
}
