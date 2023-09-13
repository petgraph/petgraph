use core::mem;

use crate::slab::generation::Generation;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum State<V> {
    Vacant,
    Occupied(V),
}

impl<V> State<V> {
    fn into(self) -> Option<V> {
        match self {
            Self::Occupied(value) => Some(value),
            Self::Vacant => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Entry<V> {
    generation: Generation,
    state: State<V>,
}

impl<V> Entry<V> {
    pub(crate) const fn new() -> Self {
        Self {
            generation: Generation::first(),
            state: State::Vacant,
        }
    }

    const fn is_occupied(&self) -> bool {
        matches!(self.state, State::Occupied(_))
    }

    fn remove(&mut self) -> Option<V> {
        if !self.is_occupied() {
            return None;
        }

        self.generation = self.generation.next();
        mem::replace(&mut self.state, State::Vacant).into()
    }

    pub(crate) fn insert(&mut self, value: V) {
        // we increment the generation on remove!
        self.state = State::Occupied(value);
    }

    const fn get(&self) -> Option<&V> {
        match &self.state {
            State::Occupied(value) => Some(value),
            State::Vacant => None,
        }
    }

    fn get_mut(&mut self) -> Option<&mut V> {
        match &mut self.state {
            State::Occupied(value) => Some(value),
            State::Vacant => None,
        }
    }

    fn into_inner(self) -> Option<V> {
        self.state.into()
    }
}
