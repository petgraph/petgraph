use core::{fmt::Debug, mem, mem::ManuallyDrop};

use crate::slab::generation::Generation;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum State<V> {
    Vacant,
    Occupied(V),
}

impl<V> From<State<V>> for Option<V> {
    fn from(state: State<V>) -> Self {
        match state {
            State::Occupied(value) => Some(value),
            State::Vacant => None,
        }
    }
}

pub(super) union PackedState<V> {
    pub(crate) occupied: ManuallyDrop<V>,
    vacant: (),
}

/// Information about an entry in the slab.
///
/// ```text
/// MSB         LSB
/// A B C D E F G H
///
/// A: 0 = vacant, 1 = occupied
/// B: 0 = not dropped, 1 = dropped
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Info(u8);

impl Info {
    const fn new() -> Self {
        Self(0)
    }

    const fn is_occupied(self) -> bool {
        self.0 & 0b1000_0000 != 0
    }

    fn set_occupied(&mut self) {
        self.0 |= 0b1000_0000;
    }

    pub(crate) const fn is_vacant(self) -> bool {
        !self.is_occupied()
    }

    fn set_vacant(&mut self) {
        self.0 &= 0b0111_1111;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct EntryProxy<'a, V> {
    generation: Generation,
    state: State<&'a V>,
}

pub(super) struct Entry<V> {
    pub(super) info: Info,
    pub(super) generation: Generation,
    pub(super) state: PackedState<V>,
}

impl<V> Entry<V> {
    pub(crate) const fn new() -> Self {
        Self {
            info: Info::new(),
            generation: Generation::first(),
            state: PackedState { vacant: () },
        }
    }

    pub(crate) fn from_parts(generation: Generation, state: State<V>) -> Self {
        let mut this = Self {
            info: Info::new(),
            generation,
            state: PackedState { vacant: () },
        };

        match state {
            State::Occupied(value) => {
                this.state.occupied = ManuallyDrop::new(value);
                this.info.set_occupied();

                this
            }
            State::Vacant => this,
        }
    }

    pub(crate) const fn is_occupied(&self) -> bool {
        self.info.is_occupied()
    }

    pub(crate) const fn is_vacant(&self) -> bool {
        self.info.is_vacant()
    }

    fn state(&self) -> State<&V> {
        if self.is_occupied() {
            // SAFETY: we know that the entry is occupied
            unsafe { State::Occupied(&*self.state.occupied) }
        } else {
            State::Vacant
        }
    }

    pub(crate) fn state_mut(&mut self) -> State<&mut V> {
        if self.is_occupied() {
            // SAFETY: we know that the entry is occupied
            unsafe { State::Occupied(&mut *self.state.occupied) }
        } else {
            State::Vacant
        }
    }

    pub(crate) fn remove(&mut self) -> Option<V> {
        if self.is_vacant() {
            return None;
        }

        self.generation = self.generation.next();
        self.info.set_vacant();
        let state = mem::replace(&mut self.state, PackedState { vacant: () });

        // SAFETY: we know that the entry is occupied
        Some(unsafe { ManuallyDrop::into_inner(state.occupied) })
    }

    pub(crate) fn insert(&mut self, value: V) {
        // we increment the generation on remove!
        self.state = PackedState {
            occupied: ManuallyDrop::new(value),
        };
        self.info.set_occupied();
    }

    pub(crate) fn get(&self) -> Option<&V> {
        if self.is_vacant() {
            return None;
        }

        // SAFETY: we know that the entry is occupied
        Some(unsafe { &*self.state.occupied })
    }

    pub(crate) fn get_mut(&mut self) -> Option<&mut V> {
        if self.is_vacant() {
            return None;
        }

        // SAFETY: we know that the entry is occupied
        Some(unsafe { &mut *self.state.occupied })
    }

    pub(crate) fn into_inner(mut self) -> Option<V> {
        if self.is_vacant() {
            return None;
        }

        // by removing the value here, we can take ownership of the inner value
        self.remove()
    }
}

impl<V> Drop for Entry<V> {
    fn drop(&mut self) {
        if self.is_occupied() {
            // SAFETY: we know that the entry is occupied
            unsafe {
                let state = &mut self.state;
                ManuallyDrop::drop(&mut state.occupied);
            }
        }
    }
}

impl<V> Debug for Entry<V>
where
    V: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Entry")
            .field("generation", &self.generation)
            .field("state", &self.state())
            .finish_non_exhaustive()
    }
}

impl<V> Clone for Entry<V>
where
    V: Clone,
{
    fn clone(&self) -> Self {
        let state = if self.is_occupied() {
            // SAFETY: we know that the entry is occupied
            unsafe {
                PackedState {
                    occupied: self.state.occupied.clone(),
                }
            }
        } else {
            PackedState { vacant: () }
        };

        Self {
            info: self.info,
            generation: self.generation,
            state,
        }
    }
}

impl<V> PartialEq for Entry<V>
where
    V: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.generation == other.generation && self.state() == other.state()
    }
}

impl<V> Eq for Entry<V> where V: Eq {}

impl<V> PartialOrd for Entry<V>
where
    V: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.generation
            .partial_cmp(&other.generation)
            .and_then(|ordering| match ordering {
                core::cmp::Ordering::Equal => self.state().partial_cmp(&other.state()),
                _ => Some(ordering),
            })
    }
}

impl<V> Ord for Entry<V>
where
    V: Ord,
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.generation
            .cmp(&other.generation)
            .then_with(|| self.state().cmp(&other.state()))
    }
}
