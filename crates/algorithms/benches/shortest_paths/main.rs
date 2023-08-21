use criterion::criterion_main;

mod bellman_ford;
#[path = "../common/mod.rs"]
pub mod common;
mod dijkstra;
mod floyd_warshall;
mod k_shortest_path_length;

criterion_main!(
    bellman_ford::benches,
    dijkstra::benches,
    floyd_warshall::benches,
    k_shortest_path_length::benches
);
