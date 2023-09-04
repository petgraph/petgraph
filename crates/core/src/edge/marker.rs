pub struct Undirected;
pub struct Directed;

mod sealed {
    pub trait Sealed {}

    impl Sealed for super::Undirected {}
    impl Sealed for super::Directed {}
}

pub trait EdgeDirection: sealed::Sealed {}

impl EdgeDirection for Undirected {}
impl EdgeDirection for Directed {}
