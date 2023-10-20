//! Attributes for graph elements.
//!
//! This module is used for types that are used to describe the attributes of a graph element on
//! insertion.
//!
//! A consumer is not expected to use this module directly, but instead use the [`From`]
//! implementations for [`Attributes`].
//!
//! This module is exposed to allow for people who like to use a more explicit style to do so and
//! for the [`NoValue`] type, which cannot be created[^1] and is used to signal that an identifier
//! is managed by the storage implementation.
//!
//! [^1]: See source code for [`NoValue`] if you _really really really_ need to construct it.
//!
//! [`GraphStorage::next_node_id`]: crate::storage::GraphStorage::next_node_id
//! [`GraphStorage::next_edge_id`]: crate::storage::GraphStorage::next_edge_id
use crate::id::ArbitraryGraphId;

/// Marker type for `GraphId` which are managed by the graph.
///
/// This type is used to represent an `id` on insertion and deletion that is unused.
/// You normally do not need to construct this value directly, as [`Graph::insert_node`] and
/// [`Graph::insert_edge`] use `Attributes`.
///
/// # Implementation Details
///
/// The existence of this type is under stability guarantee, meaning that it will only be removed or
/// renamed according to `SemVer`, but the internals, such as layout or size, are not.
/// This includes the construction method.
///
/// [`Graph::insert_node`]: crate::graph::Graph::insert_node
/// [`Graph::insert_edge`]: crate::graph::Graph::insert_edge
pub struct NoValue(());

impl NoValue {
    /// Construct a new `NoValue`.
    ///
    /// This is only available for testing purposes.
    #[doc(hidden)]
    #[must_use]
    pub const fn new() -> Self {
        Self(())
    }
}

/// Attributes for graph elements.
///
/// This type is used to represent the attributes of a graph element on insertion.
///
/// This type is completely opaque and is only used internally in the [`Graph`] implementation to
/// allow for transparent insertion using [`From`] implementations for elements that require either
/// of the types of `id`: [`ManagedGraphId`] or [`ArbitraryGraphId`].
///
/// You shouldn't need to construct this type directly, but instead simply use the [`From`]
/// implementation via `graph.insert_node(<weight>)` or `graph.insert_node((<weight>,))` for a
/// [`ManagedGraphId`] or `graph.insert_node((<id>, <weight>))` for an [`ArbitraryGraphId`].
/// This also applies for `insert_edge`.
///
/// [`Graph`]: crate::graph::Graph
/// [`ManagedGraphId`]: crate::id::ManagedGraphId
pub struct Attributes<I, W> {
    pub(crate) id: I,
    pub(crate) weight: W,
}

impl<W> Attributes<NoValue, W> {
    /// Construct a new `Attributes` with the given weight.
    ///
    /// This will not set the `id` of the attributes, and can only be used for graphs where the `id`
    /// of the element must be a [`ManagedGraphId`].
    ///
    /// [`ManagedGraphId`]: crate::id::ManagedGraphId
    pub const fn new(weight: W) -> Self {
        Self {
            id: NoValue(()),
            weight,
        }
    }
}

impl<W> Attributes<NoValue, W> {
    /// Set the `id` of the attributes.
    ///
    /// This will return a new `Attributes` with the given `id`, converting it from attributes that
    /// are only valid for elements that have a [`ManagedGraphId`] as their `id`, to ones that only
    /// have an [`ArbitraryGraphId`].
    ///
    /// [`ManagedGraphId`]: crate::id::ManagedGraphId
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

impl<I, J, W> From<(J, W)> for Attributes<I, W>
where
    I: From<J> + ArbitraryGraphId,
{
    fn from(value: (J, W)) -> Self {
        Self {
            id: I::from(value.0),
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

#[cfg(test)]
mod tests {
    use core::marker::PhantomData;

    use crate::{
        attributes::{Attributes, NoValue},
        ArbitraryGraphId, GraphId, ManagedGraphId,
    };

    #[derive(PartialEq)]
    struct Managed(usize);

    impl GraphId for Managed {
        type AttributeIndex = NoValue;
    }

    impl ManagedGraphId for Managed {}

    #[derive(PartialEq)]
    struct Arbitrary<V>(V);

    impl<V> GraphId for Arbitrary<V>
    where
        V: PartialEq,
    {
        type AttributeIndex = Self;
    }

    impl<V> ArbitraryGraphId for Arbitrary<V> where V: PartialEq {}

    impl<T> From<T> for Arbitrary<T> {
        fn from(value: T) -> Self {
            Self(value)
        }
    }

    struct Fake<T> {
        _marker: PhantomData<T>,
    }

    impl<T> Fake<T>
    where
        T: GraphId,
    {
        fn invoke(attributes: impl Into<Attributes<T::AttributeIndex, usize>>) {
            let _attr = attributes.into();
        }
    }

    #[test]
    fn from() {
        Fake::<Managed>::invoke(2usize);

        Fake::<Arbitrary<usize>>::invoke((2usize, 2usize));
    }
}
