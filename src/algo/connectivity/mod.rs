pub use cut_edges::CutEdgesSearch;
pub use cut_vertices::CutVerticesSearch;

mod cut_edges;
mod cut_vertices;

/// Marker type used in DFS searches.
#[derive(Debug, PartialEq)]
pub enum Color {
    Gray,
    Black,
}
