use numi::borrow::Moo;
use petgraph_core::{Edge, GraphStorage, Node};

pub fn cost<S, T>(closure: impl Fn(Edge<S>) -> Moo<T>) -> impl Fn(Edge<S>) -> Moo<T> {
    closure
}

pub fn heuristic<S, T>(
    closure: impl for<'graph> Fn(Node<'graph, S>, Node<'graph, S>) -> Moo<'graph, T>,
) -> impl for<'graph> Fn(Node<'graph, S>, Node<'graph, S>) -> Moo<'graph, T>
where
    S: GraphStorage,
{
    closure
}

#[cfg(test)]
mod test {
    use numi::borrow::Moo;
    use petgraph_dino::DiDinoGraph;

    use crate::shortest_paths::{
        common::closures::{cost, heuristic},
        AStar, Dijkstra, ShortestPath,
    };

    #[test]
    fn bind_cost() {
        let closure = cost(|edge| Moo::Borrowed(edge.weight()));

        let algorithm = Dijkstra::undirected().with_edge_cost(closure);

        let mut graph = DiDinoGraph::new();
        let a = graph.insert_node("A").id();
        let b = graph.insert_node("B").id();

        graph.insert_edge(7, a, b);

        let _path = algorithm.path_between(&graph, a, b).expect("path exists");
    }

    #[test]
    fn bind_heuristic() {
        let closure = heuristic(|_source, _target| Moo::Owned(0i32));

        let algorithm = AStar::undirected().with_heuristic(closure);

        let mut graph = DiDinoGraph::new();
        let a = graph.insert_node("A").id();
        let b = graph.insert_node("B").id();

        graph.insert_edge(7i32, a, b);

        let _path = algorithm.path_between(&graph, a, b).expect("path exists");
    }
}
