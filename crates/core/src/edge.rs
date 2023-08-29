// Index into the NodeIndex and EdgeIndex arrays
/// Edge direction.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub enum Direction {
    /// An `Outgoing` edge is an outward edge *from* the current node.
    Outgoing,
    /// An `Incoming` edge is an inbound edge *to* the current node.
    Incoming,
}

impl Direction {
    #[inline]
    fn to_usize(self) -> usize {
        match self {
            Self::Outgoing => 0,
            Self::Incoming => 1,
        }
    }

    /// Return the opposite `Direction`.
    #[inline]
    pub fn opposite(self) -> Direction {
        match self {
            Self::Outgoing => Self::Incoming,
            Self::Incoming => Self::Outgoing,
        }
    }

    /// Return `0` for `Outgoing` and `1` for `Incoming`.
    #[inline]
    pub fn index(self) -> usize {
        self.to_usize() & 0x1
    }
}

#[deprecated(
    since = "0.1.0",
    note = "use `Direction::Incoming` or `Direction::Outgoing` instead"
)]
pub use Direction::{Incoming, Outgoing};

/// Marker type for a directed graph.
#[derive(Copy, Clone, Debug)]
pub struct Directed;

/// Marker type for an undirected graph.
#[derive(Copy, Clone, Debug)]
pub struct Undirected;

/// A graph's edge type determines whether it has directed edges or not.
pub trait EdgeType {
    fn is_directed() -> bool;
}

impl EdgeType for Directed {
    #[inline]
    fn is_directed() -> bool {
        true
    }
}

impl EdgeType for Undirected {
    #[inline]
    fn is_directed() -> bool {
        false
    }
}

// TODO: include graph

pub struct Edge<'a, E: ?Sized, N: ?Sized, W: ?Sized> {
    id: &'a E,

    source: &'a N,
    target: &'a N,

    weight: &'a W,
}

impl<'a, E, N, W> Edge<'a, E, N, W>
where
    E: ?Sized,
    N: ?Sized,
    W: ?Sized,
{
    pub fn id(&self) -> &'a E {
        self.id
    }

    pub fn source(&self) -> &'a N {
        self.source
    }

    pub fn target(&self) -> &'a N {
        self.target
    }

    pub fn weight(&self) -> &'a W {
        self.weight
    }
}

pub struct EdgeMut<'a, E, N, W> {
    id: &'a E,

    source: &'a N,
    target: &'a N,

    weight: &'a mut W,
}

impl<'a, E, N, W> EdgeMut<'a, E, N, W> {
    pub fn id(&self) -> &'a E {
        self.id
    }

    pub fn source(&self) -> &'a N {
        self.source
    }

    pub fn target(&self) -> &'a N {
        self.target
    }

    pub fn weight(&self) -> &'a W {
        self.weight
    }

    pub fn weight_mut(&mut self) -> &'a mut W {
        self.weight
    }
}

pub struct DetachedEdge<E, N, W> {
    pub id: E,

    pub source: N,
    pub target: N,

    pub weight: W,
}

impl<E, N, W> DetachedEdge<E, N, W> {
    pub fn new(id: E, source: N, target: N, weight: W) -> Self {
        Self {
            id,
            source,
            target,
            weight,
        }
    }
}
