use std::collections::VecDeque;
use std::ptr::slice_from_raw_parts;
use alloc::vec::Vec;
use crate::Direction;
use crate::visit::{EdgeRef, GraphBase, IntoEdgeReferences, IntoNeighborsDirected, IntoNodeIdentifiers, NodeCount, NodeIndexable};

/// [grouped_toposort] result.
/// 
/// May be reused with [grouped_toposort_raw].
pub struct GroupedToposort<NodeId>{
    sorted: Vec<NodeId>, 
    /// `(start, len)` as [sorted] index range, that forms each group/layer. 
    group_spans: Vec<(usize, usize)>
}

impl<NodeId> Default for GroupedToposort<NodeId>{
    #[inline]
    fn default() -> Self {
        Self{
            sorted: Vec::new(),
            group_spans: Vec::new(),
        }
    }
}

impl<NodeId> GroupedToposort<NodeId>{
    #[inline]
    pub fn with_capacity(nodes_count: usize, groups_count: usize) -> Self {
        Self{
            sorted: Vec::with_capacity(nodes_count),
            group_spans: Vec::with_capacity(groups_count),
        }
    }
    
    #[inline]
    pub fn flatten(&self) -> &[NodeId] {
        &self.sorted
    }
    
    #[inline]
    pub fn groups(&self) -> impl Iterator<Item = &[NodeId]> {
        self.group_spans.iter()
            .map(|&(start, len)| unsafe{ 
                &*slice_from_raw_parts(self.sorted.as_ptr().add(start), len)
            })
    }
}

/// More low-level version of [grouped_toposort].
/// 
/// Useful if you already have necessary data. Also, completely 
/// eliminate allocations, if `out` and `roots` are big enough.
/// 
/// # Return
/// 
/// Returns `Err` - if cycle detected.
/// 
/// # Arguments
/// 
/// * `indegree` indices correspond graph's node indices. Will be filled with 0's
///  if `Ok` returned.
/// * `roots` will be used internally as queue for nodes in the next level. 
///   Must be actual roots - otherwise you'll get `Err`.
/// * `out` passed [GroupedToposort] will be filled with result, if `Ok`. May be
/// passed in non-empty state.
/// 
/// # Example
/// ```rust
///    # use petgraph::algo::grouped_toposort;
///    # use petgraph::graph::DiGraph;
///    # use std::println;
///    # use petgraph::algo::grouped_toposort::*;
///    # use std::collections::VecDeque;
///
///    let mut graph = DiGraph::<&str, ()>::new();
///
///    // ...
///    
///    let mut indegrees = Vec::new();
///    let mut roots = VecDeque::new();
///    let mut out = GroupedToposort::default();
///
///    // Update indegrees and roots, by reusing already allocated containers.
///    graph_indegrees(&graph, &mut indegrees);
///    roots.clear();
///    roots.extend(graph_roots(&graph, &indegrees));
///
///    // Do toposort. 
///    let res = grouped_toposort_raw(&graph, &mut roots, &mut indegrees, &mut out);
///    // `roots` and  `indegrees` are now "dirty", but can be reused later.
///    assert!(res.is_ok());
/// ```
pub fn grouped_toposort_raw<G>(
    graph: G,
    roots: &mut VecDeque<<G as GraphBase>::NodeId>,
    indegrees: &mut [usize],
    out: &mut GroupedToposort<<G as GraphBase>::NodeId>,
) -> Result<(), ()>
where 
    G: IntoEdgeReferences
     + IntoNeighborsDirected
     + IntoNodeIdentifiers
     + NodeIndexable
     + NodeCount,
{
    let node_count = graph.node_count();
    assert_eq!(indegrees.len(), node_count, "Wrong indegree len.");
    
    let queue = roots;
    
    out.sorted.clear();
    out.sorted.reserve(node_count);
    out.group_spans.clear();
    
    let result = &mut out.sorted;
    let levels = &mut out.group_spans;
    
    // On each iteration process only those vertices, that are in queue now - 
    // that's our "level". With that, we achieve grouping by levels.
    while !queue.is_empty() {
        let level_size = queue.len();       // Fixate current level size
        levels.push((result.len(), level_size));        
        for _ in 0..level_size {
            let node = queue.pop_front().unwrap();
            result.push(node);

            for succ in graph.neighbors_directed(node, Direction::Outgoing) {
                let idx = graph.to_index(succ);
                // TODO: this can be get_unchecked - for performance.
                indegrees[idx] -= 1;
                if indegrees[idx] == 0 {
                    queue.push_back(succ);
                }
            }
        }
    }
    
    if result.len() == node_count {
        Ok(())
    } else {
        Err(()) // cycle detected
    }        
}

/// Calculate graph indegrees to `out`.
///
/// `out` will be cleared and resized as needed.
/// 
/// O(E), where E - total edge count.
pub fn graph_indegrees<G>(graph: G, out: &mut Vec<usize>)
where
    G: IntoEdgeReferences + NodeCount + NodeIndexable
{
    out.clear();
    out.resize(graph.node_count(), 0);
    
    // Count incoming edges (one pass over edges) — uses IntoEdgeReferences.
    for e in graph.edge_references() {
        let tgt = e.target();
        out[graph.to_index(tgt)] += 1;
    }
}

/// O(N), where N - total nodes count
#[inline]
pub fn graph_roots<'a, G>(graph: G, indegrees: &'a [usize])
    -> impl Iterator<Item = <G as GraphBase>::NodeId> + 'a
where
    G: IntoNodeIdentifiers + NodeIndexable + 'a
{
    graph
        .node_identifiers()
        .filter(move |&n| indegrees[graph.to_index(n)] == 0)
}

/// Modified Khan's toposort algorithm, that remembers "layer"s positions.
/// 
/// Elements in returned groups have the same "distance" to their roots. 
/// This means they can be processed in parallel if graph represents dependency graph.
///
/// # Return
/// 
/// Returns `Err` - if cycle detected. 
pub fn grouped_toposort<G>(graph: G)
    -> Result<GroupedToposort<<G as GraphBase>::NodeId>, ()>
where 
    G: IntoEdgeReferences
     + IntoNeighborsDirected
     + IntoNodeIdentifiers
     + NodeIndexable
     + NodeCount,
{
    
    let mut indegrees= Vec::new(); 
    graph_indegrees(graph, &mut indegrees);
    let mut roots: VecDeque<_> = graph_roots(graph, &indegrees).collect();
    let mut out = GroupedToposort::default();
    grouped_toposort_raw(graph, &mut roots, &mut indegrees, &mut out)
        .map(|_| out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::DiGraph;
    use std::println;

    #[test]
    fn it_works() {
        let mut graph = DiGraph::<&str, ()>::new();
    
        let a = graph.add_node("A");
        let b = graph.add_node("B");
        let c = graph.add_node("C");
        let d = graph.add_node("D");
        let e = graph.add_node("E");
    
        graph.add_edge(a, b, ());
        graph.add_edge(a, c, ());
        graph.add_edge(b, d, ());
        /*graph.add_edge(c, d, ());
        graph.add_edge(d, e, ());*/
        graph.add_edge(c, e, ());
        graph.add_edge(b, e, ());
        
        let res = grouped_toposort(&graph); 
        match res {
            Ok(result) => {
                let names: Vec<_> = result.sorted.iter().map(|&n| graph[n]).collect();
                println!("Total order: {:?}", names);
                println!("Topological levels:");
                for group in result.groups() {
                    let names: Vec<_> = group.iter().map(|&n| graph[n]).collect();
                    println!("Level {:?}", names);
                }
            }
            Err(_) => println!("Cycle detected!"),
        }
    }
}