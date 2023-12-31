use crate::{GraphId, GraphStorage};

// TODO: Entry API

pub trait AttributeMapper<K, V> {
    type Iter<'a>: Iterator<Item = (K, &'a V)>
    where
        V: 'a,
        Self: 'a;

    fn get(&self, id: K) -> Option<&V>;
    fn get_mut(&mut self, id: K) -> Option<&mut V>;
    fn index(&self, id: K) -> &V {
        self.get(id).expect("item")
    }
    fn index_mut(&mut self, id: K) -> &mut V {
        self.get_mut(id).expect("item")
    }

    fn set(&mut self, id: K, value: V) -> Option<V>;
    fn remove(&mut self, id: K) -> Option<V>;

    fn iter(&self) -> Self::Iter<'_>;
}

pub trait BooleanMapper<K> {
    fn get(&self, id: K) -> Option<bool>;
    #[inline]
    fn index(&self, id: K) -> bool {
        self.get(id).unwrap_or(false)
    }

    fn set(&mut self, id: K, flag: bool) -> Option<bool>;
}

pub trait AssociativeGraphId<S>: GraphId + Sized
where
    S: GraphStorage,
{
    type AttributeMapper<'a, V>: AttributeMapper<Self, V>
    where
        S: 'a;

    type BooleanMapper<'a>: BooleanMapper<Self>
    where
        S: 'a;

    fn attribute_mapper<V>(storage: &S) -> Self::AttributeMapper<'_, V>;

    fn boolean_mapper(storage: &S) -> Self::BooleanMapper<'_>;
}
