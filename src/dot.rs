//! Simple graphviz dot file format output.

use std::fmt::{self, Display, Write};
use {
    Graph,
    EdgeType,
    GraphMap,
};
use super::graph::IndexType;
use graphmap::NodeTrait;
use std::collections::HashMap;

/// `Dot` implements output to graphviz .dot format for a graph.
///
/// Formatting and options are rather simple, this is mostly intended
/// for debugging. Exact output may change.
pub struct Dot<'a, G: 'a> {
    graph: &'a G,
    config: &'a [Config],
}

static TYPE: [&'static str; 2] = ["graph", "digraph"];
static EDGE: [&'static str; 2] = ["--", "->"];
static INDENT: &'static str = "    ";

impl<'a, G> Dot<'a, G> {
    /// Create a `Dot` formatting wrapper with default configuration.
    pub fn new(graph: &'a G) -> Self {
        Self::with_config(graph, &[])
    }

    /// Create a `Dot` formatting wrapper with custom configuration.
    pub fn with_config(graph: &'a G, config: &'a [Config]) -> Self {
        Dot {
            graph: graph,
            config: config,
        }
    }
}

/// `Dot` configuration.
///
/// This enum does not have an exhaustive definition (will be expanded)
#[derive(Debug, PartialEq, Eq)]
pub enum Config {
    /// Use indices for node labels.
    NodeIndexLabel,
    /// Use indices for edge labels.
    EdgeIndexLabel,
    /// Use no edge labels.
    EdgeNoLabel,
    #[doc(hidden)]
    _Incomplete(()),
}

impl<'a, N, E, Ty, Ix> Dot<'a, Graph<N, E, Ty, Ix>>
    where Ty: EdgeType,
          Ix: IndexType,
{
    fn graph_fmt<F, G>(&self, f: &mut fmt::Formatter,
                       mut node_fmt: F, mut edge_fmt: G) -> fmt::Result
        where F: FnMut(&N, &mut FnMut(&Display) -> fmt::Result) -> fmt::Result,
              G: FnMut(&E, &mut FnMut(&Display) -> fmt::Result) -> fmt::Result,
{
        let g = self.graph;
        try!(writeln!(f, "{} {{", TYPE[g.is_directed() as usize]));

        // output all labels
        for index in g.node_indices() {
            try!(write!(f, "{}{}", INDENT, index.index()));
            if self.config.contains(&Config::NodeIndexLabel) {
                try!(writeln!(f, ""));
            } else {
                try!(write!(f, " [label=\""));
                try!(node_fmt(&g[index], &mut |d| write!(f, "{}", Escaped(d))));
                try!(writeln!(f, "\"]"));
            }

        }
        // output all edges
        for (i, edge) in g.raw_edges().iter().enumerate() {
            try!(write!(f, "{}{} {} {}",
                        INDENT,
                        edge.source().index(),
                        EDGE[g.is_directed() as usize],
                        edge.target().index()));
            if self.config.contains(&Config::EdgeNoLabel) {
                try!(writeln!(f, ""));
            } else if self.config.contains(&Config::EdgeIndexLabel) {
                try!(writeln!(f, " [label=\"{}\"]", i));
            } else {
                try!(write!(f, " [label=\""));
                try!(edge_fmt(&edge.weight, &mut |d| write!(f, "{}", Escaped(d))));
                try!(writeln!(f, "\"]"));
            }
        }

        try!(writeln!(f, "}}"));
        Ok(())
    }
}

impl<'a, N, E, Ty, Ix> fmt::Display for Dot<'a, Graph<N, E, Ty, Ix>>
    where N: fmt::Display,
          E: fmt::Display,
          Ty: EdgeType,
          Ix: IndexType,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.graph_fmt(f, |n, cb| cb(n), |e, cb| cb(e))
    }
}

impl<'a, N, E, Ty, Ix> fmt::Debug for Dot<'a, Graph<N, E, Ty, Ix>>
    where N: fmt::Debug,
          E: fmt::Debug,
          Ty: EdgeType,
          Ix: IndexType,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.graph_fmt(f,
                       |n, cb| cb(&format_args!("{:?}", n)),
                       |e, cb| cb(&format_args!("{:?}", e)))
    }
}

impl<'a, N, E> Dot<'a, GraphMap<N, E>>
    where N: NodeTrait,
{
    fn graphmap_fmt<F, G>(&self, f: &mut fmt::Formatter,
                          mut node_fmt: F, mut edge_fmt: G) -> fmt::Result
        where F: FnMut(&N, &mut FnMut(&Display) -> fmt::Result) -> fmt::Result,
              G: FnMut(&E, &mut FnMut(&Display) -> fmt::Result) -> fmt::Result,
    {
        let g = self.graph;
        try!(writeln!(f, "{} {{", TYPE[0]));

        let mut labels = HashMap::new();

        // output all labels
        for (i, node) in g.nodes().enumerate() {
            labels.insert(node, i);
            try!(write!(f, "{}{}", INDENT, i));
            if self.config.contains(&Config::NodeIndexLabel) {
                try!(writeln!(f, ""));
            } else {
                try!(write!(f, " [label=\""));
                try!(node_fmt(&node, &mut |d| write!(f, "{}", Escaped(d))));
                try!(writeln!(f, "\"]"));
            }
        }
        // output all edges
        for (i, (a, b, edge_weight)) in g.all_edges().enumerate() {
            try!(write!(f, "{}{} {} {}",
                        INDENT,
                        labels[&a],
                        EDGE[0],
                        labels[&b]));
            if self.config.contains(&Config::EdgeNoLabel) {
                try!(writeln!(f, ""));
            } else if self.config.contains(&Config::EdgeIndexLabel) {
                try!(writeln!(f, " [label=\"{}\"]", i));
            } else {
                try!(write!(f, " [label=\""));
                try!(edge_fmt(&edge_weight, &mut |d| write!(f, "{}", Escaped(d))));
                try!(writeln!(f, "\"]"));
            }
        }

        try!(writeln!(f, "}}"));
        Ok(())
    }
}
impl<'a, N, E> fmt::Display for Dot<'a, GraphMap<N, E>>
    where N: fmt::Display + NodeTrait,
          E: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.graphmap_fmt(f, |n, cb| cb(n), |e, cb| cb(e))
    }
}

impl<'a, N, E> fmt::Debug for Dot<'a, GraphMap<N, E>>
    where N: fmt::Debug + NodeTrait,
          E: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.graphmap_fmt(f,
                          |n, cb| cb(&format_args!("{:?}", n)),
                          |e, cb| cb(&format_args!("{:?}", e)))
    }
}

/// For Graphviz, we only need to escape double quotes in labels
struct Escaper<W>(W);

impl<W> fmt::Write for Escaper<W>
    where W: fmt::Write
{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            try!(self.write_char(c));
        }
        Ok(())
    }

    fn write_char(&mut self, c: char) -> fmt::Result {
        match c {
            '"' => try!(self.0.write_char('\\')),
            _   => { }
        }
        self.0.write_char(c)
    }
}

/// Pass Display and Debug through simple escaping filter
struct Escaped<T>(T);

impl<T> fmt::Display for Escaped<T>
    where T: fmt::Display
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(&mut Escaper(f), "{}", &self.0)
    }
}

impl<T> fmt::Debug for Escaped<T>
    where T: fmt::Debug
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(&mut Escaper(f), "{:?}", &self.0)
    }
}
