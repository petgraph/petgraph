pub struct GraphCollection<T, N, E> {
    pub graph: T,
    pub nodes: N,
    pub edges: E,
}

/// Utility macro to create an arbitrary graph.
///
/// TODO: redo documentation
///
/// Syntax: `graph!(<graph>; [<nodes>], [<edges>])`.
/// * Node: `<ident>: <attributes>`
/// * (Directed) Edge: `id: <source> -> <target>: <attributes>`
/// * (Undirected) Edge: `id: <source> -- <target>: <attributes>`
///
/// The output is a [`GraphCollection`], that contains all identifiers (be sure that the identifiers
/// are unique), which is instantiated.
///
/// These identifiers are collected through a newly created type in the macro `NodeCollection` and
/// `EdgeCollection`.
#[macro_export]
macro_rules! graph {
    (@collection: node $name:ident[]) => {
        #[allow(unreachable_pub)]
        pub struct $name;
    };

    (@collection: node $name:ident[$($id:ident : $attr:expr),* $(,)?]) => {
        #[allow(unreachable_pub)]
        pub struct $name {
            $(pub $id: petgraph_core::node::NodeId,)*
        }
    };

    (
        @collection: edge
        $name:ident[]
    ) => {
        #[allow(unreachable_pub)]
        pub struct $name;
    };

    (
        @collection: edge
        $name:ident[$($id:ident : $source:ident $(->)? $(--)? $target:ident : $attr:expr),* $(,)?]
    ) => {
        #[allow(unreachable_pub)]
        pub struct $name {
            $(pub $id: petgraph_core::edge::EdgeId,)*
        }
    };

    (
        @collection: edge
        $name:ident[$($id:ident : $source:ident $(->)? $(--)? $target:ident : @{$attr:expr}),* $(,)?]
    ) => {
        #[allow(unreachable_pub)]
        pub struct $name {
            $(pub $id: petgraph_core::edge::EdgeId,)*
        }
    };

    (
        @insert: node
        $graph:ident; $output:ident; $name:ident[$($id:ident : $attr:expr),* $(,)?]
    ) => {
        let $output = $name {
            $($id: $graph.insert_node($attr).id(),)*
        };
    };

    (
        @insert: edge
        $graph:ident; $nodes:ident; $source:ident; $target:ident; $attr:expr
    ) => {
        $graph.insert_edge($attr, $nodes.$source, $nodes.$target).id()
    };

    (
        @insert: edge
        $graph:ident; $nodes:ident; $source:ident; $target:ident; @{ $attr:expr }
    ) => {{
        let $source = $graph.node($nodes.$source).unwrap().weight();
        let $target = $graph.node($nodes.$target).unwrap().weight();

        $graph.insert_edge($attr, $nodes.$source, $nodes.$target).id()
    }};

    (
        @insert: edge
        $graph:ident; $nodes:ident; $output:ident; $name:ident[$($id:ident : $source:ident $(->)? $(--)? $target:ident : @{ $attr:expr }),* $(,)?]
    ) => {
        let $output = $name {
            $($id: $crate::graph!(@insert: edge $graph; $nodes; $source; $target; @{ $attr }),)*
        };
    };

    (
        @insert: edge
        $graph:ident; $nodes:ident; $output:ident; $name:ident[$($id:ident : $source:ident $(->)? $(--)? $target:ident : $attr:expr),* $(,)?]
    ) => {
        let $output = $name {
            $($id: $crate::graph!(@insert: edge $graph; $nodes; $source; $target; $attr),)*
        };
    };

    ($(#[$meta:meta])* $vis:vis factory($name:ident) => $graph:ty; [$($nodes:tt)*], [$($edges:tt)*]) => {
        $(#[$meta])*
        $vis mod $name {
            use super::*;

            $crate::graph!(@collection: node NodeCollection[$($nodes)*]);
            $crate::graph!(@collection: edge EdgeCollection[$($edges)*]);

            pub type Graph = $graph;

            pub fn create() -> $crate::GraphCollection<$graph, NodeCollection, EdgeCollection> {
                let mut graph = <$graph>::new();

                $crate::graph!(@insert: node graph; nodes; NodeCollection[$($nodes)*]);
                $crate::graph!(@insert: edge graph; nodes; edges; EdgeCollection[$($edges)*]);

                $crate::GraphCollection {
                    graph,
                    nodes,
                    edges,
                }
            }
        }
    };

    ($graph:ident; [$($nodes:tt)*],[$($edges:tt)*]) => {{
        $crate::graph!(@collection: node NodeCollection[$($nodes)*]);
        $crate::graph!(@collection: edge EdgeCollection[$($edges)*]);

        let mut graph = $graph::new();

        $crate::graph!(@insert: node graph; nodes; NodeCollection[$($nodes)*]);
        $crate::graph!(@insert: edge graph; nodes; edges; EdgeCollection[$($edges)*]);

        $crate::GraphCollection {
            graph,
            nodes,
            edges,
        }
    }};

    ($graph:ty; [$($nodes:tt)*],[$($edges:tt)*]) => {{
        $crate::graph!(@collection: node NodeCollection[$($nodes)*]);
        $crate::graph!(@collection: edge EdgeCollection[$($edges)*]);

        let mut graph = <$graph>::new();

        $crate::graph!(@insert: node graph; nodes; NodeCollection[$($nodes)*]);
        $crate::graph!(@insert: edge graph; nodes; edges; EdgeCollection[$($edges)*]);

        $crate::GraphCollection {
            graph,
            nodes,
            edges,
        }
    }};
}

#[cfg(test)]
mod tests {
    use petgraph_dino::DiDinoGraph;

    #[test]
    fn simple() {
        let collection = graph!(DiDinoGraph; [a: 1, b: 2], [ab: a -> b: 3]);
    }
}
