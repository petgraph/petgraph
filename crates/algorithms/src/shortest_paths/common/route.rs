use petgraph_core::{GraphStorage, Node};

use crate::shortest_paths::common::{cost::Cost, path::Path};

pub struct Route<'a, S, T>
where
    S: GraphStorage,
{
    pub(in crate::shortest_paths) path: Path<'a, S>,

    pub(in crate::shortest_paths) cost: Cost<T>,
}

impl<'a, S, T> Route<'a, S, T>
where
    S: GraphStorage,
{
    pub fn path(&self) -> &Path<'a, S> {
        &self.path
    }

    pub fn cost(&self) -> &Cost<T> {
        &self.cost
    }

    pub fn into_cost(self) -> Cost<T> {
        self.cost
    }

    pub fn into_path(self) -> Path<'a, S> {
        self.path
    }

    pub fn into_parts(self) -> (Path<'a, S>, Cost<T>) {
        (self.path, self.cost)
    }

    pub(in crate::shortest_paths) fn reverse(self) -> Self {
        Self {
            path: self.path.reverse(),
            cost: self.cost,
        }
    }
}

pub struct DirectRoute<'a, S, T>
where
    S: GraphStorage,
{
    pub(in crate::shortest_paths) source: Node<'a, S>,
    pub(in crate::shortest_paths) target: Node<'a, S>,

    pub(in crate::shortest_paths) cost: Cost<T>,
}

impl<'a, S, T> DirectRoute<'a, S, T>
where
    S: GraphStorage,
{
    pub fn source(&self) -> &Node<'a, S> {
        &self.source
    }

    pub fn target(&self) -> &Node<'a, S> {
        &self.target
    }

    pub fn cost(&self) -> &Cost<T> {
        &self.cost
    }

    pub fn into_endpoints(self) -> (Node<'a, S>, Node<'a, S>) {
        (self.source, self.target)
    }

    pub fn into_cost(self) -> Cost<T> {
        self.cost
    }

    pub fn into_parts(self) -> (Node<'a, S>, Node<'a, S>, Cost<T>) {
        (self.source, self.target, self.cost)
    }

    fn reverse(self) -> Self {
        Self {
            source: self.target,
            target: self.source,
            cost: self.cost,
        }
    }
}
