use crate::storage::GraphStorage;

pub trait ResizableGraphStorage: GraphStorage {
    fn reserve(&mut self, additional_nodes: usize, additional_edges: usize) {
        self.reserve_nodes(additional_nodes);
        self.reserve_edges(additional_edges);
    }

    fn reserve_nodes(&mut self, additional: usize);
    fn reserve_edges(&mut self, additional: usize);

    fn reserve_exact(&mut self, additional_nodes: usize, additional_edges: usize) {
        self.reserve_exact_nodes(additional_nodes);
        self.reserve_exact_edges(additional_edges);
    }

    fn reserve_exact_nodes(&mut self, additional: usize);
    fn reserve_exact_edges(&mut self, additional: usize);

    fn shrink_to_fit(&mut self) {
        self.shrink_to_fit_nodes();
        self.shrink_to_fit_edges();
    }

    fn shrink_to_fit_nodes(&mut self);
    fn shrink_to_fit_edges(&mut self);
}
