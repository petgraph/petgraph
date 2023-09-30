/// A `Direction` is used to specify the direction of an edge.
///
/// Not to be confused with [`crate::edge::marker::GraphDirectionality`],
/// [`crate::edge::marker::Directed`] and [`crate::edge::marker::Undirected`] which serve as markers
/// to specify the directionality of a graph, instead of the direction of an edge in the graph
/// itself.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub enum Direction {
    /// An `Outgoing` edge is an outbound edge *from* the current node.
    ///
    /// e.g. `a -> b` is `Outgoing` from `a` and `Incoming` to `b`.
    Outgoing,
    /// An `Incoming` edge is an inbound edge *to* the current node.
    ///
    /// e.g. `a -> b` is `Outgoing` from `a` and `Incoming` to `b`.
    Incoming,
}

impl Direction {
    #[deprecated(since = "0.1.0")]
    #[inline]
    const fn to_usize(self) -> usize {
        match self {
            Self::Outgoing => 0,
            Self::Incoming => 1,
        }
    }

    /// Return the opposite direction.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::edge::Direction;
    ///
    /// assert_eq!(Direction::Outgoing.reverse(), Direction::Incoming);
    /// assert_eq!(Direction::Incoming.reverse(), Direction::Outgoing);
    /// ```
    #[inline]
    #[must_use]
    pub const fn reverse(self) -> Self {
        match self {
            Self::Outgoing => Self::Incoming,
            Self::Incoming => Self::Outgoing,
        }
    }

    /// Return the index of the direction.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::edge::Direction;
    ///
    /// assert_eq!(Direction::Outgoing.index(), 0);
    /// assert_eq!(Direction::Incoming.index(), 1);
    /// ```
    #[deprecated(
        since = "0.1.0",
        note = "application specific and defeats the purpose of an enum"
    )]
    #[inline]
    #[must_use]
    #[allow(deprecated)]
    pub const fn index(self) -> usize {
        self.to_usize() & 0x1
    }
}
