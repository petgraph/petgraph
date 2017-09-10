use std::collections::HashSet;
use std::mem;
use std::ops::{Index, IndexMut};

#[derive(Clone)]
pub struct DenseMapping<T> {
    elements: Vec<Option<T>>,
    ids: IdGenerator,
}

impl<T: Clone> DenseMapping<T> {
    pub fn with_capacity(capacity: usize) -> Self {
        DenseMapping {
            elements: Vec::with_capacity(capacity),
            ids: IdGenerator::new(),
        }
    }

    pub fn add(&mut self, element: T) -> (usize, Option<T>) {
        let id: usize = self.ids.add();

        // Resize the vector to make sure we aren't out of bounds
        let elements_len = self.elements.len();
        if elements_len <= id {
            self.elements.resize(id + 1, None);
        }

        // Swap the previous value with the new value
        let previous_element = mem::replace(&mut self.elements[id], Some(element));

        (id, previous_element)
    }

    pub fn remove(&mut self, id: usize) -> Option<T> {
        let element_exists = self.ids.exists(id);

        self.ids.remove(id);

        if element_exists {
            let old_weight = self.elements[id].clone();
            self.elements[id] = None;
            old_weight
        } else {
            None
        }
    }

    pub fn exists(&self, id: usize) -> bool {
        self.ids.exists(id)
    }

    pub fn size(&self) -> usize {
        self.ids.size()
    }

    pub fn clear(&mut self) {
        self.elements.clear();
        self.ids.clear();
    }

    pub fn iter_ids(&self) -> IdIterator {
        self.ids.iter()
    }
}

impl<T: Clone> Index<usize> for DenseMapping<T> {
    type Output = T;
    fn index(&self, index: usize) -> &T {
        self.elements[index].as_ref().unwrap()
    }
}

impl<T: Clone> IndexMut<usize> for DenseMapping<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        self.elements[index].as_mut().unwrap()
    }
}

#[derive(Clone)]
struct IdGenerator {
    upper_bound: usize,
    removed_ids: HashSet<usize>,
}

impl IdGenerator {
    fn new() -> IdGenerator {
        IdGenerator {
            upper_bound: 0,
            removed_ids: HashSet::new(),
        }
    }

    fn add(&mut self) -> usize {
        if !self.removed_ids.is_empty() {
            let id = self.removed_ids.iter().next().unwrap().clone();
            self.removed_ids.remove(&id);
            id
        } else {
            let id = self.upper_bound;
            self.upper_bound = self.upper_bound + 1;
            id
        }
    }

    fn remove(&mut self, id: usize) {
        if id < self.upper_bound {
            self.removed_ids.insert(id);
        }
    }

    fn exists(&self, id: usize) -> bool {
        id < self.upper_bound && !self.removed_ids.contains(&id)
    }

    fn size(&self) -> usize {
        self.upper_bound - self.removed_ids.len()
    }

    fn clear(&mut self) {
        *self = IdGenerator::new();
    }

    fn iter(&self) -> IdIterator {
        IdIterator::new(self)
    }
}

pub struct IdIterator<'a> {
    ids: &'a IdGenerator,
    current: Option<usize>,
}

impl<'a> IdIterator<'a> {
    fn new(ids: &'a IdGenerator) -> IdIterator<'a> {
        IdIterator {
            ids: ids,
            current: None,
        }
    }
}

impl<'a> Iterator for IdIterator<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        // initialize / advance
        if self.current.is_none() {
            self.current = Some({
                (0..self.ids.upper_bound)
                    .skip_while(|id| self.ids.removed_ids.contains(&id))
                    .next()
                    .unwrap_or(self.ids.upper_bound)
            });
        } else {
            let mut current = self.current.as_mut().unwrap();
            *current += 1;
        }

        let current = self.current.unwrap();
        if current < self.ids.upper_bound {
            Some(current)
        } else {
            None
        }
    }
}
