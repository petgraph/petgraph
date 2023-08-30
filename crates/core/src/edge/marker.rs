pub trait GraphDirection {
    fn is_directed() -> bool;
    fn is_undirected() -> bool;
}

pub struct Directed;

impl GraphDirection for Directed {
    fn is_directed() -> bool {
        true
    }

    fn is_undirected() -> bool {
        false
    }
}

pub struct Undirected;

impl GraphDirection for Undirected {
    fn is_directed() -> bool {
        false
    }

    fn is_undirected() -> bool {
        true
    }
}
