//! Simple graphml file format output.

use std::borrow::Cow;
use std::collections::HashSet;
use std::io::{Cursor, Result as IoResult, Write};
use std::string::ToString;
use visit::{EdgeRef, GraphProp, IntoEdgeReferences, IntoNodeReferences, NodeIndexable, NodeRef};
use xml::common::XmlVersion;
use xml::writer::events::XmlEvent;
use xml::writer::Error as XmlError;
use xml::writer::{EventWriter, Result as WriterResult};
use xml::EmitterConfig;

static NAMESPACE_URL: &str = "http://graphml.graphdrawing.org/xmlns";

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
struct Attribute {
    name: Cow<'static, str>,
    for_: For,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
enum For {
    Node,
    Edge,
}

impl For {
    fn to_str(&self) -> &'static str {
        match *self {
            For::Node => "node",
            For::Edge => "edge",
        }
    }
}

type PrintWeights<W> = for<'a> Fn(&'a W) -> Vec<(Cow<'static, str>, Cow<'a, str>)>;

pub struct GraphMl<G>
where
    G: IntoEdgeReferences,
    G: IntoNodeReferences,
{
    graph: G,
    pretty_print: bool,
    export_edges: Option<Box<PrintWeights<G::EdgeWeight>>>,
    export_nodes: Option<Box<PrintWeights<G::NodeWeight>>>,
}

