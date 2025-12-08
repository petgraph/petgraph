use crate::id::Id;

pub enum Direction {
    Incoming,
    Outgoing,
}

pub trait Edge<'graph> {
    type Id: Id;
    type Node: Id;
    type Weight;

    fn id(&self) -> Self::Id;

    fn source(&self) -> Self::Node;
    fn target(&self) -> Self::Node;

    fn into_weight(self) -> Self::Weight;
    fn weight(&self) -> &Self::Weight;

    fn opposite(&self, direction: Direction) -> Self::Node {
        match direction {
            Direction::Incoming => self.source(),
            Direction::Outgoing => self.target(),
        }
    }
}

pub trait EdgeMut<'graph>: Edge<'graph> {
    fn weight_mut(&mut self) -> &mut Self::Weight;
}
