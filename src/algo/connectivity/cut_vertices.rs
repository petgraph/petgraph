use std::{collections::HashMap, hash::Hash};

use super::Color;
use crate::{algo::IntoNeighbors, visit::IntoNodeIdentifiers};

pub struct CutVerticesSearch<'a, N> {
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
    /// The map of subcomponents count of each node. Used to identify if root
    /// is a cut vertex and to avoid returning duplicate cut vertices.
    subcomponents_count: HashMap<N, usize>,
    /// The root of the Dfs Tree search. Used to identify if root is a cut vertex.
    root: Option<N>,
}

/// Each call to `next` should return a graph's cut vertex (articulation point)
/// if it exists, otherwise returns `None`.
impl<'a, N> CutVerticesSearch<'a, N>
where
    N: Hash + Eq + Copy,
{
    pub fn new<G>(graph: G) -> Self
    where
        G: IntoNodeIdentifiers<NodeId = N>,
    {
        let root = graph.node_identifiers().next();

        let edges_stack = if let Some(start) = root {
            // Initial dummy edge
            let mut edges_stack = Vec::new();
            edges_stack.push((start, start));
            edges_stack
        } else {
            Vec::new()
        };

        CutVerticesSearch {
            color: HashMap::new(),
            pre: HashMap::new(),
            low: HashMap::new(),
            edges_stack,
            neighbors: HashMap::new(),
            root,
            subcomponents_count: HashMap::new(),
        }
    }

    pub fn next<G>(&mut self, graph: G) -> Option<N>
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
                // Check if its initial dummy edge
                if parent == a {
                    return None;
                }
                let low_parent = *self.low.get(&parent).unwrap();
                let low_a = *self.low.get(&a).unwrap();
                let pre_parent = *self.pre.get(&parent).unwrap();
                if low_a < low_parent {
                    self.low.insert(parent, low_a);
                }
                if low_a >= pre_parent {
                    let subcomponents_count = self.subcomponents_count.entry(parent).or_insert(0);
                    *subcomponents_count += 1;

                    let is_root_articulation_point =
                        self.root == Some(parent) && *subcomponents_count == 2;
                    let is_standard_articulation_point =
                        self.root != Some(parent) && *subcomponents_count == 1;

                    if is_root_articulation_point || is_standard_articulation_point {
                        return Some(parent);
                    }
                }
            }
        }

        None
    }
}
