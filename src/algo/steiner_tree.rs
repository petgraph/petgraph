use crate::algo::floyd_warshall::floyd_warshall_path;
use crate::algo::{dijkstra, min_spanning_tree, BoundedMeasure, Measure};
use crate::data::FromElements;
use crate::dot::Dot;
use crate::graph::{NodeIndex, UnGraph};
use crate::visit::{
    Data, EdgeRef, GraphBase, GraphProp, IntoEdgeReferences, IntoEdges, NodeCompactIndexable,
    NodeIndexable, Visitable,
};
use crate::visit::{IntoNodeReferences, NodeRef};
use crate::{Graph, Undirected};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Add;

pub fn compute_shortest_path_length<G>(graph: G, source: G::NodeId, target: G::NodeId) -> G::EdgeWeight
where
    G: Visitable + IntoEdges + IntoNodeReferences,
    G::NodeId: Eq + Hash,
    G::EdgeWeight: Add + Measure + Copy,
{
    let m_target = Some(target);
    let output = dijkstra(graph, source, m_target, |e| *e.weight());
    output[&target]
}


pub fn compute_shortest_path<G>(graph: G, source: usize, target: usize) -> (Vec<usize>, G::EdgeWeight)
where
    G: Visitable + IntoEdges + IntoNodeReferences + NodeCompactIndexable + GraphProp,
    G::NodeId: Eq + Hash + Debug,
    G::EdgeWeight: Add + Measure + Copy + BoundedMeasure,
{
    let (dist, prev) = floyd_warshall_path(&graph, |e| *e.weight());
    if prev[source][target] == None {
        return (vec![], dist[source][target]);
    }
    let mut path = vec![target];
    let mut current = target;
    while source != current {
        if let Some(prev_node) = prev[source][current] {
            current = prev_node;
            path.insert(0, current);
        } else {
            return (vec![], dist[source][target]);
        }
    }
    (path, dist[source][target])
}

pub fn compute_metric_closure<G>(
    graph: G,
    terminals: Vec<G::NodeId>,
) -> Graph<G::NodeWeight, G::EdgeWeight, Undirected>
where
    G: Visitable +
    GraphBase +
    Data +
    IntoNodeReferences +
    IntoEdges +
    IntoEdgeReferences +
    NodeIndexable,
    G::NodeId: PartialOrd + Copy + Hash + Eq + Debug + PartialOrd,
    G::NodeWeight: Clone,
    G::EdgeWeight: PartialOrd + Copy + Measure,
    G::NodeRef: PartialOrd,
{
    let mut closure: Graph<G::NodeWeight, (G::EdgeWeight), Undirected> = UnGraph::new_undirected();
    let mut node_map: HashMap<G::NodeId, NodeIndex> = HashMap::new();
    for graph_node in graph.node_references() {
        if terminals.contains(&graph_node.id()) {
            let new_node = closure.add_node(graph_node.weight().clone());
            node_map.insert(graph_node.id(), new_node);
        }
    }
    for source in graph.node_references() {
        for target in graph.node_references() {
            if source < target {
                if terminals.contains(&source.id()) &&
                    terminals.contains(&target.id()) {
                    let weight = compute_shortest_path_length(graph, source.id(), target.id());
                    closure.add_edge(node_map[&source.id()], node_map[&target.id()], weight);
                }
            }
        }
    }

    closure
}


pub fn reconstruct_metric_closure<G>(graph: G, t_1: Graph<G::NodeWeight, G::EdgeWeight, Undirected>)
                                     -> Graph<G::NodeWeight, G::EdgeWeight, Undirected>
