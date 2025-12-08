use crate::id::Id;

pub trait Node<'graph> {
    type Id: Id;
    type Weight;

    fn id(&self) -> Self::Id;

    fn weight(&self) -> Self::Weight;
    fn into_weight(self) -> Self::Weight;
}

pub trait NodeMut<'graph>: Node<'graph> {
    fn weight_mut(&mut self) -> &mut Self::Weight;
}
