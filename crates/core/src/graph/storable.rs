use crate::graph::Graph;

trait Container<Id, Data> {
    fn get(&self, node_id: Id) -> Option<&Data>;
    fn insert(&mut self, node_id: Id, data: Data);
    fn remove(&mut self, node_id: Id);
}

trait Storable: Graph {
    fn node_visit_container<'graph>(&'graph self) -> impl Container<Self::NodeId, ()>
    where
        Self: 'graph;

    fn node_data_container<'graph>(
        &'graph self,
    ) -> impl Container<Self::NodeId, Self::NodeData<'graph>>
    where
        Self: 'graph;

    fn edge_visit_container<'graph>(&'graph self) -> impl Container<Self::EdgeId, ()>
    where
        Self: 'graph;

    fn edge_data_container<'graph>(
        &'graph self,
    ) -> impl Container<Self::EdgeId, Self::EdgeData<'graph>>
    where
        Self: 'graph;
}
