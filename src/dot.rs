//! Simple graphviz dot file format output.
// TODO: potentially move this into petgraph-io or petgraph-dot (io for different formats)

use std::fmt::{self, Debug, Display, Formatter, Write};

use dot::{Edges, Id, LabelText, Nodes, RenderOption};

use crate::visit::{
    EdgeRef, GraphProp, IntoEdgeReferences, IntoNodeReferences, NodeIndexable, NodeRef,
};

/// `Dot` implements output to graphviz .dot format for a graph.
///
/// Formatting and options are rather simple, this is mostly intended
/// for debugging. Exact output may change.
///
/// # Examples
///
/// ```
/// use petgraph::{
///     dot::{Config, Dot},
///     Graph,
/// };
///
/// let mut graph = Graph::<_, ()>::new();
/// graph.add_node("A");
/// graph.add_node("B");
/// graph.add_node("C");
/// graph.add_node("D");
/// graph.extend_with_edges(&[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)]);
///
/// println!("{:?}", Dot::with_config(&graph, &[Config::EdgeNoLabel]));
///
/// // In this case the output looks like this:
/// //
/// // digraph {
/// //     0 [label="\"A\""]
/// //     1 [label="\"B\""]
/// //     2 [label="\"C\""]
/// //     3 [label="\"D\""]
/// //     0 -> 1
/// //     0 -> 2
/// //     0 -> 3
/// //     1 -> 2
/// //     1 -> 3
/// //     2 -> 3
/// // }
///
/// // If you need multiple config options, just list them all in the slice.
/// ```
pub struct Dot<'a, G>
where
    G: IntoEdgeReferences + IntoNodeReferences,
{
    graph: G,
    get_edge_attributes: &'a dyn Fn(G, G::EdgeRef) -> String,
    get_node_attributes: &'a dyn Fn(G, G::NodeRef) -> String,
    options: &'a [RenderOption],
}

static TYPE: [&str; 2] = ["graph", "digraph"];
static EDGE: [&str; 2] = ["--", "->"];
static INDENT: &str = "    ";

impl<'a, G> Dot<'a, G>
where
    G: IntoNodeReferences + IntoEdgeReferences,
{
    /// Create a `Dot` formatting wrapper with default configuration.
    #[inline]
    pub fn new(graph: G) -> Self {
        Self::with_config(graph, &[])
    }

    /// Create a `Dot` formatting wrapper with custom configuration.
    #[inline]
    pub fn with_config(graph: G, options: &'a [RenderOption]) -> Self {
        Self::with_attr_getters(
            graph, //
            &options,
            &|_, _| String::new(),
            &|_, _| String::new(),
        )
    }

    #[inline]
    pub fn with_attr_getters(
        graph: G,
        options: &'a [RenderOption],

        get_edge_attributes: &'a dyn Fn(G, G::EdgeRef) -> String,
        get_node_attributes: &'a dyn Fn(G, G::NodeRef) -> String,
    ) -> Self {
        Self {
            graph,
            options,

            get_edge_attributes,
            get_node_attributes,
        }
    }
}

