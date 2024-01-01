use alloc::vec::Vec;
use core::{
    cmp::Ordering,
    fmt::{Debug, Display, Formatter},
    hash::{Hash, Hasher},
    iter::once,
};

use petgraph_core::{GraphStorage, Node};

pub struct Path<'graph, S>
where
    S: GraphStorage,
{
    source: Node<'graph, S>,
    target: Node<'graph, S>,

    transit: Vec<Node<'graph, S>>,
}

impl<'graph, S> Path<'graph, S>
where
    S: GraphStorage,
{
    #[must_use]
    pub fn new(
        source: Node<'graph, S>,
        transit: Vec<Node<'graph, S>>,
        target: Node<'graph, S>,
    ) -> Self {
        Self {
            source,
            target,
            transit,
        }
    }

    #[must_use]
    pub const fn source(&self) -> Node<'graph, S> {
        self.source
    }

    #[must_use]
    pub const fn target(&self) -> Node<'graph, S> {
        self.target
    }

    #[must_use]
    pub fn transit(&self) -> &[Node<'graph, S>] {
        &self.transit
    }

    #[must_use]
    pub fn to_vec(self) -> Vec<Node<'graph, S>> {
        let mut vec = Vec::with_capacity(self.transit.len() + 2);

        vec.push(self.source);
        vec.extend(self.transit);
        vec.push(self.target);

        vec
    }

    pub fn iter(&self) -> impl Iterator<Item = Node<'graph, S>> + '_ {
        once(self.source)
            .chain(self.transit.iter().copied())
            .chain(once(self.target))
    }

    pub(in crate::shortest_paths) fn reverse(self) -> Self {
        let mut transit = self.transit;

        transit.reverse();

        Self {
            source: self.target,
            target: self.source,
            transit,
        }
    }
}

// ensure that all traits have been implemented
// see: https://rust-lang.github.io/api-guidelines/interoperability.html
#[cfg(test)]
static_assertions::assert_impl_all!(Path<'_, petgraph_dino::DinoStorage<(), ()>>: Debug, Display, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Send, Sync);

impl<'graph, S> Debug for Path<'graph, S>
where
    S: GraphStorage,
    Node<'graph, S>: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Path")
            .field("source", &self.source)
            .field("target", &self.target)
            .field("transit", &self.transit)
            .finish()
    }
}

impl<S> Display for Path<'_, S>
where
    S: GraphStorage,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&self.source.id(), f)?;

        for node in &self.transit {
            write!(f, " -> {}", node.id())?;
        }

        write!(f, " -> {}", self.target.id())
    }
}

impl<S> Clone for Path<'_, S>
where
    S: GraphStorage,
{
    fn clone(&self) -> Self {
        Self {
            source: self.source,
            target: self.target,
            transit: self.transit.clone(),
        }
    }
}

impl<'graph, S> PartialEq for Path<'graph, S>
where
    S: GraphStorage,
    Node<'graph, S>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        (&self.source, &self.target, &self.transit)
            == (&other.source, &other.target, &other.transit)
    }
}

impl<'graph, S> Eq for Path<'graph, S>
where
    S: GraphStorage,
    Node<'graph, S>: Eq,
{
}

impl<'graph, S> PartialOrd for Path<'graph, S>
where
    S: GraphStorage,
    Node<'graph, S>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        (&self.source, &self.transit, &self.target).partial_cmp(&(
            &other.source,
            &other.transit,
            &other.target,
        ))
    }
}

impl<'graph, S> Ord for Path<'graph, S>
where
    S: GraphStorage,
    Node<'graph, S>: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        (&self.source, &self.transit, &self.target).cmp(&(
            &other.source,
            &other.transit,
            &other.target,
        ))
    }
}

impl<'graph, S> Hash for Path<'graph, S>
where
    S: GraphStorage,
    Node<'graph, S>: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        (&self.source, &self.target, &self.transit).hash(state);
    }
}

impl<'graph, S> IntoIterator for Path<'graph, S>
where
    S: GraphStorage,
{
    type IntoIter = alloc::vec::IntoIter<Node<'graph, S>>;
    type Item = Node<'graph, S>;

    fn into_iter(self) -> Self::IntoIter {
        self.to_vec().into_iter()
    }
}