impl<G> GraphMl<G>
where
    G: GraphProp,
    G: IntoNodeReferences,
    G: IntoEdgeReferences,
    G: NodeIndexable,
{
    pub fn new(graph: G) -> Self {
        Self {
            graph: graph,
            pretty_print: true,
            export_edges: None,
            export_nodes: None,
        }
    }

    pub fn pretty_print(mut self, state: bool) -> Self {
        self.pretty_print = state;
        self
    }

    pub fn export_edge_weights_display(self) -> Self
    where
        G::EdgeWeight: ToString,
    {
        self.export_edge_weights(Box::new(|edge| {
            vec![("weight".into(), edge.to_string().into())]
        }))
    }

    pub fn export_edge_weights(mut self, edge_weight: Box<PrintWeights<G::EdgeWeight>>) -> Self {
        self.export_edges = Some(edge_weight);
        self
    }

    pub fn export_node_weights_display(self) -> Self
    where
        G::NodeWeight: ToString,
    {
        self.export_node_weights(Box::new(|node| {
            vec![("weight".into(), node.to_string().into())]
        }))
    }

    pub fn export_node_weights(mut self, node_weight: Box<PrintWeights<G::NodeWeight>>) -> Self {
        self.export_nodes = Some(node_weight);
        self
    }

    pub fn to_string(&self) -> String {
        let mut buff = Cursor::new(Vec::new());
        self.to_writer(&mut buff)
            .expect("Writing to a Cursor should never create IO errors.");
        String::from_utf8(buff.into_inner()).unwrap()
    }

    pub fn to_writer<W>(&self, writer: W) -> IoResult<()>
    where
        W: Write,
    {
        let mut writer = EventWriter::new_with_config(
            writer,
            EmitterConfig::new().perform_indent(self.pretty_print),
        );
        match self.emit_graphml(&mut writer) {
            Ok(()) => Ok(()),
            Err(XmlError::Io(ioerror)) => Err(ioerror),
            _ => panic!(""),
        }
    }

    fn emit_graphml<W>(&self, writer: &mut EventWriter<W>) -> WriterResult<()>
    where
        W: Write,
    {
        // Store information about the attributes for nodes and edges.
        // We cannot know in advance what the attribute names will be, so we just keep track of what gets emitted.
        let mut attributes: HashSet<Attribute> = HashSet::new();

        // XML/GraphML boilerplate
        writer.write(XmlEvent::StartDocument {
            version: XmlVersion::Version10,
            encoding: Some("UTF-8"),
            standalone: None,
        })?;
        writer.write(XmlEvent::start_element("graphml").attr("xmlns", NAMESPACE_URL))?;

        // emit graph with nodes/edges and possibly weights
        self.emit_graph(writer, &mut attributes)?;
        // Emit <key> tags for all the attributes
        self.emit_keys(writer, &attributes)?;

        writer.write(XmlEvent::end_element())?; // end graphml
        Ok(())
    }

    fn emit_graph<W>(
        &self,
        writer: &mut EventWriter<W>,
        attributes: &mut HashSet<Attribute>,
    ) -> WriterResult<()>
    where
        W: Write,
    {
        // convenience function to turn a NodeId into a String
        let node2str_id = |node: G::NodeId| -> String { format!("n{}", self.graph.to_index(node)) };
        // Emit an attribute for either node or edge
        // This will also keep track of updating the global attributes list
        let mut emit_attribute = |writer: &mut EventWriter<_>,
                                  name: Cow<'static, str>,
                                  data: &str,
                                  for_: For|
         -> WriterResult<()> {
            writer.write(XmlEvent::start_element("data").attr("key", &*name))?;
            attributes.insert(Attribute {
                name: name,
                for_: for_,
            });
            writer.write(XmlEvent::characters(data))?;
            writer.write(XmlEvent::end_element()) // end data
        };

        // Each graph needs a default edge type
        writer.write(XmlEvent::start_element("graph").attr(
            "edgedefault",
            if self.graph.is_directed() {
                "directed"
            } else {
                "undirected"
            },
        ))?;

        // Emit nodes
        for node in self.graph.node_references() {
            writer.write(XmlEvent::start_element("node").attr("id", &*node2str_id(node.id())))?;
            // Print weights
            if let Some(ref node_labels) = self.export_nodes {
                let datas = node_labels(&node.weight());
                for (name, data) in datas {
                    emit_attribute(writer, name, &*data, For::Node)?;
                }
            }
            writer.write(XmlEvent::end_element())?; // end node
        }

        // Emit edges
        for (i, edge) in self.graph.edge_references().enumerate() {
            writer.write(
                XmlEvent::start_element("edge")
                    .attr("id", &format!("e{}", i))
                    .attr("source", &*node2str_id(edge.source()))
                    .attr("target", &*node2str_id(edge.target())),
            )?;
            // Print weights
            if let Some(ref edge_labels) = self.export_edges {
                let datas = edge_labels(&edge.weight());
                for (name, data) in datas {
                    emit_attribute(writer, name, &*data, For::Edge)?;
                }
            }
            writer.write(XmlEvent::end_element())?; // end edge
        }
        writer.write(XmlEvent::end_element()) // end graph
    }

    fn emit_keys<W>(
        &self,
        writer: &mut EventWriter<W>,
        attributes: &HashSet<Attribute>,
    ) -> WriterResult<()>
    where
        W: Write,
    {
        for attr in attributes {
            writer.write(
                XmlEvent::start_element("key")
                    .attr("id", &*attr.name)
                    .attr("for", attr.for_.to_str())
                    .attr("attr.name", &*attr.name)
                    .attr("attr.type", "string"),
            )?;
            writer.write(XmlEvent::end_element())?; // end key
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::GraphMl;
    use graph::Graph;

    #[test]
    fn single_node() {
        let mut deps = Graph::<&str, &str>::new();
        deps.add_node("petgraph");

        let graphml = GraphMl::new(&deps).pretty_print(true);
        let xml = graphml.to_string();
        let expected = r#"<?xml version="1.0" encoding="UTF-8"?>
<graphml xmlns="http://graphml.graphdrawing.org/xmlns">
  <graph edgedefault="directed">
    <node id="n0" />
  </graph>
</graphml>"#;

        assert_eq!(expected, xml);
    }

    #[test]
    fn single_node_disable_pretty() {
        let mut deps = Graph::<&str, &str>::new();
        deps.add_node("petgraph");

        let graphml = GraphMl::new(&deps).pretty_print(false);
        let xml = graphml.to_string();
        let expected = r#"<?xml version="1.0" encoding="UTF-8"?><graphml xmlns="http://graphml.graphdrawing.org/xmlns"><graph edgedefault="directed"><node id="n0" /></graph></graphml>"#;

        assert_eq!(expected, xml);
    }

    #[test]
    fn single_node_with_display_weight() {
        let mut deps = Graph::<&str, &str>::new();
        deps.add_node("petgraph");

        let graphml = GraphMl::new(&deps)
            .pretty_print(true)
            .export_node_weights_display();
        let xml = graphml.to_string();
        let expected = r#"<?xml version="1.0" encoding="UTF-8"?>
<graphml xmlns="http://graphml.graphdrawing.org/xmlns">
  <graph edgedefault="directed">
    <node id="n0">
      <data key="weight">petgraph</data>
    </node>
  </graph>
  <key id="weight" for="node" attr.name="weight" attr.type="string" />
</graphml>"#;

        assert_eq!(expected, xml);
    }

    #[test]
    fn single_edge() {
        let mut deps = Graph::<&str, &str>::new();
        let pg = deps.add_node("petgraph");
        let fb = deps.add_node("fixedbitset");
        deps.extend_with_edges(&[(pg, fb)]);

        let graphml = GraphMl::new(&deps).pretty_print(true);
        let xml = graphml.to_string();
        let expected = r#"<?xml version="1.0" encoding="UTF-8"?>
<graphml xmlns="http://graphml.graphdrawing.org/xmlns">
  <graph edgedefault="directed">
    <node id="n0" />
    <node id="n1" />
    <edge id="e0" source="n0" target="n1" />
  </graph>
</graphml>"#;
        assert_eq!(expected, xml);
    }

    #[test]
    fn single_edge_with_display_weight() {
        let mut deps = Graph::<&str, &str>::new();
        let pg = deps.add_node("petgraph");
        let fb = deps.add_node("fixedbitset");
        deps.update_edge(pg, fb, "depends on");

        let graphml = GraphMl::new(&deps)
            .pretty_print(true)
            .export_edge_weights_display();
        let xml = graphml.to_string();
        let expected = r#"<?xml version="1.0" encoding="UTF-8"?>
<graphml xmlns="http://graphml.graphdrawing.org/xmlns">
  <graph edgedefault="directed">
    <node id="n0" />
    <node id="n1" />
    <edge id="e0" source="n0" target="n1">
      <data key="weight">depends on</data>
    </edge>
  </graph>
  <key id="weight" for="edge" attr.name="weight" attr.type="string" />
</graphml>"#;
        assert_eq!(expected, xml);
    }

    #[test]
    fn node_and_edge_display_weight() {
        let mut deps = Graph::<&str, &str>::new();
        let pg = deps.add_node("petgraph");
        let fb = deps.add_node("fixedbitset");
        deps.update_edge(pg, fb, "depends on");

        let graphml = GraphMl::new(&deps)
            .pretty_print(true)
            .export_edge_weights_display()
            .export_node_weights_display();
        let xml = graphml.to_string();
        let expected1 = r#"<?xml version="1.0" encoding="UTF-8"?>
<graphml xmlns="http://graphml.graphdrawing.org/xmlns">
  <graph edgedefault="directed">
    <node id="n0">
      <data key="weight">petgraph</data>
    </node>
    <node id="n1">
      <data key="weight">fixedbitset</data>
    </node>
    <edge id="e0" source="n0" target="n1">
      <data key="weight">depends on</data>
    </edge>
  </graph>"#;
        let expected2 = r#"<key id="weight" for="edge" attr.name="weight" attr.type="string" />"#;
        let expected3 = r#"<key id="weight" for="node" attr.name="weight" attr.type="string" />"#;
        let expected4 = r#"</graphml>"#;

        // HashSet output is unordered, therefore we do not know the order of the keys
        assert!(xml.starts_with(expected1));
        assert!(xml.contains(expected2));
        assert!(xml.contains(expected3));
        assert!(xml.ends_with(expected4));
    }
}