impl<'a, G> dot::Labeller<'a, G::NodeId, G::EdgeRef> for Dot<'a, G>
where
    G: IntoNodeReferences + IntoEdgeReferences,
{
    fn graph_id(&'a self) -> Id<'a> {
        // TODO: make configurable
        Id::new("petgraph").expect("infallible")
    }

    fn node_id(&'a self, n: &G::NodeId) -> Id<'a> {
        Id::new(format!("N{}", n.as_usize())).expect("infallible")
    }

    fn node_label(&'a self, n: &G::NodeId) -> LabelText<'a> {
        // TODO: this needs improvement, as it is incomplete!
        let label = (self.get_node_attributes)(self.graph, self.graph.node_reference(*n));

        LabelText::LabelStr(label.into())
    }

    fn edge_label(&'a self, e: &G::EdgeRef) -> LabelText<'a> {
        let label = (self.get_edge_attributes)(self.graph, self.graph.edge_reference(e.id()));

        LabelText::LabelStr(label.into())
    }
}

impl<'a, G> dot::GraphWalk<'a, G::NodeId, G::EdgeRef> for Dot<'a, G>
where
    G: IntoNodeReferences + IntoEdgeReferences,
{
    fn nodes(&'a self) -> Nodes<'a, G::NodeId> {
        Nodes::Owned(self.graph.node_identifiers().collect())
    }

    fn edges(&'a self) -> Edges<'a, G::EdgeRef> {
        Edges::Owned(self.graph.edge_references().collect())
    }

    fn source(&'a self, edge: &G::EdgeRef) -> G::NodeId {
        edge.source()
    }

    fn target(&'a self, edge: &G::EdgeRef) -> G::NodeId {
        edge.target()
    }
}

impl<'a, G> Display for Dot<'a, G>
where
    G: IntoNodeReferences + IntoEdgeReferences,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        dot::render_opts(self, f, self.options).map_err(|_| fmt::Error)
    }
}

impl<'a, G> Debug for Dot<'a, G>
where
    G: IntoNodeReferences + IntoEdgeReferences,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        dot::render_opts(self, f, self.options).map_err(|_| fmt::Error)
    }
}

#[cfg(test)]
mod test {
    use std::fmt::Write;

    use dot::RenderOption;

    use super::{Config, Dot, Escaper};
    use crate::{prelude::Graph, visit::NodeRef};

    fn simple_graph() -> Graph<&'static str, &'static str> {
        let mut graph = Graph::<&str, &str>::new();
        let a = graph.add_node("A");
        let b = graph.add_node("B");
        graph.add_edge(a, b, "edge_label");
        graph
    }

    #[test]
    fn test_nodeindexlable_option() {
        let graph = simple_graph();
        let dot = format!("{:?}", Dot::with_config(&graph, &[]));
        assert_eq!(
            dot,
            "digraph {\n    0 [ label = \"0\" ]\n    1 [ label = \"1\" ]\n    0 -> 1 [ label = \
             \"\\\"edge_label\\\"\" ]\n}\n"
        );
    }

    #[test]
    fn test_edgeindexlable_option() {
        let graph = simple_graph();
        let dot = format!("{:?}", Dot::with_config(&graph, &[]));
        assert_eq!(
            dot,
            "digraph {\n    0 [ label = \"\\\"A\\\"\" ]\n    1 [ label = \"\\\"B\\\"\" ]\n    0 \
             -> 1 [ label = \"0\" ]\n}\n"
        );
    }

    #[test]
    fn test_edgenolable_option() {
        let graph = simple_graph();
        let dot = format!(
            "{:?}",
            Dot::with_config(&graph, &[RenderOption::NoEdgeLabels])
        );
        assert_eq!(
            dot,
            "digraph {\n    0 [ label = \"\\\"A\\\"\" ]\n    1 [ label = \"\\\"B\\\"\" ]\n    0 \
             -> 1 [ ]\n}\n"
        );
    }

    #[test]
    fn test_nodenolable_option() {
        let graph = simple_graph();
        let dot = format!(
            "{:?}",
            Dot::with_config(&graph, &[RenderOption::NoNodeLabels])
        );
        assert_eq!(
            dot,
            "digraph {\n    0 [ ]\n    1 [ ]\n    0 -> 1 [ label = \"\\\"edge_label\\\"\" ]\n}\n"
        );
    }

    #[test]
    fn test_with_attr_getters() {
        let graph = simple_graph();
        let dot = format!(
            "{:?}",
            Dot::with_attr_getters(
                &graph,
                &[RenderOption::NoNodeLabels, RenderOption::NoEdgeLabels],
                &|_, er| format!("label = \"{}\"", er.weight().to_uppercase()),
                &|_, nr| format!("label = \"{}\"", nr.weight().to_lowercase()),
            ),
        );
        assert_eq!(
            dot,
            "digraph {\n    0 [ label = \"a\"]\n    1 [ label = \"b\"]\n    0 -> 1 [ label = \
             \"EDGE_LABEL\"]\n}\n"
        );
    }
}
