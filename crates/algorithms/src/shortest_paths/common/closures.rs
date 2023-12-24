use core::marker::PhantomData;

use petgraph_core::{base::MaybeOwned, Edge, GraphStorage, Node};

struct GraphCostClosure<F, S, T> {
    closure: F,
    _marker: PhantomData<fn() -> *const (S, T)>,
}

impl<S, T> GraphCostClosure<(), S, T> {
    pub fn new(
        closure: impl Fn(Edge<S>) -> MaybeOwned<T>,
    ) -> GraphCostClosure<impl Fn(Edge<S>) -> MaybeOwned<T>, S, T> {
        GraphCostClosure {
            closure,
            _marker: PhantomData,
        }
    }
}

impl<F, S, T> GraphCostClosure<F, S, T>
where
    F: Fn(Edge<S>) -> MaybeOwned<T>,
{
    pub fn narrow_node_weight<N>(self) -> GraphCostClosure<impl Fn(Edge<S>) -> MaybeOwned<T>, S, T>
    where
        S: GraphStorage<NodeWeight = N>,
    {
        GraphCostClosure {
            closure: self.closure,
            _marker: PhantomData,
        }
    }

    pub fn narrow_edge_weight<E>(self) -> GraphCostClosure<impl Fn(Edge<S>) -> MaybeOwned<T>, S, T>
    where
        S: GraphStorage<EdgeWeight = E>,
    {
        GraphCostClosure {
            closure: self.closure,
            _marker: PhantomData,
        }
    }
}

#[cfg(test)]
mod test {
    use petgraph_core::{base::MaybeOwned, Graph};

    use crate::shortest_paths::{common::closures::GraphCostClosure, Dijkstra};

    #[test]
    fn cost() {
        let closure = GraphCostClosure::new(|edge| MaybeOwned::Borrowed(edge.weight()))
            .narrow_edge_weight::<i32>()
            .narrow_node_weight::<i32>();

        let closure = Dijkstra::undirected().with_edge_cost(closure);
    }
}

// pub fn bind_heuristic<S, T>(
//     closure: impl Fn(Node<S>, Node<S>) -> MaybeOwned<T>,
// ) -> impl Fn(Node<S>, Node<S>) -> MaybeOwned<T> {
//     closure
// }
