use crate::index::ArbitraryGraphIndex;

pub struct Never(());
pub struct Attributes<I, W> {
    pub(crate) id: I,
    pub(crate) weight: W,
}

impl<W> Attributes<Never, W> {
    pub fn new(weight: W) -> Self {
        Self {
            id: Never(()),
            weight,
        }
    }
}

impl<I, W> Attributes<I, W>
where
    I: ArbitraryGraphIndex,
{
    pub fn new(id: impl Into<I>, weight: W) -> Self {
        Self { id, weight }
    }
}
