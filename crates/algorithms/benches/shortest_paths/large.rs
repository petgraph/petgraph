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
    let mut workspaces = WORKSPACES.lock().expect("lock workspaces");

    if let Some(path) = workspaces.get(manifest_dir) {
        return Arc::clone(path);
    }

    let output =
        process::Command::new(env::var("CARGO").ok().unwrap_or_else(|| "cargo".to_owned()))
            .arg("metadata")
            .arg("--format-version=1")
            .arg("--no-deps")
            .current_dir(manifest_dir)
            .output()
            .expect("execute `cargo metadata`");

    let manifest = serde_json::from_slice::<serde_json::Value>(&output.stdout).expect("parse json");

    let root = PathBuf::from(manifest["workspace_root"].as_str().expect("workspace root"));
    let root = Arc::from(root);

    workspaces.insert(manifest_dir.to_owned(), Arc::clone(&root));
    root
}

fn input(file: &str) -> Arc<Path> {
    let workspace = get_cargo_workspace();
    let path = workspace.join("crates/algorithms/benches/input").join(file);

    assert!(path.exists(), "{} does not exist", path.display());

    Arc::from(path)
}

macro_rules! next {
    ($iter:ident as $ty:ty) => {
        $iter
            .next()
            .expect("expected token")
            .parse::<$ty>()
            .expect(concat!("is ", stringify!($ty)))
    };
}

fn parse_coordinate_file(path: &Path) -> Vec<(usize, i128, i128)> {
    let contents = fs::read_to_string(path).expect("able to read file");

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

                amount_of_lines = Some(next!(iter as usize));

                assert_eq!(iter.next(), None);
            }
            b'p' => panic!("multiple problem lines"),
            b'v' if amount_of_lines.is_some() => {
                let mut iter = line.split_whitespace();
                assert_eq!(iter.next(), Some("v"));
                let id = next!(iter as usize);
                let x = next!(iter as i128);
                let y = next!(iter as i128);

                assert_eq!(iter.next(), None);

                output.push((id, x, y));

                if let Some(amount_of_lines) = &mut amount_of_lines {
                    *amount_of_lines = amount_of_lines.checked_sub(1).expect("too many lines");
                }
            }
            b'v' => panic!("vertex lines before problem statement"),
            _ => panic!("unknown instruction"),
        }
    }

    output
}

fn parse_graph_file(path: &Path) -> Vec<(usize, usize, u128)> {
    let contents = fs::read_to_string(path).expect("able to read file");

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
                let _number_of_nodes = next!(iter as usize);
                let number_of_edges = next!(iter as usize);
                amount_of_lines = Some(number_of_edges);

                assert_eq!(iter.next(), None);
            }
            b'p' => panic!("multiple problem lines"),
            b'a' if amount_of_lines.is_some() => {
                let mut iter = line.split_whitespace();
                assert_eq!(iter.next(), Some("a"));
                let source = next!(iter as usize);
                let target = next!(iter as usize);
                let weight = next!(iter as u128);

                assert_eq!(iter.next(), None);

                output.push((source, target, weight));

                if let Some(amount_of_lines) = &mut amount_of_lines {
                    *amount_of_lines = amount_of_lines.checked_sub(1).expect("too many lines");
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

    (source.expect("source available"), graph)
}

fn dijkstra(criterion: &mut Criterion) {
    criterion.bench_with_input(
        BenchmarkId::new("dimacs9/florida", "dijkstra"),
        &"USA-road-d.FLA",
        |bench, &filename| {
            let (source, graph) = build_graph(filename);
            let dijkstra = Dijkstra::directed();

            bench.iter(|| {
                for distances in dijkstra
                    .distance_from(&graph, &source)
                    .expect("route available")
                {
                    black_box(distances);
                }
            });
        },
    );
}

criterion_group!(benches, dijkstra);
