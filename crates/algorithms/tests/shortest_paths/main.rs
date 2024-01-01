mod harness;

use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader, Lines},
    path::{Path, PathBuf},
};

use petgraph_algorithms::shortest_paths::{
    BellmanFord, Dijkstra, FloydWarshall, Route, ShortestPath,
};
use petgraph_core::{edge::EdgeId, node::NodeId};
use petgraph_dino::{DiDinoGraph, DinoStorage};
use snapbox::{utils::normalize_lines, Action};

use crate::harness::{Case, Harness};

fn setup(input_path: PathBuf) -> Case {
    let name = input_path
        .file_stem()
        .expect("file stem")
        .to_str()
        .expect("to str")
        .to_owned();

    let expected = input_path.with_extension("out");

    Case {
        name,
        fixture: input_path,
        expected,
    }
}

struct Parse {
    graph: DiDinoGraph<usize, u64>,
    source: NodeId,
    target: NodeId,
    nodes: Vec<NodeId>,
    edges: Vec<EdgeId>,
}

fn read(input_path: &Path) -> Result<Parse, Box<dyn Error>> {
    let file = File::open(input_path)?;
    let reader = BufReader::new(file);

    let input = reader.lines();
    parse(input)
}

fn parse(mut input: Lines<impl BufRead>) -> Result<Parse, Box<dyn Error>> {
    let meta = input.next().expect("first line")?;
    let meta = meta
        .split_whitespace()
        .map(str::parse::<usize>)
        .collect::<Result<Vec<_>, _>>()?;

    let n = meta[0];
    let m = meta[1];
    let s = meta[2];
    let t = meta[3];

    let mut graph = DiDinoGraph::new();

    let mut nodes = Vec::with_capacity(n);
    for index in 0..n {
        nodes.push(graph.insert_node(index).id());
    }

    let mut edges = Vec::with_capacity(m);
    for _ in 0..m {
        let edge = input.next().expect("edge")?;
        let edge = edge
            .split_whitespace()
            .map(str::parse::<usize>)
            .collect::<Result<Vec<_>, _>>()?;

        let u = edge[0];
        let v = edge[1];
        let w = edge[2] as u64;

        let id = graph.insert_edge(w, nodes[u], nodes[v]).id();
        edges.push(id);
    }

    let source = nodes[s];
    let target = nodes[t];

    Ok(Parse {
        graph,
        source,
        target,
        nodes,
        edges,
    })
}

fn dump(route: Route<DinoStorage<usize, u64>, u64>) -> String {
    let x = route.cost().into_value();
    // transit are all intermediate nodes, we have to additional nodes (source and target)
    // so we add `+ 2`, edges are between them, so we subtract `- 1`, resulting in `+ 1`
    let y = route.path().transit().len() + 1;

    let nodes: Vec<_> = route.into_path().into_iter().map(|x| *x.weight()).collect();

    let mut output = vec![];
    output.push(format!("{x} {y}"));

    output.extend(
        nodes
            .iter()
            .zip(nodes.iter().skip(1))
            .map(|(u, v)| format!("{u} {v}")),
    );

    let mut output = output.join("\n");
    output.push('\n');
    output
}

fn test(input_path: &Path, name: &'static str) -> Result<String, Box<dyn Error>> {
    let Parse {
        graph,
        source,
        target,
        ..
    } = read(input_path)?;

    let route = match name {
        "dijkstra" => Dijkstra::directed().path_between(&graph, source, target),
        "bellman_ford" => BellmanFord::directed().path_between(&graph, source, target),
        "floyd_warshall" => FloydWarshall::directed().path_between(&graph, source, target),
        _ => unreachable!(),
    };

    let Some(route) = route else {
        return Ok(normalize_lines("-1\n"));
    };

    Ok(normalize_lines(&dump(route)))
}

pub fn main() {
    Harness::new("tests/cases/shortest_path", setup, test)
        .select(["*.in"])
        .each(&["dijkstra", "bellman_ford", "floyd_warshall"])
        .action(Action::Verify)
        .test();
}
