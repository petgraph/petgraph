//! Simple graphml file format output.
#![allow(dead_code)]

use std::borrow::Cow;
use std::collections::HashSet;
use std::fmt;
use std::io::Cursor;
use visit::EdgeRef;
use visit::GraphProp;
use visit::GraphRef;
use visit::IntoEdgeReferences;
use visit::IntoNodeReferences;
use visit::NodeIndexable;
use visit::NodeRef;
use xml::common::XmlVersion;
use xml::writer::events::XmlEvent;
use xml::writer::EventWriter;
use xml::writer::Result as WriterResult;
use xml::EmitterConfig;

static NAMESPACE_URL: &str = "http://graphml.graphdrawing.org/xmlns";

pub struct GraphML<G> {
    graph: G,
    config: Config,
}

impl<G> GraphML<G>
where
    G: GraphRef,
{
    pub fn new(graph: G) -> Self {
        Self::with_config(graph, Config::default())
    }
    pub fn with_config(graph: G, config: Config) -> Self {
        Self { graph, config }
    }

    pub fn to_string(&self) -> String
    where
        G: GraphProp,
        G: IntoEdgeReferences,
        G: IntoNodeReferences,
        G: NodeIndexable,
        G::EdgeWeight: fmt::Display,
        G::NodeWeight: fmt::Display,
    {
        let mut buff = Cursor::new(Vec::new());
        {
            // let mut writer = EventWriter::new(&mut buff);
            let mut writer =
                EventWriter::new_with_config(&mut buff, EmitterConfig::new().perform_indent(true));
            self.do_write(
                &mut writer,
                |ew| vec![("weight".into(), format!("{}", ew).into())],
                |nw| vec![("weight".into(), format!("{}", nw).into())],
            ).expect("Creating a GraphML output should never cause any errors");
        }
        String::from_utf8(buff.into_inner()).unwrap()
    }

    pub fn to_string_with_label_functions<EL, NL>(&self, edge_labels: EL, node_labels: NL) -> String
    where
        G: GraphProp,
        G: IntoEdgeReferences,
        G: IntoNodeReferences,
        G: NodeIndexable,
        for<'a> EL: Fn(&'a G::EdgeWeight) -> Vec<(Cow<'a, str>, Cow<'a, str>)>,
        for<'b> NL: Fn(&'b G::NodeWeight) -> Vec<(Cow<'b, str>, Cow<'b, str>)>,
    {
        let mut buff = Cursor::new(Vec::new());
        {
            // let mut writer = EventWriter::new(&mut buff);
            let mut writer =
                EventWriter::new_with_config(&mut buff, EmitterConfig::new().perform_indent(true));
            self.do_write(&mut writer, edge_labels, node_labels)
                .expect("Creating a GraphML output should never cause any errors");
        }
        String::from_utf8(buff.into_inner()).unwrap()
    }

    fn do_write<EL, NL, W>(
        &self,
        writer: &mut EventWriter<W>,
        edge_labels: EL,
        node_labels: NL,
    ) -> WriterResult<()>
    where
        G: GraphProp,
        G: IntoNodeReferences,
        G: IntoEdgeReferences,
        G: NodeIndexable,
        W: ::std::io::Write,
        for<'a> EL: Fn(&'a G::EdgeWeight) -> Vec<(Cow<'a, str>, Cow<'a, str>)>,
        for<'b> NL: Fn(&'b G::NodeWeight) -> Vec<(Cow<'b, str>, Cow<'b, str>)>,
    {
        #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
        enum For {
            Node,
            Edge,
        };
        impl For {
            fn to_string(&self) -> &'static str {
                match *self {
                    For::Node => "node",
                    For::Edge => "edge",
                }
            }
        }
        #[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
        struct Attribute<'l> {
            name: Cow<'l, str>,
            for_: For,
        };
        let mut attributes: HashSet<Attribute> = HashSet::new();

        writer.write(XmlEvent::StartDocument {
            version: XmlVersion::Version10,
            encoding: Some("UTF-8"),
            standalone: None,
        })?;
        writer.write(XmlEvent::start_element("graphml").attr("xmlns", NAMESPACE_URL))?;
        writer.write(XmlEvent::start_element("graph").attr(
            "edgedefault",
            if self.graph.is_directed() {
                "directed"
            } else {
                "undirected"
            },
        ))?;
        for node in self.graph.node_references() {
            writer.write(
                XmlEvent::start_element("node")
                    .attr("id", &format!("n{}", self.graph.to_index(node.id()))),
            )?;
            // Print weights/labels
            if self.config.node_labels {
                let datas = node_labels(&node.weight());
                for (name, data) in datas {
                    writer.write(XmlEvent::start_element("data").attr("key", &*name))?;
                    attributes.insert(Attribute {
                        name: name.into_owned().into(),
                        for_: For::Node,
                    });
                    writer.write(XmlEvent::characters(&*data))?;
                    writer.write(XmlEvent::end_element())?; // data
                }
            }
            writer.write(XmlEvent::end_element())?; // node
        }
        for (i, edge) in self.graph.edge_references().enumerate() {
            writer.write(
                XmlEvent::start_element("edge")
                    .attr("id", &format!("e{}", i))
                    .attr(
                        "source",
                        &format!("n{}", self.graph.to_index(edge.source())),
                    )
                    .attr(
                        "target",
                        &format!("n{}", self.graph.to_index(edge.target())),
                    ),
            )?;
            // Print weights/labels
            if self.config.edge_labels {
                let datas = edge_labels(&edge.weight());
                for (name, data) in datas {
                    writer.write(XmlEvent::start_element("data").attr("key", &*name))?;
                    attributes.insert(Attribute {
                        name: name.into_owned().into(),
                        for_: For::Edge,
                    });
                    writer.write(XmlEvent::characters(&*data))?;
                    writer.write(XmlEvent::end_element())?; // data
                }
            }
            writer.write(XmlEvent::end_element())?; // node
        }
        writer.write(XmlEvent::end_element())?; // graph

        for attr in attributes {
            writer.write(
                XmlEvent::start_element("key")
                    .attr("id", &*attr.name)
                    .attr("for", attr.for_.to_string())
                    .attr("attr.name", &*attr.name)
                    .attr("attr.type", "string"),
            )?;
            writer.write(XmlEvent::end_element())?; // key
        }

        writer.write(XmlEvent::end_element())?; // graphml
        Ok(())
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Config {
    node_labels: bool,
    edge_labels: bool,
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn export_edge_labels(mut self, state: bool) -> Self {
        self.edge_labels = state;
        self
    }

    pub fn export_node_labels(mut self, state: bool) -> Self {
        self.node_labels = state;
        self
    }
}

#[cfg(test)]
mod test {
    use super::Config;
    use super::GraphML;
    use graph::Graph;

    #[test]
    fn single_node() {
        let mut deps = Graph::<&str, &str>::new();
        deps.add_node("petgraph");

        let graphml = GraphML::new(&deps);
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
    fn single_node_with_display_label() {
        let mut deps = Graph::<&str, &str>::new();
        deps.add_node("petgraph");

        let graphml = GraphML::with_config(&deps, Config::new().export_node_labels(true));
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

        let graphml = GraphML::new(&deps);
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
    fn single_edge_with_display_label() {
        let mut deps = Graph::<&str, &str>::new();
        let pg = deps.add_node("petgraph");
        let fb = deps.add_node("fixedbitset");
        deps.update_edge(pg, fb, "depends on");

        let graphml = GraphML::with_config(&deps, Config::new().export_edge_labels(true));
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
    fn node_and_edge_display_label() {
        let mut deps = Graph::<&str, &str>::new();
        let pg = deps.add_node("petgraph");
        let fb = deps.add_node("fixedbitset");
        deps.update_edge(pg, fb, "depends on");

        let graphml = GraphML::with_config(
            &deps,
            Config::new()
                .export_edge_labels(true)
                .export_node_labels(true),
        );
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
