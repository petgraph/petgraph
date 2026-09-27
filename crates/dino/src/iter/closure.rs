use core::{cmp::Ordering, iter::Peekable};

use petgraph_core::{edge::EdgeId, node::NodeId};

use crate::node::NodeClosures;

pub(crate) type NodeIdClosureIter<'a> = core::iter::Copied<core::slice::Iter<'a, NodeId>>;
pub(crate) type EdgeIdClosureIter<'a> = core::iter::Copied<core::slice::Iter<'a, EdgeId>>;

/// Computes the union of two sorted iterators with unique elements.
///
/// The resulting iterator will contain all elements from both iterators, removing duplicates.
struct UnionIterator<I, J>
where
    I: Iterator,
    I::Item: Ord,
    J: Iterator<Item = I::Item>,
{
    left: Peekable<I>,
    right: Peekable<J>,
}

impl<I, J> UnionIterator<I, J>
where
    I: Iterator,
    I::Item: Ord,
    J: Iterator<Item = I::Item>,
{
    fn new(left: I, right: J) -> Self {
        Self {
            left: left.peekable(),
            right: right.peekable(),
        }
    }
}

impl<I, J> Iterator for UnionIterator<I, J>
where
    I: Iterator,
    I::Item: Ord,
    J: Iterator<Item = I::Item>,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let next_left = self.left.peek();
        let next_right = self.right.peek();

        match (next_left, next_right) {
            (Some(a), Some(b)) => {
                match a.cmp(b) {
                    Ordering::Less => self.left.next(),
                    Ordering::Greater => self.right.next(),
                    Ordering::Equal => {
                        // remove duplicates
                        self.left.next();
                        self.right.next()
                    }
                }
            }
            (Some(_), None) => self.left.next(),
            (None, Some(_)) => self.right.next(),
            (None, None) => None,
        }
    }
}

/// Computes the intersection of two sorted iterators with unique elements.
///
/// The resulting iterator will contain all elements that are present in both iterators.
struct IntersectionIterator<I, J>
where
    I: Iterator,
    I::Item: Ord,
    J: Iterator<Item = I::Item>,
{
    left: Peekable<I>,
    right: Peekable<J>,
}

impl<I, J> IntersectionIterator<I, J>
where
    I: Iterator,
    I::Item: Ord,
    J: Iterator<Item = I::Item>,
{
    fn new(left: I, right: J) -> Self {
        Self {
            left: left.peekable(),
            right: right.peekable(),
        }
    }
}

impl<I, J> Iterator for IntersectionIterator<I, J>
where
    I: Iterator,
    I::Item: Ord,
    J: Iterator<Item = I::Item>,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // if either of them are exhausted the intersection is empty
            let Some(next_left) = self.left.peek() else {
                return None;
            };
            let Some(next_right) = self.right.peek() else {
                return None;
            };

            match next_left.cmp(next_right) {
                Ordering::Less => {
                    // left is behind, advance it
                    self.left.next();
                }
                Ordering::Greater => {
                    // right is behind, advance it
                    self.right.next();
                }
                Ordering::Equal => {
                    // both are equal, advance both
                    self.left.next();
                    return self.right.next();
                }
            }
        }
    }
}

pub(crate) struct NeighbourIterator<'a>(
    UnionIterator<NodeIdClosureIter<'a>, NodeIdClosureIter<'a>>,
);

impl<'a> NeighbourIterator<'a> {
    pub(crate) fn new(
        incoming_nodes: NodeIdClosureIter<'a>,
        outgoing_nodes: NodeIdClosureIter<'a>,
    ) -> Self {
        Self(UnionIterator::new(incoming_nodes, outgoing_nodes))
    }
}

impl<'a> Iterator for NeighbourIterator<'a> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

pub(crate) struct EdgeIterator<'a>(UnionIterator<EdgeIdClosureIter<'a>, EdgeIdClosureIter<'a>>);

impl<'a> EdgeIterator<'a> {
    pub(crate) fn new(
        incoming_edges: EdgeIdClosureIter<'a>,
        outgoing_edges: EdgeIdClosureIter<'a>,
    ) -> Self {
        Self(UnionIterator::new(incoming_edges, outgoing_edges))
    }
}

