mod toposort;
#[cfg(feature = "remove-me-only-intended-for-move-adjacency-matrix")]
mod transitive_reduction;

pub use toposort::toposort;
#[cfg(feature = "remove-me-only-intended-for-move-adjacency-matrix")]
pub use transitive_reduction::{
    dag_to_toposorted_adjacency_list, dag_transitive_reduction_closure,
};
