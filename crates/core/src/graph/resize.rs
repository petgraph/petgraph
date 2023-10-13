use crate::{graph::Graph, storage::GraphStorage};

impl<S> Graph<S>
where
    S: GraphStorage,
{
    /// Reserves capacity for at least `additional_nodes` more nodes and `additional_edges` more
    /// edges to be inserted.
    ///
    /// Depending on the implementation, this may reserve more space than requested or may be a
    /// no-op.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// // we need to explicitly state the type of the node and edge weights,
    /// // as we do insert any nodes/edges and therefore cannot infer them.
    /// let mut graph = DiDinoGraph::<(), ()>::new();
    ///
    /// graph.reserve(16, 16);
    /// ```
    pub fn reserve(&mut self, additional_nodes: usize, additional_edges: usize) {
        self.storage.reserve(additional_nodes, additional_edges);
    }

    /// Reserves capacity for at least `additional_nodes` more nodes to be inserted.
    ///
    /// Depending on the implementation, this may reserve more space than requested or may be a
    /// no-op.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// // we need to explicitly state the type of the node and edge weights, as we do insert
    /// // any nodes/edges and therefore cannot infer them.
    /// let mut graph = DiDinoGraph::<(), ()>::new();
    ///
    /// graph.reserve_nodes(16);
    /// ```
    pub fn reserve_nodes(&mut self, additional: usize) {
        self.storage.reserve_nodes(additional);
    }

    /// Reserves capacity for at least `additional_edges` more edges to be inserted.
    ///
    /// Depending on the implementation, this may reserve more space than requested or may be a
    /// no-op.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// // we need to explicitly state the type of the node and edge weights, as we do insert
    /// // any nodes/edges and therefore cannot infer them.
    /// let mut graph = DiDinoGraph::<(), ()>::new();
    ///
    /// graph.reserve_edges(16);
    /// ```
    pub fn reserve_edges(&mut self, additional: usize) {
        self.storage.reserve_edges(additional);
    }

    /// Reserves the minimum capacity for exactly `additional_nodes` more nodes and
    /// `additional_edges` more edges to be inserted.
    ///
    /// Note that the allocator may give the collection more space than it requests.
    /// Depending on implementation this may also be a no-op.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// // we need to explicitly state the type of the node and edge weights, as we do insert
    /// // any nodes/edges and therefore cannot infer them.
    /// let mut graph = DiDinoGraph::<(), ()>::new();
    ///
    /// graph.reserve_exact(16, 16);
    /// ```
    pub fn reserve_exact(&mut self, additional_nodes: usize, additional_edges: usize) {
        self.storage
            .reserve_exact(additional_nodes, additional_edges);
    }

    /// Reserves the minimum capacity for exactly `additional_nodes` more nodes to be inserted.
    ///
    /// Note that the allocator may give the collection more space than it requests.
    /// Depending on implementation this may also be a no-op.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// // we need to explicitly state the type of the node and edge weights, as we do insert
    /// // any nodes/edges and therefore cannot infer them.
    /// let mut graph = DiDinoGraph::<(), ()>::new();
    ///
    /// graph.reserve_exact_nodes(16);
    /// ```
    pub fn reserve_exact_nodes(&mut self, additional: usize) {
        self.storage.reserve_exact_nodes(additional);
    }

    /// Reserves the minimum capacity for exactly `additional_edges` more edges to be inserted.
    ///
    /// Note that the allocator may give the collection more space than it requests.
    /// Depending on implementation this may also be a no-op.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// // we need to explicitly state the type of the node and edge weights, as we do insert
    /// // any nodes/edges and therefore cannot infer them.
    /// let mut graph = DiDinoGraph::<(), ()>::new();
    ///
    /// graph.reserve_exact_edges(16);
    /// ```
    pub fn reserve_exact_edges(&mut self, additional: usize) {
        self.storage.reserve_exact_edges(additional);
    }

    /// Shrinks the capacity of the storage as much as possible.
    ///
    /// It will drop down as close as possible to the length but the allocator may still inform the
    /// collection that there is space for a few more elements.
    /// Depending on implementation this may also be a no-op.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// // we need to explicitly state the type of the node and edge weights, as we do insert
    /// // any nodes/edges and therefore cannot infer them.
    /// let mut graph = DiDinoGraph::<(), ()>::new();
    ///
    /// graph.shrink_to_fit();
    /// ```
    pub fn shrink_to_fit(&mut self) {
        self.storage.shrink_to_fit();
    }

    /// Shrinks the capacity of the node storage as much as possible.
    ///
    /// It will drop down as close as possible to the length but the allocator may still inform the
    /// collection that there is space for a few more elements.
    /// Depending on implementation this may also be a no-op.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// // we need to explicitly state the type of the node and edge weights, as we do insert
    /// // any nodes/edges and therefore cannot infer them.
    /// let mut graph = DiDinoGraph::<(), ()>::new();
    ///
    /// graph.shrink_to_fit_nodes();
    /// ```
    pub fn shrink_to_fit_nodes(&mut self) {
        self.storage.shrink_to_fit_nodes();
    }

    /// Shrinks the capacity of the edge storage as much as possible.
    ///
    /// It will drop down as close as possible to the length but the allocator may still inform the
    /// collection that there is space for a few more elements.
    /// Depending on implementation this may also be a no-op.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// // we need to explicitly state the type of the node and edge weights, as we do insert
    /// // any nodes/edges and therefore cannot infer them.
    /// let mut graph = DiDinoGraph::<(), ()>::new();
    ///
    /// graph.shrink_to_fit_edges();
    /// ```
    pub fn shrink_to_fit_edges(&mut self) {
        self.storage.shrink_to_fit_edges();
    }
}
