use std::{collections::HashMap, hash::Hash};

use super::Color;
use crate::{algo::IntoNeighbors, visit::IntoNodeIdentifiers};

pub struct CutEdgesSearch<'a, N> {
    /// The map of colors of each node.
    /// If it hasn't any color it means it wasn't visited yet,
    /// if it has gray color it means it is being processed and
    /// if it has black color it means its processing is finished.
    pub color: HashMap<N, Color>,
    /// The preorder number of each node in the DFS search.
    pub pre: HashMap<N, usize>,
    /// The map of lowest preorder number each node is reachable in DFS search.
    pub low: HashMap<N, usize>,
    /// The map of neighbors of each node.
    pub neighbors: HashMap<N, Box<dyn Iterator<Item = N> + 'a>>,
    /// The stack of edges to be processed, it simulates a DFS search.
    pub edges_stack: Vec<(N, N)>,
}

/// Each call to `next` should return a graph's cut edge (bridge) if it exists,
/// otherwise returns `None`.
impl<'a, N> CutEdgesSearch<'a, N>
where
    N: Hash + Eq + Copy,
{
    pub fn new<G>(graph: G) -> Self
    where
        G: IntoNodeIdentifiers<NodeId = N>,
    {
        let edges_stack = if let Some(start) = graph.node_identifiers().next() {
            // Initial dummy edge
            let mut edges_stack = Vec::new();
            edges_stack.push((start, start));
            edges_stack
        } else {
            Vec::new()
        };

        CutEdgesSearch {
            color: HashMap::new(),
            pre: HashMap::new(),
            low: HashMap::new(),
            edges_stack,
            neighbors: HashMap::new(),
        }
    }

    pub fn next<G>(&mut self, graph: G) -> Option<(N, N)>
    where
        G: 'a + IntoNeighbors<NodeId = N>,
    {
        while !self.edges_stack.is_empty() {
            let (parent, a) = *self.edges_stack.last().unwrap();

            if self.color.get(&a) == None {
                let cnt = self.color.len();
                self.color.insert(a, Color::Gray);
                self.pre.insert(a, cnt);
                self.low.insert(a, cnt);
                self.neighbors.insert(a, Box::new(graph.neighbors(a)));
            }

            if self.color.get(&a) == Some(&Color::Gray) {
                if let Some(b) = (*self.neighbors.get_mut(&a).unwrap()).next() {
                    if self.color.get(&b) == None {
                        self.edges_stack.push((a, b));
                    } else if b != parent {
                        let low_a = *self.low.get(&a).unwrap();
                        let pre_b = *self.pre.get(&b).unwrap();
                        if low_a > pre_b {
                            self.low.insert(a, pre_b);
                        }
                    }
                } else {
                    self.color.insert(a, Color::Black);
                }
            } else {
                self.edges_stack.pop();
                // Check if its initial dummy Edge
                if parent == a {
                    return None;
                }
                let low_parent = *self.low.get(&parent).unwrap();
                let low_a = *self.low.get(&a).unwrap();
                let pre_a = *self.pre.get(&a).unwrap();
                if low_a < low_parent {
                    self.low.insert(parent, low_a);
                }
                if low_a == pre_a {
                    return Some((parent, a));
                }
            }
        }

        None
    }
}
