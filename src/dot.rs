//! Simple graphviz dot file format output.
// TODO: potentially move this into petgraph-io or petgraph-dot (io for different formats)

use std::{
    borrow::Cow,
    collections::HashMap,
    fmt::{self, Debug, Display, Formatter, Write},
    sync::{Arc, RwLock},
};

pub use dot::RenderOption;
use dot::{Arrow, Edges, Id, Kind, LabelText, Nodes, Style};
use petgraph_core::{edge::EdgeType, visit::IntoNodeIdentifiers};

use crate::visit::{
    EdgeRef, GraphProp, IntoEdgeReferences, IntoNodeReferences, NodeIndexable, NodeRef,
};

pub struct NodeAttributes {
    pub label: LabelText<'static>,
    pub style: Style,

    pub color: Option<LabelText<'static>>,

    pub shape: Option<LabelText<'static>>,
}

impl NodeAttributes {
    #[must_use]
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: LabelText::LabelStr(Cow::Owned(label.into())),
            style: Style::None,

            color: None,

            shape: None,
        }
    }

    #[must_use]
    pub const fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    #[must_use]
    pub fn with_color(mut self, color: impl Into<LabelText<'static>>) -> Self {
        self.color = Some(color.into());
        self
    }

    #[must_use]
    pub fn with_shape(mut self, shape: impl Into<LabelText<'static>>) -> Self {
        self.shape = Some(shape.into());
        self
    }
}

pub struct EdgeAttributes {
    pub label: LabelText<'static>,
    pub style: Style,

    pub color: Option<LabelText<'static>>,

    pub start_arrow: Arrow,
    pub end_arrow: Arrow,
}

impl EdgeAttributes {
    #[must_use]
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: LabelText::LabelStr(Cow::Owned(label.into())),
            style: Style::None,

            color: None,

            start_arrow: Arrow::none(),
            end_arrow: Arrow::normal(),
        }
    }

    #[must_use]
    pub const fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    #[must_use]
    pub fn with_color(mut self, color: impl Into<LabelText<'static>>) -> Self {
        self.color = Some(color.into());
        self
    }

    #[allow(clippy::missing_const_for_fn)] // Reason: false positive
    #[must_use]
    pub fn with_start_arrow(mut self, arrow: Arrow) -> Self {
        self.start_arrow = arrow;
        self
    }

    #[allow(clippy::missing_const_for_fn)] // Reason: false positive
    #[must_use]
    pub fn with_end_arrow(mut self, arrow: Arrow) -> Self {
        self.end_arrow = arrow;
        self
    }
}

/// `Dot` implements output to graphviz .dot format for a graph.
///
/// Formatting and options are rather simple, this is mostly intended
/// for debugging. Exact output may change.
///
/// # Examples
///
/// ```
/// use dot::RenderOption;
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
/// println!(
///     "{:?}",
///     Dot::with_config(&graph, &[RenderOption::NoEdgeLabels])
/// );
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
// TODO: revisit this API, it's not very ergonomic
pub struct Dot<'a, G>
where
    G: IntoEdgeReferences + IntoNodeReferences,
{
    graph: G,

    get_node_attributes: &'a dyn Fn(G, G::NodeRef) -> NodeAttributes,
    get_edge_attributes: &'a dyn Fn(G, G::EdgeRef) -> EdgeAttributes,

    options: &'a [RenderOption],
    // in future we might want to cache node and edge attributes, not implemented yet because we
    // would need to tighten the bounds, which we will change anyway
}

impl<'a, G> Dot<'a, G>
where
    G: IntoEdgeReferences + IntoNodeReferences,
{
    fn call_node_attributes(&self, node: G::NodeId) -> NodeAttributes {
        let node = self
            .graph
            .node_references()
            .find(|n| n.id() == node)
            .expect("node not found");

        (self.get_node_attributes)(self.graph, node)
    }

    fn call_edge_attributes(&self, edge: G::EdgeRef) -> EdgeAttributes {
        (self.get_edge_attributes)(self.graph, edge)
    }
}

impl<'a, G> Dot<'a, G>
where
    G: IntoNodeIdentifiers + IntoNodeReferences + IntoEdgeReferences,
    // TODO: remove this bound once we have reworked how indices work
    G::NodeId: Display,
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
            options,
            &|_, _| EdgeAttributes::new(""),
            &|_, _| NodeAttributes::new(""),
        )
    }

    #[inline]
    pub fn with_attr_getters(
        graph: G,
        options: &'a [RenderOption],

        get_edge_attributes: &'a dyn Fn(G, G::EdgeRef) -> EdgeAttributes,
        get_node_attributes: &'a dyn Fn(G, G::NodeRef) -> NodeAttributes,
    ) -> Self {
        Self {
            graph,

            get_node_attributes,
            get_edge_attributes,

            options,
        }
    }

    /// Set the function used to get the attributes for an edge.
    #[must_use]
    pub fn with_edge_attributes(
        mut self,
        get_edge_attributes: &'a dyn Fn(G, G::EdgeRef) -> EdgeAttributes,
    ) -> Self {
        self.get_edge_attributes = get_edge_attributes;
        self
    }

    /// Set the function used to get the attributes for a node.
    #[must_use]
    pub fn with_node_attributes(
        mut self,
        get_node_attributes: &'a dyn Fn(G, G::NodeRef) -> NodeAttributes,
    ) -> Self {
        self.get_node_attributes = get_node_attributes;
        self
    }

    /// Write the graph to the given writer in the dot format.
    ///
    /// This delegates most of the heavy lifting to the [`dot`] crate.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying [`dot`] crate fails to write to the
    /// given writer.
    ///
    /// [`dot`]: https://docs.rs/dot
    pub fn write<W>(self, write: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        dot::render_opts(&self, write, self.options)
    }
}

