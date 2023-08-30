/// Edge direction.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub enum Direction {
    /// An `Outgoing` edge is an outward edge *from* the current node.
    Outgoing,
    /// An `Incoming` edge is an inbound edge *to* the current node.
    Incoming,
}

impl Direction {
    #[deprecated(since = "0.1.0")]
    #[inline]
    fn to_usize(self) -> usize {
        match self {
            Self::Outgoing => 0,
            Self::Incoming => 1,
        }
    }

    /// Return the opposite `Direction`.
    #[inline]
    pub fn reverse(self) -> Direction {
        match self {
            Self::Outgoing => Self::Incoming,
            Self::Incoming => Self::Outgoing,
        }
    }

    /// Return `0` for `Outgoing` and `1` for `Incoming`.
    #[deprecated(since = "0.1.0")]
    #[inline]
    pub fn index(self) -> usize {
        self.to_usize() & 0x1
    }
}
