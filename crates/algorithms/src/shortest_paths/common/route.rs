use core::{
    fmt::{Debug, Display},
    hash::Hash,
};

use petgraph_core::{GraphStorage, Node};

use crate::shortest_paths::common::{cost::Cost, path::Path};

pub struct Route<'graph, S, T>
where
    S: GraphStorage,
{
    path: Path<'graph, S>,

    cost: Cost<T>,
}

impl<'graph, S, T> Route<'graph, S, T>
where
    S: GraphStorage,
{
    pub const fn new(path: Path<'graph, S>, cost: Cost<T>) -> Self {
        Self { path, cost }
    }

    pub const fn path(&self) -> &Path<'graph, S> {
        &self.path
    }

    pub const fn cost(&self) -> &Cost<T> {
        &self.cost
    }

    pub fn into_cost(self) -> Cost<T> {
        self.cost
    }

    pub fn into_path(self) -> Path<'graph, S> {
        self.path
    }

    pub fn into_parts(self) -> (Path<'graph, S>, Cost<T>) {
        (self.path, self.cost)
    }

    pub(in crate::shortest_paths) fn reverse(self) -> Self {
        Self {
            path: self.path.reverse(),
            cost: self.cost,
        }
    }
}

// ensure that all traits have been implemented
// see: https://rust-lang.github.io/api-guidelines/interoperability.html
#[cfg(test)]
static_assertions::assert_impl_all!(Route<'_, petgraph_dino::DinoStorage<(), ()>, &'static str>: Debug, Display, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Send, Sync);

impl<'graph, S, T> Debug for Route<'graph, S, T>
where
    S: GraphStorage,
    Node<'graph, S>: Debug,
    T: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Route")
            .field("path", &self.path)
            .field("cost", &self.cost)
            .finish()
    }
}

impl<S, T> Display for Route<'_, S, T>
where
    S: GraphStorage,
    T: Display,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&self.path, f)?;
        f.write_str(" (")?;
        Display::fmt(&self.cost, f)?;
        f.write_str(")")
    }
}

impl<'graph, S, T> Clone for Route<'graph, S, T>
where
    S: GraphStorage,
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            cost: self.cost.clone(),
        }
    }
}

impl<'graph, S, T> PartialEq for Route<'graph, S, T>
where
    S: GraphStorage,
    Path<'graph, S>: PartialEq,
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        (&self.path, &self.cost) == (&other.path, &other.cost)
    }
}

impl<'graph, S, T> Eq for Route<'graph, S, T>
where
    S: GraphStorage,
    Path<'graph, S>: Eq,
    T: Eq,
{
}

impl<'graph, S, T> PartialOrd for Route<'graph, S, T>
where
    S: GraphStorage,
    Path<'graph, S>: PartialOrd,
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        // the order is intentional, when ordering we want to order by cost first
        (&self.cost, &self.path).partial_cmp(&(&other.cost, &other.path))
    }
}

impl<'graph, S, T> Ord for Route<'graph, S, T>
where
    S: GraphStorage,
    Path<'graph, S>: Ord,
    T: Ord,
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        (&self.cost, &self.path).cmp(&(&other.cost, &other.path))
    }
}

impl<'graph, S, T> Hash for Route<'graph, S, T>
where
    S: GraphStorage,
    Path<'graph, S>: Hash,
    T: Hash,
{
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        (&self.path, &self.cost).hash(state);
    }
}

pub struct DirectRoute<'graph, S, T>
where
    S: GraphStorage,
{
    source: Node<'graph, S>,
    target: Node<'graph, S>,

    cost: Cost<T>,
}

impl<'graph, S, T> DirectRoute<'graph, S, T>
where
    S: GraphStorage,
{
    pub const fn new(source: Node<'graph, S>, target: Node<'graph, S>, cost: Cost<T>) -> Self {
        Self {
            source,
            target,
            cost,
        }
    }

    pub const fn source(&self) -> &Node<'graph, S> {
        &self.source
    }

    pub const fn target(&self) -> &Node<'graph, S> {
        &self.target
    }

    pub const fn cost(&self) -> &Cost<T> {
        &self.cost
    }

    pub fn into_endpoints(self) -> (Node<'graph, S>, Node<'graph, S>) {
        (self.source, self.target)
    }

    pub fn into_cost(self) -> Cost<T> {
        self.cost
    }

    pub fn into_parts(self) -> (Node<'graph, S>, Node<'graph, S>, Cost<T>) {
        (self.source, self.target, self.cost)
    }

    pub(in crate::shortest_paths) fn reverse(self) -> Self {
        Self {
            source: self.target,
            target: self.source,
            cost: self.cost,
        }
    }
}

// ensure that all traits have been implemented
// see: https://rust-lang.github.io/api-guidelines/interoperability.html
#[cfg(test)]
static_assertions::assert_impl_all!(DirectRoute<'_, petgraph_dino::DinoStorage<(), ()>, &'static str>: Debug, Display, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Send, Sync);

impl<'graph, S, T> Debug for DirectRoute<'graph, S, T>
where
    S: GraphStorage,
    Node<'graph, S>: Debug,
    T: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DirectRoute")
            .field("source", &self.source)
            .field("target", &self.target)
            .field("cost", &self.cost)
            .finish()
    }
}

impl<S, T> Display for DirectRoute<'_, S, T>
where
    S: GraphStorage,
    T: Display,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&self.source.id(), f)?;
        f.write_str(" -> ")?;
        Display::fmt(&self.target.id(), f)?;
        f.write_str(" (")?;
        Display::fmt(&self.cost, f)?;
        f.write_str(")")
    }
}

impl<'graph, S, T> Clone for DirectRoute<'graph, S, T>
where
    S: GraphStorage,
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            source: self.source,
            target: self.target,
            cost: self.cost.clone(),
        }
    }
}

impl<'graph, S, T> PartialEq for DirectRoute<'graph, S, T>
where
    S: GraphStorage,
    Node<'graph, S>: PartialEq,
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        (&self.source, &self.target, &self.cost) == (&other.source, &other.target, &other.cost)
    }
}

impl<'graph, S, T> Eq for DirectRoute<'graph, S, T>
where
    S: GraphStorage,
    Node<'graph, S>: Eq,
    T: Eq,
{
}

impl<'graph, S, T> PartialOrd for DirectRoute<'graph, S, T>
where
    S: GraphStorage,
    Node<'graph, S>: PartialOrd,
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        // the order is intentional, when ordering we want to order by cost first
        (&self.cost, &self.source, &self.target).partial_cmp(&(
            &other.cost,
            &other.source,
            &other.target,
        ))
    }
}

impl<'graph, S, T> Ord for DirectRoute<'graph, S, T>
where
    S: GraphStorage,
    Node<'graph, S>: Ord,
    T: Ord,
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        (&self.cost, &self.source, &self.target).cmp(&(&other.cost, &other.source, &other.target))
    }
}

impl<'graph, S, T> Hash for DirectRoute<'graph, S, T>
where
    S: GraphStorage,
    Node<'graph, S>: Hash,
    T: Hash,
{
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        (&self.source, &self.target, &self.cost).hash(state);
    }
}

impl<'graph, S, T> From<Route<'graph, S, T>> for DirectRoute<'graph, S, T>
where
    S: GraphStorage,
    T: Clone,
{
    fn from(route: Route<'graph, S, T>) -> Self {
        Self {
            source: route.path.source(),
            target: route.path.target(),
            cost: route.cost,
        }
    }
}
