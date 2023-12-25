use core::marker::PhantomData;

use numi::borrow::Moo;
use petgraph_core::{Edge, GraphStorage, Node};

use crate::shortest_paths::GraphCost;

struct GraphCostClosure<F, S, T> {
    closure: F,
    _marker: PhantomData<fn() -> *const (S, T)>,
}

impl<S, T> GraphCostClosure<(), S, T> {
    pub fn new(
        closure: impl Fn(Edge<S>) -> Moo<T>,
    ) -> GraphCostClosure<impl Fn(Edge<S>) -> Moo<T>, S, T> {
        GraphCostClosure {
            closure,
            _marker: PhantomData,
        }
    }
}

impl<F, S, T> GraphCostClosure<F, S, T>
where
    F: Fn(Edge<S>) -> Moo<T>,
{
    pub fn narrow_node_weight<N>(self) -> GraphCostClosure<impl Fn(Edge<S>) -> Moo<T>, S, T>
    where
        S: GraphStorage<NodeWeight = N>,
    {
        GraphCostClosure {
            closure: self.closure,
            _marker: PhantomData,
        }
    }

    pub fn narrow_edge_weight<E>(self) -> GraphCostClosure<impl Fn(Edge<S>) -> Moo<T>, S, T>
    where
        S: GraphStorage<EdgeWeight = E>,
    {
        GraphCostClosure {
            closure: self.closure,
            _marker: PhantomData,
        }
    }
}

impl<F, S, T> GraphCost<S> for GraphCostClosure<F, S, T>
where
    F: Fn(Edge<S>) -> Moo<T>,
    S: GraphStorage,
{
    type Value = T;

    fn cost<'graph>(&self, edge: Edge<'graph, S>) -> Moo<'graph, Self::Value> {
        (self.closure)(edge)
    }
}

#[cfg(test)]
mod test {
    use numi::borrow::Moo;
    use petgraph_core::Graph;

    use crate::shortest_paths::{common::closures::GraphCostClosure, Dijkstra};

    #[test]
    fn cost() {
        let closure = GraphCostClosure::new(|edge| Moo::Borrowed(edge.weight()))
            .narrow_edge_weight::<i32>()
            .narrow_node_weight::<i32>();

        let closure = Dijkstra::undirected().with_edge_cost(closure);
    }
}

// pub fn bind_heuristic<S, T>(
//     closure: impl Fn(Node<S>, Node<S>) -> Moo<T>,
// ) -> impl Fn(Node<S>, Node<S>) -> Moo<T> {
//     closure
// }
