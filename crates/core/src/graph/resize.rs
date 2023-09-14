use crate::{graph::Graph, storage::GraphStorage};

impl<S> Graph<S>
where
    S: GraphStorage,
{
    pub fn reserve(&mut self, additional_nodes: usize, additional_edges: usize) {
        self.storage.reserve(additional_nodes, additional_edges);
    }

    pub fn reserve_nodes(&mut self, additional: usize) {
        self.storage.reserve_nodes(additional);
    }

    pub fn reserve_edges(&mut self, additional: usize) {
        self.storage.reserve_edges(additional);
    }

    pub fn reserve_exact(&mut self, additional_nodes: usize, additional_edges: usize) {
        self.storage
            .reserve_exact(additional_nodes, additional_edges);
    }

    pub fn reserve_exact_nodes(&mut self, additional: usize) {
        self.storage.reserve_exact_nodes(additional);
    }

    pub fn reserve_exact_edges(&mut self, additional: usize) {
        self.storage.reserve_exact_edges(additional);
    }

    pub fn shrink_to_fit(&mut self) {
        self.storage.shrink_to_fit();
    }

    pub fn shrink_to_fit_nodes(&mut self) {
        self.storage.shrink_to_fit_nodes();
    }

    pub fn shrink_to_fit_edges(&mut self) {
        self.storage.shrink_to_fit_edges();
    }
}
