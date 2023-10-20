mod error;
mod r#impl;

use petgraph_core::{
    edge::marker::{Directed, Undirected},
    GraphDirectionality,
};

pub struct AStar<D, E, H>
where
    D: GraphDirectionality,
{
    direction: D,

    edge_cost: E,
    heuristic: H,
}

impl AStar<Directed, (), ()> {
    pub fn directed() -> Self {
        Self {
            direction: Directed,
            edge_cost: (),
            heuristic: (),
        }
    }
}

impl AStar<Undirected, (), ()> {
    pub fn undirected() -> Self {
        Self {
            direction: Undirected,
            edge_cost: (),
            heuristic: (),
        }
    }
}

impl<D, E, H> AStar<D, E, H>
where
    D: GraphDirectionality,
{
    pub fn with_edge_cost<E2>(self, edge_cost: E2) -> AStar<D, E2, H> {
        AStar {
            direction: self.direction,
            edge_cost,
            heuristic: self.heuristic,
        }
    }

    pub fn without_edge_cost(self) -> AStar<D, (), H> {
        AStar {
            direction: self.direction,
            edge_cost: (),
            heuristic: self.heuristic,
        }
    }

    pub fn with_heuristic<H2>(self, heuristic: H2) -> AStar<D, E, H2> {
        AStar {
            direction: self.direction,
            edge_cost: self.edge_cost,
            heuristic,
        }
    }
}