where
    G: Visitable +
    IntoEdges +
    IntoEdgeReferences +
    IntoNodeReferences +
    NodeCompactIndexable +
    GraphProp +
    Data +
    GraphBase +
    IntoEdgeReferences,
    G::NodeId: PartialOrd + Copy + Hash + Eq + Debug + PartialOrd,
    G::NodeWeight: Clone,
    G::EdgeWeight: PartialOrd + Copy + Measure + BoundedMeasure,
    G::NodeRef: PartialOrd,
{
    let mut result: Graph<G::NodeWeight, G::EdgeWeight, Undirected> = UnGraph::new_undirected();
    let mut node_map: HashMap<G::NodeId, NodeIndex> = HashMap::new();
    let mut rev_node_map: HashMap<NodeIndex, G::NodeId> = HashMap::new();
    for node in graph.node_references() {
        let new_node = result.add_node(node.weight().clone());
        node_map.insert(node.id(), new_node);
        rev_node_map.insert(new_node, node.id());
    }


    let mut edge_weight: HashMap<(usize, usize), G::EdgeWeight> = HashMap::new();

    for edge in graph.edge_references() {
        let source = graph.to_index(edge.source());
        let target = graph.to_index(edge.target());
        edge_weight.insert((source, target), edge.weight().clone());
    }


    for edge in t_1.edge_references() {
        let source = graph.to_index(rev_node_map[&edge.source()]);
        let target = graph.to_index(rev_node_map[&edge.target()]);
        let (path, path_length) = compute_shortest_path(&graph, source, target);
        for window in path.windows(2) {
            if let [u, v] = window {
                result.update_edge(node_map[&graph.from_index(*u)],
                                   node_map[&graph.from_index(*v)],
                                   edge_weight[&(*u, *v)]);
            }
        }
    }

    result
}

fn my_node_ids<G>(graph: &G, terminal_ids: Vec<G::NodeId>) -> Vec<NodeIndex>
where
    G: NodeIndexable,
{
    terminal_ids
        .iter()
        .map(|id| graph.to_index(*id))  // Convert NodeId to the corresponding index
        .map(NodeIndex::new)            // Create a NodeIndex from the index
        .collect()
}

pub fn remove_non_terminal_leaves<G>(graph: &mut Graph<G::NodeWeight, G::EdgeWeight, Undirected>, terminals: Vec<G::NodeId>)
where
    G: Visitable + GraphBase + Data + IntoNodeReferences + IntoEdges + IntoEdgeReferences + NodeCompactIndexable + GraphProp,
    G::NodeId: PartialOrd + Copy + Hash + Eq + Debug + PartialOrd,
    G::EdgeWeight: PartialOrd + Copy + BoundedMeasure,
    G::NodeRef: PartialOrd,
    G::NodeWeight: Debug,
    G::NodeWeight: Clone,
{
    // TODO implement as in: https://networkx.org/documentation/stable/_modules/networkx/algorithms/approximation/steinertree.html#steiner_tree
}

pub fn minimal_steiner_tree<G>(
    graph: G,
    terminals: Vec<G::NodeId>,
) -> Graph<G::NodeWeight, G::EdgeWeight, Undirected>
where
    G: Visitable + GraphBase + Data + IntoNodeReferences + IntoEdges + IntoEdgeReferences + NodeCompactIndexable + GraphProp,
    G::NodeId: PartialOrd + Copy + Hash + Eq + Debug + PartialOrd,
    G::EdgeWeight: PartialOrd + Copy + BoundedMeasure,
    G::NodeRef: PartialOrd,
    G::NodeWeight: Debug,
    G::NodeWeight: Clone,
{
    println!("Graph:");
    println!("{:?}", Dot::with_config(&graph, &[]));
    // Step 1: Compute the metric closure of the graph.
    let g_1 = compute_metric_closure(graph, terminals);
    println!("G_1");
    println!("{:?}", Dot::with_config(&g_1, &[]));
    // Step 2: Compute the MST of the metric closure.
    let t_1: Graph<G::NodeWeight, G::EdgeWeight, Undirected> = UnGraph::from_elements(min_spanning_tree(&g_1));
    println!("T_1");
    println!("{:?}", Dot::with_config(&t_1, &[]));
    // Step 3: Construct the subgraph of G, by placing the shortest path on each edge.
    let g_s = reconstruct_metric_closure(graph, t_1);
    println!("G_s");
    println!("{:?}", Dot::with_config(&g_s, &[]));
    // Step 4: Compute the MST of the subgraph
    let mut t_s = UnGraph::from_elements(min_spanning_tree(&g_s));

    t_s
    //g_1
}
