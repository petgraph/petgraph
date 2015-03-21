
use std::fmt;
use super::Graph;
use super::EdgeType;
use super::graph::IndexType;

/// Wrapper implementing formatting to graphviz .dot format for a graph.
#[derive(Copy, Clone, Debug)]
pub struct DisplayDot<'a, G: 'a> {
    graph: &'a G,
}

static TYPE: [&'static str; 2] = ["graph", "digraph"];
static EDGE: [&'static str; 2] = ["--", "->"];
static INDENT: &'static str = "    ";

impl<'a, G> DisplayDot<'a, G>
{
    pub fn new(graph: &'a G) -> Self {
        DisplayDot {
            graph: graph,
        }
    }
}

impl<'a, N, E, Ty, Ix> fmt::Display for DisplayDot<'a, Graph<N, E, Ty, Ix>> where
    N: fmt::Display,
    E: fmt::Display,
    Ty: EdgeType,
    Ix: IndexType,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        let g = self.graph;
        try!(writeln!(f, "{} {{", TYPE[g.is_directed() as usize]));

        // output all labels
        for (index, node) in g.raw_nodes().iter().enumerate() {
            try!(writeln!(f, "{}N{} [label=\"{}\"]",
                          INDENT, index, node.weight));

        }
        // output all edges
        for edge in g.raw_edges().iter() {
            try!(writeln!(f, "{}N{} {} N{} [label=\"{}\"]",
                          INDENT,
                          edge.source().index(),
                          EDGE[g.is_directed() as usize],
                          edge.target().index(),
                          edge.weight));

        }

        // node name is "N%d"
        try!(writeln!(f, "}}"));
        Ok(())
    }
}

