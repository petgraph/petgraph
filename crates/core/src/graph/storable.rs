use crate::graph::Graph;

pub trait Container<Id, Data> {
    fn get(&self, node_id: Id) -> Option<&Data>;
    fn insert(&mut self, node_id: Id, data: Data);
    fn remove(&mut self, node_id: Id);
}

pub trait Storable: Graph {
    fn node_visit_container(&self) -> impl Container<Self::NodeId, ()>;

    fn node_data_container<Data>(&self) -> impl Container<Self::NodeId, Data>;

    fn edge_visit_container(&self) -> impl Container<Self::EdgeId, ()>;

    fn edge_data_container<Data>(&self) -> impl Container<Self::EdgeId, Data>;
}
