use crate::{EdgeId, NodeId};

pub type NodeIdClosureIter<'a> = core::iter::Copied<core::slice::Iter<'a, NodeId>>;
pub type EdgeIdClosureIter<'a> = core::iter::Copied<core::slice::Iter<'a, EdgeId>>;
