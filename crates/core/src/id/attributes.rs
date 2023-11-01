use crate::GraphStorage;
// TODO: naming

pub trait FlagStorage<Id> {
    fn get(&self, id: &Id) -> Option<bool>;
    fn index(&self, id: &Id) -> bool {
        self.get(id).unwrap_or(false)
    }

    fn set(&mut self, id: &Id, flag: bool) -> Option<bool>;
}

pub trait FlaggableGraphId<S>: Sized
where
    S: GraphStorage,
{
    type Store<'a>: FlagStorage<Self>
    where
        S: 'a;

    fn flag_store(storage: &S) -> Self::Store<'_>;
}