impl<'a, G> dot::Labeller<'a, G::NodeId, G::EdgeRef> for Dot<'a, G>
where
    G: IntoNodeIdentifiers + IntoNodeReferences + IntoEdgeReferences,
    G::NodeId: Display,
{
    fn graph_id(&'a self) -> Id<'a> {
        // TODO: make configurable
        Id::new("petgraph").expect("infallible")
    }

    fn node_id(&'a self, n: &G::NodeId) -> Id<'a> {
        Id::new(format!("N{n}")).expect("infallible")
    }

    fn node_label(&'a self, node: &G::NodeId) -> LabelText<'a> {
        self.call_node_attributes(*node).label
    }

    fn node_color(&'a self, node: &G::NodeId) -> Option<LabelText<'a>> {
        self.call_node_attributes(*node).color
    }

    fn node_style(&'a self, node: &G::NodeId) -> Style {
        self.call_node_attributes(*node).style
    }

    fn node_shape(&'a self, node: &G::NodeId) -> Option<LabelText<'a>> {
        self.call_node_attributes(*node).shape
    }

    fn edge_color(&'a self, edge: &G::EdgeRef) -> Option<LabelText<'a>> {
        self.call_edge_attributes(*edge).color
    }

    fn edge_end_arrow(&'a self, edge: &G::EdgeRef) -> Arrow {
        self.call_edge_attributes(*edge).end_arrow
    }

    fn edge_label(&'a self, edge: &G::EdgeRef) -> LabelText<'a> {
        self.call_edge_attributes(*edge).label
    }

    fn edge_start_arrow(&'a self, edge: &G::EdgeRef) -> Arrow {
        self.call_edge_attributes(*edge).start_arrow
    }

    fn edge_style(&'a self, edge: &G::EdgeRef) -> Style {
        self.call_edge_attributes(*edge).style
    }

    // TODO: we have no way to know if the graph is directed or not
    // fn kind(&self) -> Kind {
    //     if G::is_directed() {
    //         Kind::Digraph
    //     } else {
    //         Kind::Graph
    //     }
    // }
}

impl<'a, G> dot::GraphWalk<'a, G::NodeId, G::EdgeRef> for Dot<'a, G>
where
    G: IntoNodeIdentifiers + IntoNodeReferences + IntoEdgeReferences,
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
    G: IntoNodeReferences + IntoNodeReferences + IntoEdgeReferences,
    G::NodeId: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl<'a, G> Debug for Dot<'a, G>
where
    G: IntoNodeReferences + IntoNodeReferences + IntoEdgeReferences,
    G::NodeId: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut buffer = Vec::new();

        dot::render_opts(self, &mut buffer, self.options).map_err(|_| fmt::Error)?;

        let mut s = String::from_utf8(buffer).map_err(|_| fmt::Error)?;
        f.write_str(&s)
    }
}

#[cfg(test)]
mod test {
    use dot::RenderOption;
    use insta::assert_debug_snapshot;
    use petgraph_core::visit::EdgeRef;

    use crate::{
        dot::{Dot, EdgeAttributes, NodeAttributes},
        prelude::Graph,
        visit::NodeRef,
    };

    fn simple_graph() -> Graph<&'static str, &'static str> {
        let mut graph = Graph::<&str, &str>::new();
        let a = graph.add_node("A");
        let b = graph.add_node("B");
        graph.add_edge(a, b, "edge_label");
        graph
    }

    #[test]
    fn node_index_label() {
        let graph = simple_graph();

        let dot = Dot::with_config(&graph, &[])
            .with_node_attributes(&|_, (index, _)| NodeAttributes::new(index.to_string()));

        assert_debug_snapshot!(dot);
    }

    #[test]
    fn edge_index_label() {
        let graph = simple_graph();

        let dot = Dot::with_config(&graph, &[])
            .with_edge_attributes(&|_, edge| EdgeAttributes::new(edge.id().index().to_string()));

        assert_debug_snapshot!(dot);
    }

    #[test]
    fn edge_no_label() {
        let graph = simple_graph();
        let dot = Dot::with_config(&graph, &[RenderOption::NoEdgeLabels]);

        assert_debug_snapshot!(dot);
    }

    #[test]
    fn node_no_label() {
        let graph = simple_graph();
        let dot = Dot::with_config(&graph, &[RenderOption::NoNodeLabels]);

        assert_debug_snapshot!(dot);
    }

    #[test]
    fn label_map_to_weight() {
        let graph = simple_graph();
        let dot = Dot::with_config(&graph, &[])
            .with_node_attributes(&|_, (_, weight)| NodeAttributes::new(weight.to_uppercase()))
            .with_edge_attributes(&|_, edge| EdgeAttributes::new(edge.weight().to_uppercase()));

        assert_debug_snapshot!(dot);
    }
}
