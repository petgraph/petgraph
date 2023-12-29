use criterion::criterion_main;

// mod bellman_ford;
// #[path = "../common/mod.rs"]
// pub mod common;
// mod dijkstra;
// mod floyd_warshall;
// mod k_shortest_path_length;
mod large;

criterion_main!(large::benches);
