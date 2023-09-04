use petgraph_core::{edge::marker::GraphDirection, storage::ResizableGraphStorage};

use crate::DinosaurStorage;

impl<N, E, D> ResizableGraphStorage for DinosaurStorage<N, E, D>
where
    D: GraphDirection,
{
    fn reserve_nodes(&mut self, additional: usize) {
        self.nodes.reserve(additional);
        self.closures.nodes.reserve(additional);
    }

    fn reserve_edges(&mut self, additional: usize) {
        self.edges.reserve(additional);
        self.closures.edges.reserve(additional);
    }

    fn shrink_to_fit_nodes(&mut self) {
        self.nodes.shrink_to_fit();
        self.closures.nodes.shrink_to_fit();
    }

    fn shrink_to_fit_edges(&mut self) {
        self.edges.shrink_to_fit();
        self.closures.edges.shrink_to_fit();
    }
}