impl<'a> Iterator for EdgeIterator<'a> {
    type Item = EdgeId;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

pub(crate) struct EdgeIntersectionIterator<'a>(
    IntersectionIterator<EdgeIdClosureIter<'a>, EdgeIdClosureIter<'a>>,
);

impl<'a> EdgeIntersectionIterator<'a> {
    pub(crate) fn new(
        incoming_edges: EdgeIdClosureIter<'a>,
        outgoing_edges: EdgeIdClosureIter<'a>,
    ) -> Self {
        Self(IntersectionIterator::new(incoming_edges, outgoing_edges))
    }
}

impl<'a> Iterator for EdgeIntersectionIterator<'a> {
    type Item = EdgeId;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

pub(crate) struct EdgeBetweenIterator<'a>(
    UnionIterator<EdgeIntersectionIterator<'a>, EdgeIntersectionIterator<'a>>,
);

impl<'a> EdgeBetweenIterator<'a> {
    pub(crate) fn new(this: &'a NodeClosures, other: &'a NodeClosures) -> Self {
        Self(UnionIterator::new(
            EdgeIntersectionIterator::new(this.outgoing_edges(), other.incoming_edges()),
            EdgeIntersectionIterator::new(this.incoming_edges(), other.outgoing_edges()),
        ))
    }
}

impl<'a> Iterator for EdgeBetweenIterator<'a> {
    type Item = EdgeId;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

#[cfg(test)]
mod test {
    use alloc::vec::Vec;

    use super::*;
    use crate::slab::{EntryId, Generation, Key};

    fn nid(id: usize) -> NodeId {
        NodeId::from_id(EntryId::new(Generation::first(), id).expect("valid id"))
    }

    fn eid(id: usize) -> EdgeId {
        EdgeId::from_id(EntryId::new(Generation::first(), id).expect("valid id"))
    }

    #[test]
    fn union_iterator() {
        let left = [1, 2, 3, 4, 5];
        let right = [2, 4, 6, 8, 10];

        let iter = UnionIterator::new(left.iter().copied(), right.iter().copied());

        assert_eq!(iter.collect::<Vec<_>>(), [1, 2, 3, 4, 5, 6, 8, 10]);
    }

    #[test]
    fn intersection_iterator() {
        let left = [1, 2, 3, 4, 5];
        let right = [2, 4, 6, 8, 10];

        let iter = IntersectionIterator::new(left.iter().copied(), right.iter().copied());

        assert_eq!(iter.collect::<Vec<_>>(), [2, 4]);
    }

    #[test]
    fn intersection_iterator_empty() {
        let left = [1, 2, 3, 4, 5];
        let right = [6, 7, 8, 9, 10];

        let iter = IntersectionIterator::new(left.iter().copied(), right.iter().copied());

        assert_eq!(iter.collect::<Vec<_>>(), []);
    }

    #[test]
    fn neighbour_iterator() {
        let incoming = [nid(1), nid(2), nid(3), nid(4), nid(5)];
        let outgoing = [nid(2), nid(4), nid(6), nid(8), nid(10)];

        let iter = NeighbourIterator::new(incoming.iter().copied(), outgoing.iter().copied());

        assert_eq!(
            iter.collect::<Vec<_>>(),
            [
                nid(1),
                nid(2),
                nid(3),
                nid(4),
                nid(5),
                nid(6),
                nid(8),
                nid(10)
            ]
        );
    }

    #[test]
    fn edge_iterator() {
        let incoming = [eid(1), eid(2), eid(3), eid(4), eid(5)];
        let outgoing = [eid(2), eid(4), eid(6), eid(8), eid(10)];

        let iter = EdgeIterator::new(incoming.iter().copied(), outgoing.iter().copied());

        assert_eq!(
            iter.collect::<Vec<_>>(),
            [
                eid(1),
                eid(2),
                eid(3),
                eid(4),
                eid(5),
                eid(6),
                eid(8),
                eid(10)
            ]
        );
    }

    #[test]
    fn edge_intersection_iterator() {
        let incoming = [eid(1), eid(2), eid(3), eid(4), eid(5)];
        let outgoing = [eid(2), eid(4), eid(6), eid(8), eid(10)];

        let iter =
            EdgeIntersectionIterator::new(incoming.iter().copied(), outgoing.iter().copied());

        assert_eq!(iter.collect::<Vec<_>>(), [eid(2), eid(4)]);
    }
}
