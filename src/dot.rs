//! Simple graphviz dot file format output.

use std::fmt::{self, Display, Write};

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
/// use petgraph::Graph;
/// use petgraph::dot::{Dot, Config};
///
/// let mut graph = Graph::<_, ()>::new();
/// graph.add_node("A");
/// graph.add_node("B");
/// graph.add_node("C");
/// graph.add_node("D");
/// graph.extend_with_edges(&[
///     (0, 1), (0, 2), (0, 3),
///     (1, 2), (1, 3),
///     (2, 3),
/// ]);
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
    config: Configs,
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
    pub fn with_config(graph: G, config: &'a [Config]) -> Self {
        Self::with_attr_getters(graph, config, &|_, _| String::new(), &|_, _| String::new())
    }

    #[inline]
    pub fn with_attr_getters(
        graph: G,
        config: &'a [Config],
        get_edge_attributes: &'a dyn Fn(G, G::EdgeRef) -> String,
        get_node_attributes: &'a dyn Fn(G, G::NodeRef) -> String,
    ) -> Self {
        let config = Configs::extract(config);
        Dot {
            graph,
            get_edge_attributes,
            get_node_attributes,
            config,
        }
    }
}

/// `Dot` configuration.
///
/// This enum does not have an exhaustive definition (will be expanded)
// TODO: #[non_exhaustive] once MSRV >= 1.40,
// and/or for a breaking change make this something like an EnumSet: https://docs.rs/enumset
#[derive(Debug, PartialEq, Eq)]
pub enum Config {
    /// Use indices for node labels.
    NodeIndexLabel,
    /// Use indices for edge labels.
    EdgeIndexLabel,
    /// Use no edge labels.
    EdgeNoLabel,
    /// Use no node labels.
    NodeNoLabel,
    /// Do not print the graph/digraph string.
    GraphContentOnly,
    #[doc(hidden)]
    _Incomplete(()),
}
macro_rules! make_config_struct {
    ($($variant:ident,)*) => {
        #[allow(non_snake_case)]
        #[derive(Default)]
        struct Configs {
            $($variant: bool,)*
        }
        impl Configs {
            #[inline]
            fn extract(configs: &[Config]) -> Self {
                let mut conf = Self::default();
                for c in configs {
                    match *c {
                        $(Config::$variant => conf.$variant = true,)*
                        Config::_Incomplete(()) => {}
                    }
                }
                conf
            }
        }
    }
}
make_config_struct!(
    NodeIndexLabel,
    EdgeIndexLabel,
    EdgeNoLabel,
    NodeNoLabel,
    GraphContentOnly,
);

impl<'a, G> Dot<'a, G>
where
    G: IntoNodeReferences + IntoEdgeReferences + NodeIndexable + GraphProp,
{
    fn graph_fmt<NF, EF>(&self, f: &mut fmt::Formatter, node_fmt: NF, edge_fmt: EF) -> fmt::Result
    where
        NF: Fn(&G::NodeWeight, &mut fmt::Formatter) -> fmt::Result,
        EF: Fn(&G::EdgeWeight, &mut fmt::Formatter) -> fmt::Result,
    {
        let g = self.graph;
        if !self.config.GraphContentOnly {
            writeln!(f, "{} {{", TYPE[g.is_directed() as usize])?;
        }

        // output all labels
        for node in g.node_references() {
            write!(f, "{}{} [ ", INDENT, g.to_index(node.id()),)?;
            if !self.config.NodeNoLabel {
                write!(f, "label = \"")?;
                if self.config.NodeIndexLabel {
                    write!(f, "{}", g.to_index(node.id()))?;
                } else {
                    Escaped(FnFmt(node.weight(), &node_fmt)).fmt(f)?;
                }
                write!(f, "\" ")?;
            }
            writeln!(f, "{}]", (self.get_node_attributes)(g, node))?;
        }
        // output all edges
        for (i, edge) in g.edge_references().enumerate() {
            write!(
                f,
                "{}{} {} {} [ ",
                INDENT,
                g.to_index(edge.source()),
                EDGE[g.is_directed() as usize],
                g.to_index(edge.target()),
            )?;
            if !self.config.EdgeNoLabel {
                write!(f, "label = \"")?;
                if self.config.EdgeIndexLabel {
                    write!(f, "{}", i)?;
                } else {
                    Escaped(FnFmt(edge.weight(), &edge_fmt)).fmt(f)?;
                }
                write!(f, "\" ")?;
            }
            writeln!(f, "{}]", (self.get_edge_attributes)(g, edge))?;
        }

        if !self.config.GraphContentOnly {
            writeln!(f, "}}")?;
        }
        Ok(())
    }
}

impl<'a, G> fmt::Display for Dot<'a, G>
where
    G: IntoEdgeReferences + IntoNodeReferences + NodeIndexable + GraphProp,
    G::EdgeWeight: fmt::Display,
    G::NodeWeight: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.graph_fmt(f, fmt::Display::fmt, fmt::Display::fmt)
    }
}

impl<'a, G> fmt::Debug for Dot<'a, G>
where
    G: IntoEdgeReferences + IntoNodeReferences + NodeIndexable + GraphProp,
    G::EdgeWeight: fmt::Debug,
    G::NodeWeight: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.graph_fmt(f, fmt::Debug::fmt, fmt::Debug::fmt)
    }
}

/// Escape for Graphviz
struct Escaper<W>(W);

