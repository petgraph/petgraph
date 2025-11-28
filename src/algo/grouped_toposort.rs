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
/// * `indegrees` indices correspond graph's node indices. Will be filled with 0's
///  if `Ok` returned.
/// * `roots` - If `Some` - will be used as roots source. Otherwise -
///   roots will be computed from `indegrees` at O(N).
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
///    let mut indegrees = Vec::new();
///    let mut out = GroupedToposort::default();
///
///    for _ in 0..3 {
///       // ...
///     
///       // Update indegrees, by reusing already allocated containers.
///       graph_indegrees(&graph, &mut indegrees);
///     
///       // Do toposort.
///       // We always use the same pre-allocated containers.
///       // Because we reuse `GroupedToposort` - algorithm do 0 allocations inside.
///       let res = grouped_toposort_raw(&graph, None, &mut indegrees, &mut out);
///       // `indegrees` are now filled with 0's, but container can be reused later.
///       assert!(res.is_ok());
///    }
/// ```
pub fn grouped_toposort_raw<G>(
    graph: G,
    roots: Option< &[<G as GraphBase>::NodeId] >,
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
    
    out.sorted.clear();
    out.sorted.reserve(node_count);
    out.group_spans.clear();
    
    let result = &mut out.sorted;
    let levels = &mut out.group_spans;
    
    // Fill initial result with roots.
    if let Some(roots) = roots {
        result.extend_from_slice(roots);    
    } else {
        // Get roots from indegrees
        result.extend(
            indegrees
                .iter()
                .enumerate()
                .filter_map(|(i, &d)|{
                    if d == 0 {
                        Some(graph.from_index(i))
                    } else {
                        None
                    }
                })
        );
    }
    
    // On each iteration process only those vertices, that are in queue now - 
    // that's our "level". With that, we achieve grouping by levels.
    //
    // We use `result` itself as a queue.
    let mut level_start = 0;
    while level_start != result.len() {
        let level_end = result.len();
        let range = level_start..level_end;
        
        levels.push((level_start, level_end-level_start));
        
        level_start = level_end;
        
        for i in range {
            // TODO: use get_unchecked everywhere - for performance.
            let node = result[i];
            for succ in graph.neighbors_directed(node, Direction::Outgoing) {
                let idx = graph.to_index(succ);
                indegrees[idx] -= 1;
                if indegrees[idx] == 0 {
                    result.push(succ);
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
    let mut indegrees = Vec::new(); 
    graph_indegrees(graph, &mut indegrees);
    let mut out = GroupedToposort::default();
    grouped_toposort_raw(graph, None, &mut indegrees, &mut out)
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