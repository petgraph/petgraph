use std::{
    collections::{BTreeMap, HashMap},
    env, fs,
    hint::black_box,
    path::{Path, PathBuf},
    process,
    sync::{Arc, Mutex},
};

use criterion::{criterion_group, BenchmarkId, Criterion};
use petgraph_algorithms::shortest_paths::{Dijkstra, ShortestDistance};
use petgraph_dino::{DiDinoGraph, NodeId};

fn get_cargo_workspace() -> Arc<Path> {
    static WORKSPACES: Mutex<BTreeMap<String, Arc<Path>>> = Mutex::new(BTreeMap::new());

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let mut workspaces = WORKSPACES.lock().unwrap();

    if let Some(path) = workspaces.get(manifest_dir) {
        return Arc::clone(path);
    }

    let output = process::Command::new(
        env::var("CARGO")
            .ok()
            .unwrap_or_else(|| "cargo".to_string()),
    )
    .arg("metadata")
    .arg("--format-version=1")
    .arg("--no-deps")
    .current_dir(manifest_dir)
    .output()
    .unwrap();

    let manifest = serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap();

    let root = PathBuf::from(manifest["workspace_root"].as_str().unwrap());
    let root = Arc::from(root);

    workspaces.insert(manifest_dir.to_string(), Arc::clone(&root));
    root
}

fn input(file: &str) -> Arc<Path> {
    let workspace = get_cargo_workspace();
    let path = workspace.join("crates/algorithms/benches/input").join(file);

    if !path.exists() {
        panic!("{} does not exist", path.display());
    }

    Arc::from(path)
}

fn parse_coordinate_file(path: &Path) -> Vec<(usize, i128, i128)> {
    let contents = fs::read_to_string(path).unwrap();

    let mut output = vec![];
    let mut amount_of_lines = None;

    for line in contents.lines() {
        let type_ = line.as_bytes()[0];

        match type_ {
            b'c' => continue,
            b'p' if amount_of_lines.is_none() => {
                let mut iter = line.split_whitespace();
                assert_eq!(iter.next(), Some("p"));
                assert_eq!(iter.next(), Some("aux"));
                assert_eq!(iter.next(), Some("sp"));
                assert_eq!(iter.next(), Some("co"));

                amount_of_lines = Some(iter.next().unwrap().parse::<usize>().unwrap());

                assert_eq!(iter.next(), None);
            }
            b'p' => panic!("multiple problem lines"),
            b'v' if amount_of_lines.is_some() => {
                let mut iter = line.split_whitespace();
                assert_eq!(iter.next(), Some("v"));
                let id = iter.next().unwrap().parse::<usize>().unwrap();
                let x = iter.next().unwrap().parse::<i128>().unwrap();
                let y = iter.next().unwrap().parse::<i128>().unwrap();

                assert_eq!(iter.next(), None);

                output.push((id, x, y));

                if let Some(amount_of_lines) = &mut amount_of_lines {
                    *amount_of_lines = amount_of_lines.checked_sub(1).expect("too many lines")
                }
            }
            b'v' => panic!("vertex lines before problem statement"),
            _ => panic!("unknown instruction"),
        }
    }

    output
}

fn parse_graph_file(path: &Path) -> Vec<(usize, usize, u128)> {
    let mut contents = fs::read_to_string(path).unwrap();

    let mut output = vec![];
    let mut amount_of_lines = None;

    for line in contents.lines() {
        let type_ = line.as_bytes()[0];

        match type_ {
            b'c' => continue,
            b'p' if amount_of_lines.is_none() => {
                let mut iter = line.split_whitespace();
                assert_eq!(iter.next(), Some("p"));
                assert_eq!(iter.next(), Some("sp"));
                let _number_of_nodes = iter.next().unwrap().parse::<usize>().unwrap();
                let number_of_edges = iter.next().unwrap().parse::<usize>().unwrap();
                amount_of_lines = Some(number_of_edges);

                assert_eq!(iter.next(), None);
            }
            b'p' => panic!("multiple problem lines"),
            b'a' if amount_of_lines.is_some() => {
                let mut iter = line.split_whitespace();
                assert_eq!(iter.next(), Some("a"));
                let source = iter.next().unwrap().parse::<usize>().unwrap();
                let target = iter.next().unwrap().parse::<usize>().unwrap();
                let weight = iter.next().unwrap().parse::<u128>().unwrap();

                assert_eq!(iter.next(), None);

                output.push((source, target, weight));

                if let Some(amount_of_lines) = &mut amount_of_lines {
                    *amount_of_lines = amount_of_lines.checked_sub(1).expect("too many lines")
                }
            }
            b'a' => panic!("edge lines before problem statement"),
            _ => panic!("unknown instruction"),
        }
    }

    output
}

struct Node {
    id: usize,
    x: i128,
    y: i128,
}

fn build_graph(filename: &str) -> (NodeId, DiDinoGraph<Node, u128>) {
    let nodes = parse_coordinate_file(&input(format!("{filename}.co").as_ref()));
    let edges = parse_graph_file(&input(format!("{filename}.gr").as_ref()));

    let mut lookup = HashMap::new();
    let mut source = None;

    let mut graph = DiDinoGraph::<Node, u128>::with_capacity(Some(nodes.len()), Some(edges.len()));

    for (id, x, y) in nodes {
        let node_id = *graph.insert_node(Node { id, x, y }).id();
        lookup.insert(id, node_id);

        if source.is_none() {
            source = Some(node_id);
        }
    }

    for (source, target, weight) in edges {
        let source = lookup[&source];
        let target = lookup[&target];

        graph.insert_edge(weight, &source, &target);
    }

    (source.unwrap(), graph)
}

fn dijkstra(criterion: &mut Criterion) {
    criterion.bench_with_input(
        // BenchmarkId::from_parameter("dimacs9/florida"),
        BenchmarkId::new("dimacs9/florida", "dijkstra"),
        &"USA-road-d.FLA",
        |bench, &filename| {
            let (source, graph) = build_graph(filename);
            let dijkstra = Dijkstra::directed();

            bench.iter(|| {
                let _scores: Vec<_> = dijkstra.distance_from(&graph, &source).unwrap().collect();
            });
        },
    );
}

criterion_group!(benches, dijkstra);
