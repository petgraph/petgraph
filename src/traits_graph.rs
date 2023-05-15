use fixedbitset::FixedBitSet;

use super::{
    graph::{Graph, IndexType, NodeIndex},
    visit::GetAdjacencyMatrix,
    EdgeType,
};
#[cfg(feature = "stable_graph")]
use crate::stable_graph::StableGraph;
use crate::visit::EdgeRef;
#[cfg(feature = "stable_graph")]
use crate::visit::{IntoEdgeReferences, NodeIndexable};
