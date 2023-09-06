use crate::id::ArbitraryGraphId;

pub struct NoValue(());

impl NoValue {
    #[must_use]
    pub const fn new() -> Self {
        Self(())
    }
}

pub struct Attributes<I, W> {
    pub(crate) id: I,
    pub(crate) weight: W,
}

impl<W> Attributes<NoValue, W> {
    pub const fn new(weight: W) -> Self {
        Self {
            id: NoValue(()),
            weight,
        }
    }
}

impl<W> Attributes<NoValue, W> {
    pub fn with_id<I>(self, id: impl Into<I>) -> Attributes<I, W>
    where
        I: ArbitraryGraphId,
    {
        Attributes {
            id: id.into(),
            weight: self.weight,
        }
    }
}

impl<I, W> From<(I, W)> for Attributes<I, W>
where
    I: ArbitraryGraphId,
{
    fn from(value: (I, W)) -> Self {
        Self {
            id: value.0,
            weight: value.1,
        }
    }
}

impl<W> From<(W,)> for Attributes<NoValue, W> {
    fn from((weight,): (W,)) -> Self {
        Self {
            id: NoValue(()),
            weight,
        }
    }
}

impl<W> From<W> for Attributes<NoValue, W> {
    fn from(weight: W) -> Self {
        Self {
            id: NoValue(()),
            weight,
        }
    }
}
