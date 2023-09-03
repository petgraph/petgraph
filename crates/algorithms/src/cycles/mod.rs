mod check;
#[cfg(feature = "remove-me-only-intended-for-move-graph")]
mod feedback_arc_set;

pub use check::{is_cyclic_directed, is_cyclic_undirected};
#[cfg(feature = "remove-me-only-intended-for-move-graph")]
pub use feedback_arc_set::greedy_feedback_arc_set;