impl<W> fmt::Write for Escaper<W>
where
    W: fmt::Write,
{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_char(c)?;
        }
        Ok(())
    }

    fn write_char(&mut self, c: char) -> fmt::Result {
        match c {
            '"' | '\\' => self.0.write_char('\\')?,
            // \l is for left justified linebreak
            '\n' => return self.0.write_str("\\l"),
            _ => {}
        }
        self.0.write_char(c)
    }
}

/// Detects whether the input has multiple lines
struct MultilineDetector<W> {
    writer: W,
    saw_backslash: bool,
    saw_backslash_then_l: bool,
}

impl<W> MultilineDetector<W> {
    fn new(writer: W) -> Self {
        Self {
            writer,
            saw_backslash: false,
            saw_backslash_then_l: false,
        }
    }

    /// Whether the written data was multiline
    fn was_multiline(&self) -> bool {
        // This will be true if we saw at least one occurence of `\l`
        self.saw_backslash_then_l
    }
}

impl<W> fmt::Write for MultilineDetector<W>
where
    W: fmt::Write,
{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if !self.was_multiline() {
            for c in s.chars() {
                self.write_char(c)?;
            }
        } else {
            // We got what we wanted and don't need to check anything else
            self.writer.write_str(s)?;
        }

        Ok(())
    }

    fn write_char(&mut self, c: char) -> fmt::Result {
        self.writer.write_char(c)?;

        if self.saw_backslash_then_l {
            // We got what we wanted and don't need to check anything else
            return Ok(());
        }

        match c {
            '\\' => self.saw_backslash = true,
            'l' if self.saw_backslash => self.saw_backslash_then_l = true,
            _ => {
                self.saw_backslash = false;
            }
        }

        Ok(())
    }
}

/// Pass Display formatting through a simple escaping filter
struct Escaped<T>(T);

impl<T> fmt::Display for Escaped<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let alternate = f.alternate();

        let mut w = Escaper(MultilineDetector::new(&mut *f));

        if alternate {
            writeln!(w, "{:#}", &self.0)?;
        } else {
            write!(w, "{}", &self.0)?;
        }

        // If there's more than one line, make the last line left-justified
        if w.0.was_multiline() {
            write!(f, "\\l")?;
        }

        Ok(())
    }
}

/// Format data using a specific format function
struct FnFmt<'a, T, F>(&'a T, F);

impl<'a, T, F> fmt::Display for FnFmt<'a, T, F>
where
    F: Fn(&'a T, &mut fmt::Formatter<'_>) -> fmt::Result,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.1(self.0, f)
    }
}

#[cfg(test)]
mod test {
    use super::{Config, Dot, Escaper};
    use crate::prelude::Graph;
    use crate::visit::NodeRef;
    use std::fmt::Write;

    #[test]
    fn test_escape() {
        let mut buff = String::new();
        {
            let mut e = Escaper(&mut buff);
            let _ = e.write_str("\" \\ \n");
        }
        assert_eq!(buff, "\\\" \\\\ \\l");
    }

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
        let dot = format!("{:?}", Dot::with_config(&graph, &[Config::NodeIndexLabel]));
        assert_eq!(dot, "digraph {\n    0 [ label = \"0\" ]\n    1 [ label = \"1\" ]\n    0 -> 1 [ label = \"\\\"edge_label\\\"\" ]\n}\n");
    }

    #[test]
    fn test_edgeindexlable_option() {
        let graph = simple_graph();
        let dot = format!("{:?}", Dot::with_config(&graph, &[Config::EdgeIndexLabel]));
        assert_eq!(dot, "digraph {\n    0 [ label = \"\\\"A\\\"\" ]\n    1 [ label = \"\\\"B\\\"\" ]\n    0 -> 1 [ label = \"0\" ]\n}\n");
    }

    #[test]
    fn test_edgenolable_option() {
        let graph = simple_graph();
        let dot = format!("{:?}", Dot::with_config(&graph, &[Config::EdgeNoLabel]));
        assert_eq!(dot, "digraph {\n    0 [ label = \"\\\"A\\\"\" ]\n    1 [ label = \"\\\"B\\\"\" ]\n    0 -> 1 [ ]\n}\n");
    }

    #[test]
    fn test_nodenolable_option() {
        let graph = simple_graph();
        let dot = format!("{:?}", Dot::with_config(&graph, &[Config::NodeNoLabel]));
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
                &[Config::NodeNoLabel, Config::EdgeNoLabel],
                &|_, er| format!("label = \"{}\"", er.weight().to_uppercase()),
                &|_, nr| format!("label = \"{}\"", nr.weight().to_lowercase()),
            ),
        );
        assert_eq!(dot, "digraph {\n    0 [ label = \"a\"]\n    1 [ label = \"b\"]\n    0 -> 1 [ label = \"EDGE_LABEL\"]\n}\n");
    }

    #[derive(Debug)]
    struct MultilineDebug {
        #[allow(dead_code)]
        content: &'static str,
    }

    #[test]
    fn test_multiline_debug() {
        let mut graph = Graph::<_, i32>::new();
        graph.add_node(MultilineDebug { content: "foo" });

        let dot = format!("{:#?}", Dot::new(&graph));

        // The final `\l` just before the `\"` to end the label is the important
        // part. It ensures the last line is left-aligned like everything else
        // instead of center-aligned.
        assert_eq!(dot, "digraph {\n    0 [ label = \"MultilineDebug {\\l    content: \\\"foo\\\",\\l}\\l\\l\" ]\n}\n");
    }
}
