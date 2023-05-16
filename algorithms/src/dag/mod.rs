mod toposort;
mod transitive_reduction;

pub use transitive_reduction::{
    dag_to_toposorted_adjacency_list, dag_transitive_reduction_closure,
};
