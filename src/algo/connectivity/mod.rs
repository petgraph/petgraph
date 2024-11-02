pub use cut_edges::CutEdgesSearch;

mod cut_edges;

/// Marker type used in DFS searches.
#[derive(Debug, PartialEq)]
pub enum Color {
    Gray,
    Black,
}
