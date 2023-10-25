mod error;
mod r#impl;
mod matrix;

use core::marker::PhantomData;

use petgraph_core::{
    edge::marker::{Directed, Undirected},
    GraphStorage,
};

use crate::shortest_paths::common::cost::{DefaultCost, GraphCost};

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
