use core::iter::Peekable;

use crate::{EdgeId, NodeId};

pub type NodeIdClosureIter<'a> = core::iter::Copied<core::slice::Iter<'a, NodeId>>;
pub type EdgeIdClosureIter<'a> = core::iter::Copied<core::slice::Iter<'a, EdgeId>>;

pub struct NeighbourIterator<'a> {
    incoming_nodes: Peekable<NodeIdClosureIter<'a>>,
    outgoing_nodes: Peekable<NodeIdClosureIter<'a>>,
}

impl<'a> NeighbourIterator<'a> {
    pub(crate) fn new(
        incoming_nodes: NodeIdClosureIter<'a>,
        outgoing_nodes: NodeIdClosureIter<'a>,
    ) -> Self {
        Self {
            incoming_nodes: incoming_nodes.peekable(),
            outgoing_nodes: outgoing_nodes.peekable(),
        }
    }
}

impl<'a> Iterator for NeighbourIterator<'a> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        let next_incoming = self.incoming_nodes.peek();
        let next_outgoing = self.outgoing_nodes.peek();

        match (next_incoming, next_outgoing) {
            (Some(incoming), Some(outgoing)) => {
                if incoming < outgoing {
                    self.incoming_nodes.next()
                } else {
                    self.outgoing_nodes.next()
                }
            }
            (Some(_), None) => self.incoming_nodes.next(),
            (None, Some(_)) => self.outgoing_nodes.next(),
            (None, None) => None,
        }
    }
}

pub struct EdgeIterator<'a> {
    incoming_edges: Peekable<EdgeIdClosureIter<'a>>,
    outgoing_edges: Peekable<EdgeIdClosureIter<'a>>,
}

impl<'a> EdgeIterator<'a> {
    pub(crate) fn new(
        incoming_edges: EdgeIdClosureIter<'a>,
        outgoing_edges: EdgeIdClosureIter<'a>,
    ) -> Self {
        Self {
            incoming_edges: incoming_edges.peekable(),
            outgoing_edges: outgoing_edges.peekable(),
        }
    }
}

impl<'a> Iterator for EdgeIterator<'a> {
    type Item = EdgeId;

    fn next(&mut self) -> Option<Self::Item> {
        let next_incoming = self.incoming_edges.peek();
        let next_outgoing = self.outgoing_edges.peek();

        match (next_incoming, next_outgoing) {
            (Some(incoming), Some(outgoing)) => {
                if incoming < outgoing {
                    self.incoming_edges.next()
                } else {
                    self.outgoing_edges.next()
                }
            }
            (Some(_), None) => self.incoming_edges.next(),
            (None, Some(_)) => self.outgoing_edges.next(),
            (None, None) => None,
        }
    }
}
