#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Undirected;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Directed;

mod sealed {
    pub trait Sealed: Copy + 'static {}

    impl Sealed for super::Undirected {}
    impl Sealed for super::Directed {}
}

pub trait GraphDirection: sealed::Sealed {
    fn is_directed() -> bool;
}

impl GraphDirection for Undirected {
    fn is_directed() -> bool {
        false
    }
}
impl GraphDirection for Directed {
    fn is_directed() -> bool {
        true
    }
}
