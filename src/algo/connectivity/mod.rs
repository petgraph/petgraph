pub use biconnected_components::BiconnectedComponentsSearch;
pub use cut_edges::CutEdgesSearch;
pub use cut_vertices::CutVerticesSearch;
pub use two_edge_connected_components::TwoEdgeConnectedComponentsSearch;

mod biconnected_components;
mod cut_edges;
mod cut_vertices;
mod two_edge_connected_components;

/// Marker type used in DFS searches.
#[derive(Debug, PartialEq)]
pub enum Color {
    Gray,
    Black,
}
