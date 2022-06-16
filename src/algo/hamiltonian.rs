//! Computations of [Hamiltonian Circuits](https://en.wikipedia.org/wiki/Hamiltonian_path)
//!
//! The algorithms implemented here are based on:
//! Rubin, Frank (1974), "A Search Procedure for Hamilton Paths and Circuits",
//! *Journal of the ACM*, **21** (4): 576–80,
//! doi:[10.1145/321850.321854](https://doi.org/10.1145%2F321850.321854)

use std::collections::HashSet;

use crate::graph::{node_index, EdgeIndex, EdgeReference, NodeIndex};
use crate::visit::{
    EdgeRef, GraphBase, IntoEdgeReferences, IntoEdges, IntoNeighborsDirected, IntoNodeIdentifiers,
    NodeCount, NodeIndexable,
};
use crate::{Directed, Direction, Graph};

/// \[Generic\] algorithm for computing Hamiltonian circuits (Frank Rubin).
///
/// Computes circuits that cover each node once and end where they started.
///
/// The graph should implement `IntoEdges`, `IntoNeighborsDirected`, `IntoNodeIdentifiers`,
/// `NodeCount`, and `NodeIndexable`.
///
/// The graph must not contain self-loops: no node should have an edge from itself to itself.
///
/// Computing Hamiltonian circuits is O(n!), so this function may be very slow for large graphs.
///
/// The algorithm used is designed to reduce the actual execution time for reasonably large
/// graphs, by performing polynomial-time checks to prune the selection of paths that must be searched.
/// In practice, finding a single circuit in a graph with 40 nodes or fewer can generally be handled
/// within milliseconds on early 2020's hardware. Some highly connected graphs have very large
/// numbers of circuits, so to find all circuits in highly-connected graphs of more than 10 nodes
/// can take seconds or much longer.
///
/// # Example
/// ```
/// use petgraph::Graph;
/// use petgraph::graph::node_index;
/// use petgraph::algo::hamiltonian_circuits_directed;
///
/// // Create a fully-connected digraph with 3 nodes
/// let mut g = Graph::<(), ()>::new();
/// let n0 = g.add_node(());
/// let n1 = g.add_node(());
/// let n2 = g.add_node(());
/// g.extend_with_edges(&[
///     (0, 1), (0, 2),
///     (1, 0), (1, 2),
///     (2, 0), (2, 1),
/// ]);
///
/// // Ask for all the Hamiltonian circuits and collect the results
/// let circuits: Vec<_> = hamiltonian_circuits_directed(&g).collect();
///
/// // We have all the possible circuits as Vecs of NodeId
/// assert_eq!(
///     circuits,
///     vec![
///         vec![n0, n2, n1],
///         vec![n0, n1, n2],
///     ]
/// );
/// ```
///
/// Returns an iterater over the Hamiltonian Circuits in this graph.
pub fn hamiltonian_circuits_directed<G>(g: G) -> impl Iterator<Item = Vec<G::NodeId>>
where
    G: GraphBase
        + IntoEdges
        + IntoNeighborsDirected
        + IntoNodeIdentifiers
        + NodeCount
        + NodeIndexable,
{
    HamiltonianCircuits::new(g)
}

// --- HamiltonianCircuits ---

struct HamiltonianCircuits<G>
where
    G: GraphBase + IntoNeighborsDirected,
{
    partial_paths: PartialPaths<G>,
    #[cfg(test)]
    visited_count: usize,
}

impl<G> HamiltonianCircuits<G>
where
    G: GraphBase + IntoEdges + IntoNeighborsDirected + IntoNodeIdentifiers + NodeCount,
{
    fn new(g: G) -> Self {
        let num_nodes = g.node_count();
        let mut partial_paths = PartialPaths::new(g);
        match num_nodes {
            1 | 2 => {
                // Special case for graphs of 1 or 2 nodes - they have
                // no circuits by convention.
                partial_paths.skip_latest_node();
                partial_paths.skip_latest_node();
            }
            _ => (),
        };

        Self {
            partial_paths,
            #[cfg(test)]
            visited_count: 0,
        }
    }

    #[cfg(test)]
    fn partial_paths_visited(&self) -> usize {
        self.visited_count
    }
}

impl<G> Iterator for HamiltonianCircuits<G>
where
    G: GraphBase
        + IntoEdges
        + IntoNeighborsDirected
        + IntoNodeIdentifiers
        + NodeCount
        + NodeIndexable,
{
    type Item = Vec<<G as GraphBase>::NodeId>;

    fn next(&mut self) -> Option<Self::Item> {
        // S1. Select any single node as the initial path
        #[cfg(test)]
        {
            self.visited_count += 1;
        }
        let mut next_partial_path = self.partial_paths.next();

        while let Some(partial_path) = next_partial_path {
            if partial_path.covers_all_nodes(self.partial_paths.graph)
                && partial_path.last_node_has_edge_to_first(self.partial_paths.graph)
            {
                // S7. If a successor from the initial node is the origin, a Hamiltonian circuit is
                //     formed; if all Hamiltonian circuits are required, then list the circuit found,
                //     mark the partial path inadmissible, and repeat step S4.
                return Some(partial_path.into());
            }

            // S2. Test the path for admissability
            let admissable = is_admissable(self.partial_paths.graph, &partial_path);

            next_partial_path = if admissable {
                // S3. If the path so far is admissable, list the successors of the last node chosen,
                // and extend the path to the first of these. Repeat step S2.
                #[cfg(test)]
                {
                    self.visited_count += 1;
                }
                self.partial_paths.next()
            } else {
                // S4. If the path so far is inadmissible, delete the last node chosen and choose the
                //     next listed successor of the preceding node. Repeat step S2.
                //
                // S5. If all extensions from a given node have been shown inadmissible, repeat
                //     step S4.
                #[cfg(test)]
                {
                    self.visited_count += 1;
                }
                self.partial_paths.skip_latest_node()
            }
        }

        // S6. if all extensions from the initial node have been shown inadmissable, then no circuit
        //      exists.
        None
    }
}

// --- is_admissable ---

#[derive(Debug, PartialEq)]
enum EdgeRequired {
    Undecided,    // We don't know whether this edge will be needed
    Required,     // This edge will definitely be needed
    Semirequired, // This edge or its opposite partner will be needed
}

fn is_admissable<G>(graph: G, partial_path: &PartialPath<G>) -> bool
where
    G: GraphBase + IntoEdgeReferences + NodeCount + NodeIndexable,
{
    // Everything starts off undecided
    let mut classification = Classification::new(graph);

    // The edges of P are also in R
    classification.partial_path_is_required(graph, partial_path);

    let mut updated = true;

    while updated {
        updated = false;

        // R1. If a vertex has only one directed arc entering (leaving), then that arc is required.
        updated = classification.unique_edges_are_required() || updated;

        // R2. If a vertex has only two arcs incident, then both arcs are required.
        updated = classification.two_incident_edges_means_both_required() || updated;

        // A1. If a vertex has a required directed arc entering (leaving), then all incident undirected
        // arcs are assigned the direction leaving (entering) that vertex.
        updated = classification.one_direction_required_forces_other_direction_for_other_edges()
            || updated;

        // A2. If a vertex has a required undirected arc incident, and all other incident arcs are
        // leaving (entering) the vertex, then the required arc is assigned the direction entering
        // (leaving) the vertex.
        updated =
            classification.all_other_edges_in_one_direction_means_pair_becomes_single() || updated;

        // D1. If a vertex has two required arcs incident, then all undecided arcs incident may be
        // deleted.
        updated = classification.two_required_edges_means_delete_nonrequired() || updated;

        // D2. If a vertex has a required directed edge arc entering (leaving), then all undecided
        // directed arcs entering (leaving) may be deleted.
        updated = classification.required_edge_means_delete_others_in_same_direction() || updated;

        // D3. Delete any arc which forms a closed circuit with required arcs, unless it completes the
        // Hamiltonian circuit.
        updated = classification.delete_invalid_circuit_forming_edges() | updated;
    }

    let mut inadmissable = false;

    // F1. Fail if any vertex becomes isolated, that is, has no incident arc.
    // F2. Fail if any vertex has only one incident arc.
    inadmissable = inadmissable || classification.any_node_with_zero_or_one_edge();

    // F3. Fail if any vertex has no directed arc entering (leaving).
    inadmissable = inadmissable || classification.any_node_with_no_incoming_or_no_outgoing();

    // F4. Fail if any vertex has two required directed arcs entering (leaving).
    inadmissable =
        inadmissable || classification.any_node_with_two_required_edges_in_same_direction();

    // F5. Fail if any vertex has three required arcs incident.
    inadmissable = inadmissable || classification.any_node_with_three_required_edges();

    // F6. Fail if any set of required arcs forms a closed circuit, other than a Hamiltonian
    // circuit.
    inadmissable = inadmissable || classification.any_circuit_of_required_except_hamiltonian();

    !inadmissable
}

struct Classification {
    classes: Graph<(), EdgeRequired, Directed, usize>,
}

impl std::fmt::Debug for Classification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn fmt_edge_required(edge_required: &EdgeRequired) -> &'static str {
            match edge_required {
                EdgeRequired::Undecided => "_",
                EdgeRequired::Required => "R",
                EdgeRequired::Semirequired => "r",
            }
        }

        for node in self.classes.node_indices() {
            f.write_fmt(format_args!("{} ->", node.index()))?;
            for edge in self.classes.edges_directed(node, Direction::Outgoing) {
                f.write_fmt(format_args!(
                    " {}{}",
                    fmt_edge_required(edge.weight()),
                    self.classes.to_index(edge.target())
                ))?;
            }
            f.write_str("\n")?;
        }
        Ok(())
    }
}

impl Classification {
    fn new<G>(graph: G) -> Self
    where
        G: GraphBase + IntoEdgeReferences + NodeCount + NodeIndexable,
    {
        let mut classes = Graph::<_, _, _, _>::from_edges(graph.edge_references().map(|e| {
            (
                graph.to_index(e.source()),
                graph.to_index(e.target()),
                EdgeRequired::Undecided,
            )
        }));
        for _ in 0..(graph.node_count() - classes.node_count()) {
            classes.add_node(());
        }

        Self { classes }
    }

    // --- Modifying utils ---

    /// Change the required status of this edge. Do not consider other edges.
    /// Return true if the status changed.
    fn update_single_edge(&mut self, edge: EdgeIndex<usize>, required: EdgeRequired) -> bool {
        let e: &mut EdgeRequired = &mut self.classes[edge];
        if *e != required {
            *e = required;
            true
        } else {
            false
        }
    }

    /// Delete this edge. Do not consider other edges.
    /// Return true if the edge existed and was deleted.
    fn delete_single_edge(
        &mut self,
        from_node: NodeIndex<usize>,
        to_node: NodeIndex<usize>,
    ) -> bool {
        if let Some(edge) = self.classes.find_edge(from_node, to_node) {
            self.classes.remove_edge(edge);
            true
        } else {
            false
        }
    }

    /// Change this edge to be required. If it was not already required, and it has an opposite,
    /// delete the opposite.
    /// Return true if the status changed.
    fn set_required(&mut self, source: NodeIndex<usize>, target: NodeIndex<usize>) -> bool {
        if let Some(edge) = self.classes.find_edge(source, target) {
            let updated = self.update_single_edge(edge, EdgeRequired::Required);
            if updated {
                // If source->target is updated to be required, we can delete its opposite
                self.delete_single_edge(target, source);
            }
            updated
        } else {
            false
        }
    }

    /// Mark both edges between these nodes as semirequired, or if only one of these nodes
    /// exists, mark it as required.
    fn set_semirequired(&mut self, source: NodeIndex<usize>, target: NodeIndex<usize>) -> bool {
        let edge1 = self.classes.find_edge(source, target);
        let edge2 = self.classes.find_edge(target, source);
        match (edge1, edge2) {
            (Some(edge1), Some(edge2)) => {
                // If both directions exist, they become semirequired
                let updated = self.update_single_edge(edge1, EdgeRequired::Semirequired);
                self.update_single_edge(edge2, EdgeRequired::Semirequired) || updated
            }
            // If only one direction exists, it becomes required
            (Some(edge1), None) => self.update_single_edge(edge1, EdgeRequired::Required),
            (None, Some(edge2)) => self.update_single_edge(edge2, EdgeRequired::Required),
            (_, _) => false,
        }
    }

    fn delete(&mut self, source: NodeIndex<usize>, target: NodeIndex<usize>) -> bool {
        if let Some(edge) = self.classes.find_edge(source, target) {
            self.classes.remove_edge(edge);
            if let Some(opposite) = self.classes.find_edge(target, source) {
                // If we deleted an edge and its opposite was semirequired, it is now required.
                let e = &mut self.classes[opposite];
                if *e == EdgeRequired::Semirequired {
                    *e = EdgeRequired::Required;
                }
            }
            true
        } else {
            false
        }
    }

    fn set_all_required(
        &mut self,
        newly_required: Vec<(NodeIndex<usize>, NodeIndex<usize>)>,
    ) -> bool {
        let mut updated = false;
        for (source, target) in newly_required {
            updated = self.set_required(source, target) || updated;
        }
        updated
    }

    fn set_all_semirequired(
        &mut self,
        newly_semirequired: Vec<(NodeIndex<usize>, NodeIndex<usize>)>,
    ) -> bool {
        let mut updated = false;
        for (source, target) in newly_semirequired {
            updated = self.set_semirequired(source, target) || updated;
        }
        updated
    }

    fn delete_all(&mut self, to_delete: Vec<(NodeIndex<usize>, NodeIndex<usize>)>) -> bool {
        let mut updated = false;
        for (source, target) in to_delete {
            updated = self.delete(source, target) || updated;
        }
        updated
    }

    // --- Querying utils ---

    fn all_edges(
        &self,
        node: NodeIndex<usize>,
    ) -> impl Iterator<Item = EdgeReference<EdgeRequired, usize>> {
        self.classes
            .edges_directed(node, Direction::Outgoing)
            .chain(self.classes.edges_directed(node, Direction::Incoming))
    }

    fn all_undecided_edges(
        &self,
        node: NodeIndex<usize>,
    ) -> impl Iterator<Item = EdgeReference<EdgeRequired, usize>> {
        self.all_edges(node)
            .filter(|e| *e.weight() == EdgeRequired::Undecided)
    }

    /// Return an iterator of all outgoing Required + Semirequired, and incoming Required only,
    /// avoiding double-counting the Semirequired ones.
    fn all_required_edges(
        &self,
        node: NodeIndex<usize>,
    ) -> impl Iterator<Item = EdgeReference<EdgeRequired, usize>> {
        self.all_edges(node).filter(move |e| {
            *e.weight() == EdgeRequired::Required
                || (e.source() == node && *e.weight() == EdgeRequired::Semirequired)
        })
    }

    fn incoming_required_edges(
        &self,
        node: NodeIndex<usize>,
    ) -> impl Iterator<Item = EdgeReference<EdgeRequired, usize>> {
        self.classes
            .edges_directed(node, Direction::Incoming)
            .filter(|e| *e.weight() == EdgeRequired::Required)
    }

    fn outgoing_required_edges(
        &self,
        node: NodeIndex<usize>,
    ) -> impl Iterator<Item = EdgeReference<EdgeRequired, usize>> {
        self.classes
            .edges_directed(node, Direction::Outgoing)
            .filter(|e| *e.weight() == EdgeRequired::Required)
    }

    fn outgoing_required_edge(
        &self,
        node: NodeIndex<usize>,
    ) -> Option<EdgeReference<EdgeRequired, usize>> {
        self.outgoing_required_edges(node).next()
    }

    // --- Deduction rules ---

    fn partial_path_is_required<G>(&mut self, graph: G, partial_path: &PartialPath<G>)
    where
        G: GraphBase + NodeIndexable,
    {
        // All edges already in the path are required
        let mut prev_node;
        let mut next_node = None;
        for &node in &partial_path.nodes {
            prev_node = next_node;
            next_node = Some(graph.to_index(node));

            if let (Some(prev_node), Some(next_node)) = (prev_node, next_node) {
                self.set_required(node_index(prev_node), node_index(next_node));
            }
        }
    }

    fn unique_edges_are_required(&mut self) -> bool {
        fn required_if_unique(
            classification: &mut Graph<(), EdgeRequired, Directed, usize>,
            node: NodeIndex<usize>,
            dir: Direction,
            newly_required: &mut Vec<(NodeIndex<usize>, NodeIndex<usize>)>,
        ) {
            let mut outgoing = classification.edges_directed(node, dir);
            let first = outgoing.next();
            if let Some(first) = first {
                if let None = outgoing.next() {
                    // If the first outgoing edge is the only outgoing edge, this is required
                    newly_required.push((first.source(), first.target()));
                }
            }
        }

        let mut newly_required = Vec::<(NodeIndex<usize>, NodeIndex<usize>)>::new();

        for node in self.classes.node_indices() {
            required_if_unique(
                &mut self.classes,
                node,
                Direction::Outgoing,
                &mut newly_required,
            );
            required_if_unique(
                &mut self.classes,
                node,
                Direction::Incoming,
                &mut newly_required,
            );
        }

        self.set_all_required(newly_required)
    }

    fn two_incident_edges_means_both_required(&mut self) -> bool {
        let mut newly_semirequired = Vec::<(NodeIndex<usize>, NodeIndex<usize>)>::new();

        for node in self.classes.node_indices() {
            let mut other_nodes = HashSet::<NodeIndex<usize>>::new();
            for edge in self.classes.edges_directed(node, Direction::Outgoing) {
                other_nodes.insert(edge.target());
            }
            for edge in self.classes.edges_directed(node, Direction::Incoming) {
                other_nodes.insert(edge.source());
            }
            if other_nodes.len() == 2 {
                // Two nodes are adjacent to this one. In the language of the paper,
                // pairs of opposite edges are called a single undirected edge, so
                // this is equivalent to "two incident edges".
                for other in other_nodes {
                    newly_semirequired.push((node, other));
                }
            }
        }

        // Set edges in both directions as semirequired (or, if only one direction exists, this
        // will set them as required).
        self.set_all_semirequired(newly_semirequired)
    }

    fn make_any_required_edge_unique(
        &self,
        node: NodeIndex<usize>,
        direction: Direction,
    ) -> Vec<(NodeIndex<usize>, NodeIndex<usize>)> {
        let mut to_delete = Vec::<(NodeIndex<usize>, NodeIndex<usize>)>::new();
        for edge in self.classes.edges_directed(node, direction) {
            if *edge.weight() == EdgeRequired::Required {
                for other_edge in self
                    .classes
                    .edges_directed(node, direction)
                    .filter(|&e| e != edge)
                {
                    to_delete.push((other_edge.source(), other_edge.target()));
                }
                break; // We found the required edge, no need to look at others
            }
        }
        to_delete
    }

    fn one_direction_required_forces_other_direction_for_other_edges(&mut self) -> bool {
        let mut to_delete = Vec::<(NodeIndex<usize>, NodeIndex<usize>)>::new();
        for node in self.classes.node_indices() {
            // If one incoming is required, delete other incoming
            to_delete.append(&mut self.make_any_required_edge_unique(node, Direction::Incoming));
            // If one outgoing is required, delete other outgoing
            to_delete.append(&mut self.make_any_required_edge_unique(node, Direction::Outgoing));
        }
        self.delete_all(to_delete)
    }

    fn unique_semirequired_becomes_required(&mut self, direction: Direction) -> bool {
        // We delete as we go, to avoid accidentally deleting both directions of a semirequired
        // edge.
        let mut updated = false;
        for node in self.classes.node_indices() {
            // If there is only one edge in this direction
            if self.classes.edges_directed(node, direction).count() == 1 {
                let mut to_delete = Vec::<(NodeIndex<usize>, NodeIndex<usize>)>::new();
                // Find that unique edge
                for edge in self.classes.edges_directed(node, direction) {
                    // If it's semirequired, delete its opposite
                    if *edge.weight() == EdgeRequired::Semirequired {
                        to_delete.push((edge.target(), edge.source()));
                    }
                }
                updated = self.delete_all(to_delete) || updated;
            }
        }
        updated
    }

    fn all_other_edges_in_one_direction_means_pair_becomes_single(&mut self) -> bool {
        // If all other edges are incoming, set semirequired to outgoing
        let mut updated = self.unique_semirequired_becomes_required(Direction::Incoming);

        // We apply this direction before doing the opposite, to avoid deleting both directions
        // of a pair

        // If all other edges are outgoing, set semirequired to incoming
        updated = self.unique_semirequired_becomes_required(Direction::Outgoing) || updated;

        updated
    }

    fn two_required_edges_means_delete_nonrequired(&mut self) -> bool {
        let mut to_delete = Vec::<(NodeIndex<usize>, NodeIndex<usize>)>::new();
        for node in self.classes.node_indices() {
            let num_required = self.all_required_edges(node).take(2).count();

            if num_required == 2 {
                // Delete all undecided
                to_delete.append(
                    &mut self
                        .all_undecided_edges(node)
                        .map(|e| (e.source(), e.target()))
                        .collect::<Vec<_>>(),
                );
            }
        }
        self.delete_all(to_delete)
    }

    fn edges_to_delete_because_one_is_required(
        &self,
        node: NodeIndex<usize>,
        direction: Direction,
    ) -> Vec<(NodeIndex<usize>, NodeIndex<usize>)> {
        for edge in self.classes.edges_directed(node, direction) {
            // If this edge is required
            if *edge.weight() == EdgeRequired::Required {
                // Delete all other edges in the same direction
                return self
                    .classes
                    .edges_directed(node, direction)
                    .filter(|&e| e != edge)
                    .map(|e| (e.source(), e.target()))
                    .collect();
            }
        }
        vec![]
    }

    fn required_edge_means_delete_others_in_same_direction(&mut self) -> bool {
        let mut to_delete = Vec::<(NodeIndex<usize>, NodeIndex<usize>)>::new();
        for node in self.classes.node_indices() {
            to_delete.append(
                &mut self.edges_to_delete_because_one_is_required(node, Direction::Outgoing),
            );
            to_delete.append(
                &mut self.edges_to_delete_because_one_is_required(node, Direction::Incoming),
            );
        }
        self.delete_all(to_delete)
    }

    fn forms_a_short_circuit_with_required(
        &self,
        edge: EdgeReference<EdgeRequired, usize>,
    ) -> bool {
        let mut visited = Vec::<NodeIndex<usize>>::new();
        visited.push(edge.target());
        // Follow a path of required edges.
        let mut next_edge = self.outgoing_required_edge(edge.target());
        while let Some(ne) = next_edge {
            if visited.contains(&ne.target()) {
                break;
            }
            visited.push(ne.target());
            if ne.target() == edge.source() {
                // If we got back to the beginning, we made a circuit
                if visited.len() == self.classes.node_count() {
                    // We made a Hamiltonian circuit!
                    return false;
                } else {
                    // We made a smaller circuit
                    return true;
                }
            }
            next_edge = self.outgoing_required_edge(ne.target());
        }
        // If we ran out of required edges, we did not make a circuit
        false
    }

    fn delete_invalid_circuit_forming_edges(&mut self) -> bool {
        let mut to_delete = Vec::<(NodeIndex<usize>, NodeIndex<usize>)>::new();
        for edge in self
            .classes
            .edge_references()
            .filter(|e| *e.weight() == EdgeRequired::Undecided)
        {
            if self.forms_a_short_circuit_with_required(edge) {
                to_delete.push((edge.source(), edge.target()));
            }
        }
        self.delete_all(to_delete)
    }

    // --- Failure rules ---

    fn any_node_with_zero_or_one_edge(&self) -> bool {
        self.classes.node_indices().any(|node| {
            let mut edges = self.all_edges(node);
            if edges.next() == None {
                true // Zero edges
            } else if edges.next() == None {
                true // One edge
            } else {
                false // More
            }
        })
    }

    fn any_node_with_no_incoming_or_no_outgoing(&self) -> bool {
        self.classes.node_indices().any(|node| {
            self.classes
                .edges_directed(node, Direction::Outgoing)
                .next()
                == None
                || self
                    .classes
                    .edges_directed(node, Direction::Incoming)
                    .next()
                    == None
        })
    }

    fn any_node_with_two_required_edges_in_same_direction(&self) -> bool {
        self.classes.node_indices().any(|node| {
            self.outgoing_required_edges(node).take(2).count() > 1
                || self.incoming_required_edges(node).take(2).count() > 1
        })
    }

    fn any_node_with_three_required_edges(&self) -> bool {
        self.classes
            .node_indices()
            .any(|node| self.all_required_edges(node).take(3).count() == 3)
    }

    fn any_circuit_of_required_except_hamiltonian(&self) -> bool {
        let mut remaining_nodes: Vec<_> = self.classes.node_indices().collect();
        // Pick a node
        while let Some(initial_node) = remaining_nodes.pop() {
            let mut this_path_nodes = vec![initial_node];
            // Follow a path of required edges from it
            let mut later_node = self
                .outgoing_required_edge(initial_node)
                .map(|e| e.target());
            while let Some(ln) = later_node {
                // Check for a circuit
                if this_path_nodes.contains(&ln) {
                    // We found a circuit, because we hit a node we've already seen
                    if this_path_nodes.len() == self.classes.node_count() {
                        // The circuit contains all the nodes, so it's Hamiltonian.
                        return false;
                    } else {
                        // A circuit of only some nodes: this is what we were checking for
                        return true;
                    }
                } else {
                    // Not a circuit: keep going
                    this_path_nodes.push(ln);
                }
                // Follow the next required edge
                later_node = self.outgoing_required_edge(ln).map(|e| e.target());
            }
            // We ran out of required edges: this was not a circuit - remove the nodes
            // we have visited from the list so we can pick another.
            remaining_nodes.retain(|n| !this_path_nodes.contains(n));
        }
        // We didn't find any circuits at all
        false
    }
}

// --- PartialPaths ---

struct PartialPaths<G>
where
    G: GraphBase + IntoNeighborsDirected,
{
    graph: G,
    stack: PathsStack<G>,
    prev_last_node_index: Option<<G as GraphBase>::NodeId>,
}

impl<G> PartialPaths<G>
where
    G: GraphBase + IntoEdges + IntoNeighborsDirected + IntoNodeIdentifiers + NodeCount,
{
    fn new(graph: G) -> Self {
        let stack = PathsStack::new(graph);
        Self {
            graph,
            stack,
            prev_last_node_index: None,
        }
    }

    fn next(&mut self) -> Option<PartialPath<G>> {
        if self.stack.is_empty() {
            None
        } else {
            let ret = self.partial_path();
            self.prev_last_node_index = ret.nodes.last().map(|i| *i);
            self.move_next();
            Some(ret)
        }
    }

    fn move_next(&mut self) {
        let path_so_far = self.stack.node_indexes();
        let last_node = self.stack.last_mut();
        if let Some(last_node) = last_node {
            while let Some(next_neighbor) = last_node.neighbors.next() {
                if !path_so_far.contains(&next_neighbor) {
                    self.stack.push(StackItem::new(self.graph, next_neighbor));
                    return;
                }
            }
            self.stack.pop();
            self.move_next();
        }
    }

    fn skip_latest_node(&mut self) -> Option<PartialPath<G>> {
        let mut popped = false;
        while self.stack.contains(self.prev_last_node_index) {
            self.stack.pop();
            popped = true;
        }
        if popped {
            self.move_next();
        }
        self.next()
    }

    fn partial_path(&self) -> PartialPath<G> {
        self.stack
            .iter()
            .map(|path_node| path_node.node_index)
            .collect::<Vec<_>>()
            .into()
    }
}

// --- PathsStack ---

struct PathsStack<G>(Vec<StackItem<G>>)
where
    G: GraphBase + IntoNeighborsDirected;

impl<G> PathsStack<G>
where
    G: GraphBase + IntoNeighborsDirected + IntoNodeIdentifiers,
{
    fn new(graph: G) -> Self {
        let node0 = graph.node_identifiers().next();

        Self(if let Some(node0) = node0 {
            vec![StackItem::new(graph, node0)]
        } else {
            vec![]
        })
    }

    fn iter(&self) -> std::slice::Iter<StackItem<G>> {
        self.0.iter()
    }

    fn pop(&mut self) -> Option<StackItem<G>> {
        self.0.pop()
    }

    fn push(&mut self, value: StackItem<G>) {
        self.0.push(value)
    }

    fn last_mut(&mut self) -> Option<&mut StackItem<G>> {
        self.0.last_mut()
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn node_indexes(&self) -> Vec<<G as GraphBase>::NodeId> {
        self.0.iter().map(|item| item.node_index).collect()
    }

    fn contains(&self, node_index: Option<<G as GraphBase>::NodeId>) -> bool {
        if let Some(node_index) = node_index {
            self.0.iter().any(|item| item.node_index == node_index)
        } else {
            false
        }
    }
}

// --- StackItem ---

struct StackItem<G>
where
    G: GraphBase + IntoNeighborsDirected,
{
    node_index: <G as GraphBase>::NodeId,
    neighbors: <G as IntoNeighborsDirected>::NeighborsDirected,
}

impl<G> StackItem<G>
where
    G: GraphBase + IntoNeighborsDirected,
{
    fn new(graph: G, node_index: <G as GraphBase>::NodeId) -> Self {
        Self {
            node_index,
            neighbors: graph.neighbors_directed(node_index, Direction::Outgoing),
        }
    }
}

// --- PartialPath ---

struct PartialPath<G>
where
    G: GraphBase,
{
    nodes: Vec<<G as GraphBase>::NodeId>,
}

impl<G> PartialPath<G>
where
    G: GraphBase + IntoEdges + NodeCount,
{
    fn new(nodes: Vec<<G as GraphBase>::NodeId>) -> Self {
        Self { nodes }
    }

    fn covers_all_nodes(&self, g: G) -> bool {
        self.nodes.len() == g.node_count()
    }

    fn last_node_has_edge_to_first(&self, g: G) -> bool {
        let f = self.nodes.first();
        let l = self.nodes.last();
        if let (Some(f), Some(l)) = (f, l) {
            return if f == l {
                true
            } else {
                g.edges(*l).any(|edge| edge.target() == *f)
            };
        }
        false
    }
}

impl<G> From<Vec<<G as GraphBase>::NodeId>> for PartialPath<G>
where
    G: GraphBase + IntoEdges + NodeCount,
{
    fn from(v: Vec<<G as GraphBase>::NodeId>) -> Self {
        Self::new(v)
    }
}

impl<G> From<PartialPath<G>> for Vec<<G as GraphBase>::NodeId>
where
    G: GraphBase,
{
    fn from(partial_path: PartialPath<G>) -> Self {
        partial_path.nodes
    }
}

// --- tests ---

#[cfg(test)]
mod test {
    use crate::graph::node_index;
    use crate::visit::{GraphBase, NodeIndexable};
    use crate::Graph;

    use super::{
        hamiltonian_circuits_directed, Classification, HamiltonianCircuits, PartialPath,
        PartialPaths,
    };

    fn fully_connected_graph(nodes: usize) -> Graph<(), ()> {
        let mut edges = Vec::new();
        for a in 0..nodes {
            for b in (a + 1)..nodes {
                edges.push((node_index(a), node_index(b)));
                edges.push((node_index(b), node_index(a)));
            }
        }
        Graph::from_edges(edges)
    }

    fn complete_bipartite_graph(nodes_each_side: usize) -> Graph<(), ()> {
        let mut edges = Vec::new();
        for a in 0..nodes_each_side {
            for b in nodes_each_side..(nodes_each_side * 2) {
                edges.push((node_index(a), node_index(b)));
                edges.push((node_index(b), node_index(a)));
            }
        }
        Graph::from_edges(edges)
    }

    // --- Tests for PartialPaths ---

    fn as_vec(path: Option<PartialPath<&Graph<(), ()>>>) -> Vec<usize> {
        if let Some(path) = path {
            let v: Vec<<Graph<(), ()> as GraphBase>::NodeId> = path.into();
            v.iter().map(|&n| n.index()).collect()
        } else {
            vec![]
        }
    }

    #[test]
    fn graph_with_no_nodes_has_no_partials() {
        // Create a 1 node graph
        let g = Graph::<(), ()>::new();

        let mut partial_paths = PartialPaths::new(&g);

        // It has no partial paths
        assert!(partial_paths.next().is_none());
    }

    #[test]
    fn graph_with_one_node_has_one_partial() {
        // Create a 1 node graph
        let mut g = Graph::new();
        g.add_node(());

        let mut partial_paths = PartialPaths::new(&g);

        // It has one partial path
        assert_eq!(vec![0], as_vec(partial_paths.next()));
        assert!(partial_paths.next().is_none());
    }

    #[test]
    fn graph_with_two_nodes_has_two_partials() {
        // Create a 2 node graph
        let g = fully_connected_graph(2);

        let mut partial_paths = PartialPaths::new(&g);

        // It has 2 partial paths
        assert_eq!(vec![0], as_vec(partial_paths.next()));
        assert_eq!(vec![0, 1], as_vec(partial_paths.next()));
        assert!(partial_paths.next().is_none());
    }

    #[test]
    fn graph_with_3_nodes_has_5_partials() {
        // Create a 3 node graph
        let g = fully_connected_graph(3);

        let mut partial_paths = PartialPaths::new(&g);

        // It has 5 partial paths
        assert_eq!(vec![0], as_vec(partial_paths.next()));
        assert_eq!(vec![0, 2], as_vec(partial_paths.next()));
        assert_eq!(vec![0, 2, 1], as_vec(partial_paths.next()));
        assert_eq!(vec![0, 1], as_vec(partial_paths.next()));
        assert_eq!(vec![0, 1, 2], as_vec(partial_paths.next()));
        assert!(partial_paths.next().is_none());
    }

    #[test]
    fn skip_latest_node_prunes_path() {
        // Create a 3 node graph
        let g = Graph::<(), ()>::from_edges(&[(0, 1), (0, 2), (1, 0), (1, 2), (2, 0), (2, 1)]);

        let mut partial_paths = PartialPaths::new(&g);

        // Grab the first 2
        assert_eq!(vec![0], as_vec(partial_paths.next()));
        assert_eq!(vec![0, 2], as_vec(partial_paths.next()));

        // Now ask to prune this node
        assert_eq!(vec![0, 1], as_vec(partial_paths.skip_latest_node()));

        // And we continue from there
        assert_eq!(vec![0, 1, 2], as_vec(partial_paths.next()));
        assert!(partial_paths.next().is_none());
    }

    #[test]
    fn skip_latest_node_eventually_returns_none() {
        // Create a graph
        let g = fully_connected_graph(10);

        let mut partial_paths = PartialPaths::new(&g);

        // Grab the first few
        assert_eq!(vec![0], as_vec(partial_paths.next()));
        assert_eq!(vec![0, 9], as_vec(partial_paths.next()));
        assert_eq!(vec![0, 9, 8], as_vec(partial_paths.next()));

        // Start skipping
        assert_eq!(vec![0, 9, 7], as_vec(partial_paths.skip_latest_node()));
        assert_eq!(vec![0, 9, 6], as_vec(partial_paths.skip_latest_node()));
        assert_eq!(vec![0, 9, 5], as_vec(partial_paths.skip_latest_node()));
        assert_eq!(vec![0, 9, 4], as_vec(partial_paths.skip_latest_node()));
        assert_eq!(vec![0, 9, 3], as_vec(partial_paths.skip_latest_node()));
        assert_eq!(vec![0, 9, 2], as_vec(partial_paths.skip_latest_node()));
        assert_eq!(vec![0, 9, 1], as_vec(partial_paths.skip_latest_node()));
        assert_eq!(vec![0, 8], as_vec(partial_paths.skip_latest_node()));
        assert_eq!(vec![0, 7], as_vec(partial_paths.skip_latest_node()));
        assert_eq!(vec![0, 6], as_vec(partial_paths.skip_latest_node()));
        assert_eq!(vec![0, 5], as_vec(partial_paths.skip_latest_node()));
        assert_eq!(vec![0, 4], as_vec(partial_paths.skip_latest_node()));
        assert_eq!(vec![0, 3], as_vec(partial_paths.skip_latest_node()));
        assert_eq!(vec![0, 2], as_vec(partial_paths.skip_latest_node()));
        assert_eq!(vec![0, 1], as_vec(partial_paths.skip_latest_node()));
        assert!(partial_paths.skip_latest_node().is_none());
    }

    #[test]
    fn can_prune_long_paths() {
        let g = Graph::<(), ()>::from_edges(&[
            (0, 1),
            (1, 2), //     --->1--->2--->3---
            (2, 3), //    /                  \
            (3, 4), //   0                    4
            (0, 5), //    \                  /
            (5, 6), //     --->5--->6--->7---
            (6, 7),
            (7, 4),
        ]);

        let mut partial_paths = PartialPaths::new(&g);

        // Start walking down one branch
        assert_eq!(vec![0], as_vec(partial_paths.next()));
        assert_eq!(vec![0, 5], as_vec(partial_paths.next()));

        // Prune
        assert_eq!(vec![0, 1], as_vec(partial_paths.skip_latest_node()));

        // And we continue from there
        assert_eq!(vec![0, 1, 2], as_vec(partial_paths.next()));
        assert_eq!(vec![0, 1, 2, 3], as_vec(partial_paths.next()));
        assert_eq!(vec![0, 1, 2, 3, 4], as_vec(partial_paths.next()));

        // And never go down the other path
        assert!(partial_paths.next().is_none());
    }

    #[test]
    fn graph_with_one_required_edge_can_be_walked() {
        // Create a 1 node graph
        let g = Graph::<(), ()>::from_edges(&[
            (0, 1),
            (0, 2),
            (0, 3),
            (1, 0), // The only edge from 1 is to 0.
            (2, 0), // Everything else is connected.
            (2, 1),
            (2, 3),
            (3, 0),
            (3, 1),
            (3, 2),
        ]);

        let mut partial_paths = PartialPaths::new(&g);

        assert_eq!(vec![0], as_vec(partial_paths.next()));
        assert_eq!(vec![0, 3], as_vec(partial_paths.next()));
        assert_eq!(vec![0, 3, 2], as_vec(partial_paths.next()));
        assert_eq!(vec![0, 3, 2, 1], as_vec(partial_paths.next()));
        assert_eq!(vec![0, 3, 1], as_vec(partial_paths.next()));
        assert_eq!(vec![0, 2], as_vec(partial_paths.next()));
        assert_eq!(vec![0, 2, 3], as_vec(partial_paths.next()));
        assert_eq!(vec![0, 2, 3, 1], as_vec(partial_paths.next()));
        assert_eq!(vec![0, 2, 1], as_vec(partial_paths.next()));
        assert_eq!(vec![0, 1], as_vec(partial_paths.next()));
        assert!(partial_paths.next().is_none());
    }

    #[test]
    fn graph_with_one_required_edge_can_be_walked_with_meaningless_skip() {
        // Create a 1 node graph
        let g = Graph::<(), ()>::from_edges(&[
            (0, 1),
            (0, 2),
            (0, 3),
            (1, 0), // The only edge from 1 is to 0.
            (2, 0), // Everything else is connected.
            (2, 1),
            (2, 3),
            (3, 0),
            (3, 1),
            (3, 2),
        ]);

        let mut partial_paths = PartialPaths::new(&g);

        assert_eq!(vec![0], as_vec(partial_paths.next()));
        assert_eq!(vec![0, 3], as_vec(partial_paths.next()));
        assert_eq!(vec![0, 3, 2], as_vec(partial_paths.next()));
        assert_eq!(vec![0, 3, 2, 1], as_vec(partial_paths.next()));
        assert_eq!(vec![0, 3, 1], as_vec(partial_paths.next()));
        assert_eq!(vec![0, 2], as_vec(partial_paths.skip_latest_node())); // Skipping 0,3,1 should jump to 0,2
        assert_eq!(vec![0, 2, 3], as_vec(partial_paths.next()));
        assert_eq!(vec![0, 2, 3, 1], as_vec(partial_paths.next()));
        assert_eq!(vec![0, 2, 1], as_vec(partial_paths.next()));
        assert_eq!(vec![0, 1], as_vec(partial_paths.next()));
        assert!(partial_paths.next().is_none());
    }

    #[test]
    fn graph_with_one_required_edge_can_be_walked_with_meaningful_skip() {
        // Create a 1 node graph
        let g = Graph::<(), ()>::from_edges(&[
            (0, 1),
            (0, 2),
            (0, 3),
            (1, 0), // The only edge from 1 is to 0.
            (2, 0), // Everything else is connected.
            (2, 1),
            (2, 3),
            (3, 0),
            (3, 1),
            (3, 2),
        ]);

        let mut partial_paths = PartialPaths::new(&g);

        assert_eq!(vec![0], as_vec(partial_paths.next()));
        assert_eq!(vec![0, 3], as_vec(partial_paths.next()));
        assert_eq!(vec![0, 2], as_vec(partial_paths.skip_latest_node())); // Skipping 0,3 should jump to 0,2
        assert_eq!(vec![0, 2, 3], as_vec(partial_paths.next()));
        assert_eq!(vec![0, 2, 3, 1], as_vec(partial_paths.next()));
        assert_eq!(vec![0, 2, 1], as_vec(partial_paths.next()));
        assert_eq!(vec![0, 1], as_vec(partial_paths.next()));
        assert!(partial_paths.next().is_none());
    }

    // --- Tests for Classification ---

    fn path(g: &Graph<(), ()>, raw: Vec<usize>) -> PartialPath<&Graph<(), ()>> {
        raw.iter()
            .map(|i| g.from_index(*i))
            .collect::<Vec<_>>()
            .into()
    }

    #[test]
    fn initial_classification_copies_graph_with_edges_undecided() {
        // Given a classification of small graph
        let g = fully_connected_graph(3);
        let c = Classification::new(&g);

        // All edges start off undecided
        assert_eq!(
            "\
            0 -> _2 _1\n\
            1 -> _2 _0\n\
            2 -> _1 _0\n\
            ",
            format!("{:?}", c)
        );
    }

    #[test]
    fn all_edges_of_partial_path_are_required() {
        let g = fully_connected_graph(4);
        let mut c = Classification::new(&g);

        // When we supply the partial path
        c.partial_path_is_required(&g, &path(&g, vec![0, 2, 1]));

        // Then all the partial path's edges (0->2, 2->1) are required and any opposites are
        // deleted.
        assert_eq!(
            "\
            0 -> _3 R2 _1\n\
            1 -> _3 _0\n\
            2 -> _3 R1\n\
            3 -> _2 _1 _0\n\
            ",
            format!("{:?}", c)
        );
    }

    #[test]
    fn setting_an_edge_required_deletes_its_opposite() {
        // Given a graph 0<-->1
        let g = Graph::<(), ()>::from_edges(&[(0, 1), (1, 0)]);
        let mut c = Classification::new(&g);
        //  Double-check 1->0 exists
        c.classes.find_edge(node_index(1), node_index(0)).unwrap();

        // When we require 0->1
        let updated = c.set_required(node_index(0), node_index(1));
        assert!(updated);

        // Then its opposite 1->0 is gone
        let opp = c.classes.find_edge(node_index(1), node_index(0));
        assert_eq!(opp, None);
    }

    #[test]
    fn all_unique_edges_are_required() {
        // Given a graph with some nodes that only have 1 incoming or outgoing edges
        // 0->1->2<-\
        //    |     |
        //    \->3<>4
        let g = Graph::<(), ()>::from_edges(&[(0, 1), (1, 2), (1, 3), (3, 4), (4, 3), (4, 2)]);

        // When we classify its edges
        let mut c = Classification::new(&g);
        let mut updated = c.unique_edges_are_required();

        // Then edges that are unique are required.
        // (Note also 4->3 was deleted because 3->4 is required.)
        assert_eq!(
            "\
            0 -> R1\n\
            1 -> _3 _2\n\
            2 ->\n\
            3 -> R4\n\
            4 -> _2\n\
            ",
            format!("{:?}", c)
        );

        // The first time we called it, we made a change
        assert!(updated);

        // Do it again
        updated = c.unique_edges_are_required();
        // Now 4->2 is required because it is the only outgoing one from 4
        // and 1->3 is required because it is the only incoming one to 3
        assert_eq!(
            "\
            0 -> R1\n\
            1 -> R3 _2\n\
            2 ->\n\
            3 -> R4\n\
            4 -> R2\n\
            ",
            format!("{:?}", c)
        );
        assert!(updated);

        // Now when we do it again, it is not updated
        assert!(!c.unique_edges_are_required());
    }

    #[test]
    fn we_make_unique_edges_required_and_delete_edges_next_to_required() {
        let g = Graph::<(), ()>::from_edges(&[
            (0, 1),
            (0, 2),
            (0, 3),
            (1, 0), // The only edge from 1 is to 0.
            (2, 0), // Everything else is connected.
            (2, 1),
            (2, 3),
            (3, 0),
            (3, 1),
            (3, 2),
        ]);

        // Given 0->3 is already required
        let mut c = Classification::new(&g);
        c.partial_path_is_required(&g, &path(&g, vec![0, 3]));

        assert_eq!(
            "\
            0 -> R3 _2 _1\n\
            1 -> _0\n\
            2 -> _3 _1 _0\n\
            3 -> _2 _1\n\
            ",
            format!("{:?}", c)
        );

        // When we make unique edges required
        c.unique_edges_are_required();

        // Then 0->1 becomes required
        assert_eq!(
            "\
            0 -> R3 _2\n\
            1 -> R0\n\
            2 -> _3 _1 _0\n\
            3 -> _2 _1\n\
            ",
            format!("{:?}", c)
        );

        // And when we force edges next to required ones to be in the opposite direction
        c.one_direction_required_forces_other_direction_for_other_edges();

        // Then everything else out of 0 or 1, or into 3 or 0, is gone
        assert_eq!(
            "\
            0 -> R3\n\
            1 -> R0\n\
            2 -> _1\n\
            3 -> _2 _1\n\
            ",
            format!("{:?}", c)
        );
    }

    #[test]
    fn two_incident_edges_means_both_required_directed_right_way() {
        // Given node 1 has two incident directed edges, flowing through it
        // 0->1->2->3
        //       \->4
        let g = Graph::<(), ()>::from_edges(&[(0, 1), (1, 2), (2, 3), (2, 4)]);

        // When we classify the edges
        let mut c = Classification::new(&g);
        let updated = c.two_incident_edges_means_both_required();

        // 1 has 2 incident edges, so they are both required
        assert_eq!(
            "\
            0 -> R1\n\
            1 -> R2\n\
            2 -> _4 _3\n\
            3 ->\n\
            4 ->\n\
            ",
            format!("{:?}", c)
        );

        // The first time we called it, we made a change
        assert!(updated);

        // Doing it again does nothing
        assert!(!c.two_incident_edges_means_both_required());
    }

    #[test]
    fn two_incident_edges_means_both_required_directed_wrong_way() {
        // Given node 1 has two incident directed edges, even though they both enter it
        // 0->1<-2->3
        //       \->4
        let g = Graph::<(), ()>::from_edges(&[(0, 1), (2, 1), (2, 3), (2, 4)]);

        // When we classify its edges
        let mut c = Classification::new(&g);
        let updated = c.two_incident_edges_means_both_required();

        // 1 has 2 incident edges, so they are both required
        assert_eq!(
            "\
            0 -> R1\n\
            1 ->\n\
            2 -> _4 _3 R1\n\
            3 ->\n\
            4 ->\n\
            ",
            format!("{:?}", c)
        );

        // The first time we called it, we made a change
        assert!(updated);

        // Doing it again does nothing
        assert!(!c.two_incident_edges_means_both_required());
    }

    #[test]
    fn two_incident_edges_means_both_required_undirected() {
        // Given node 1 has two incident undirected edges (Note that here by undirected
        // we mean a pair of edges going in both directions)
        // 0<>1<>2->3
        //       \->4
        let g = Graph::<(), ()>::from_edges(&[(0, 1), (1, 0), (1, 2), (2, 1), (2, 3), (2, 4)]);

        // When we classify its edges
        let mut c = Classification::new(&g);
        let updated = c.two_incident_edges_means_both_required();

        // 1 has 2 incident edges, so they are both semi-required, where semi-required means
        // one of the two directions is required
        assert_eq!(
            "\
            0 -> r1\n\
            1 -> r2 r0\n\
            2 -> _4 _3 r1\n\
            3 ->\n\
            4 ->\n\
            ",
            format!("{:?}", c)
        );

        // The first time we called it, we made a change
        assert!(updated);

        // Doing it again does nothing
        assert!(!c.two_incident_edges_means_both_required());
    }

    #[test]
    fn two_incident_edges_means_both_required_mixed_directed_and_undirected() {
        // Given node 1 has two incident edges, one directed and one undirected
        // (Undirected means a pair going in opposite directions.)
        // 0->1<>2->3
        //       \->4
        let g = Graph::<(), ()>::from_edges(&[(0, 1), (1, 2), (2, 1), (2, 3), (2, 4)]);

        // When we classify its edges
        let mut c = Classification::new(&g);
        let updated = c.two_incident_edges_means_both_required();

        // 1 has 2 incident edges. The directed one becomes required, and the pair become
        // semi-directed.
        assert_eq!(
            "\
            0 -> R1\n\
            1 -> r2\n\
            2 -> _4 _3 r1\n\
            3 ->\n\
            4 ->\n\
            ",
            format!("{:?}", c)
        );

        // The first time we called it, we made a change
        assert!(updated);

        // Doing it again does nothing
        assert!(!c.two_incident_edges_means_both_required());
    }

    #[test]
    fn incoming_required_makes_all_other_incident_edges_outgoing() {
        // Given node 1 has a required edge incoming
        let g =
            Graph::<(), ()>::from_edges(&[(0, 1), (1, 2), (2, 1), (1, 3), (3, 1), (1, 4), (5, 1)]);
        let mut c = Classification::new(&g);
        c.set_required(node_index(0), node_index(1));
        c.set_semirequired(node_index(1), node_index(3));

        assert_eq!(
            "\
            0 -> R1\n\
            1 -> _4 r3 _2\n\
            2 -> _1\n\
            3 -> r1\n\
            4 ->\n\
            5 -> _1\n\
            ",
            format!("{:?}", c)
        );

        // When we classify its edges
        let mut updated = c.one_direction_required_forces_other_direction_for_other_edges();

        // Other incoming edges (2->1, 3->1, 5->1) are deleted, and the semirequired whose
        // opposite was deleted has become required
        assert_eq!(
            "\
            0 -> R1\n\
            1 -> _4 R3 _2\n\
            2 ->\n3 ->\n4 ->\n5 ->\n\
            ",
            format!("{:?}", c)
        );

        // The first time we called it, we made a change
        assert!(updated);

        // Doing it again deletes the other edges
        updated = c.one_direction_required_forces_other_direction_for_other_edges();
        assert_eq!(
            "\
            0 -> R1\n\
            1 -> R3\n\
            2 ->\n3 ->\n4 ->\n5 ->\n\
            ",
            format!("{:?}", c)
        );
        assert!(updated);

        // Doing it again does nothing
        assert!(!c.one_direction_required_forces_other_direction_for_other_edges());
    }

    #[test]
    fn outgoing_required_makes_all_other_incident_edges_incoming() {
        // Given node 1 has a required edge outgoing
        let g =
            Graph::<(), ()>::from_edges(&[(1, 0), (2, 1), (1, 2), (3, 1), (1, 3), (4, 1), (1, 5)]);
        let mut c = Classification::new(&g);
        c.set_required(node_index(1), node_index(0));
        c.set_semirequired(node_index(1), node_index(3));
        assert_eq!(
            "\
            0 ->\n\
            1 -> _5 r3 _2 R0\n\
            2 -> _1\n\
            3 -> r1\n\
            4 -> _1\n\
            5 ->\n\
            ",
            format!("{:?}", c)
        );

        // When we classify its edges
        let mut updated = c.one_direction_required_forces_other_direction_for_other_edges();

        // Other outgoing edges (1->2, 1->3, 1->5) are deleted
        assert_eq!(
            "\
            0 ->\n\
            1 -> R0\n\
            2 -> _1\n\
            3 -> R1\n\
            4 -> _1\n\
            5 ->\n\
            ",
            format!("{:?}", c)
        );

        // The first time we called it, we made a change
        assert!(updated);

        updated = c.one_direction_required_forces_other_direction_for_other_edges();

        // The second time, more are deleted
        assert_eq!(
            "\
            0 ->\n\
            1 -> R0\n\
            2 ->\n\
            3 -> R1\n\
            4 ->\n\
            5 ->\n\
            ",
            format!("{:?}", c)
        );
        assert!(updated);

        // Doing it again does nothing
        assert!(!c.one_direction_required_forces_other_direction_for_other_edges());
    }

    #[test]
    fn all_others_outgoing_means_semirequired_becomes_required() {
        // Given node 1 has a semirequired edge and all others are outgoing
        let g = Graph::<(), ()>::from_edges(&[(0, 1), (0, 3), (1, 0), (1, 2), (1, 3), (2, 0)]);
        let mut c = Classification::new(&g);
        c.set_semirequired(node_index(0), node_index(1));
        assert_eq!(
            "\
            0 -> _3 r1\n\
            1 -> _3 _2 r0\n\
            2 -> _0\n\
            3 ->\n\
            ",
            format!("{:?}", c)
        );

        // When we classify its edges
        let updated = c.all_other_edges_in_one_direction_means_pair_becomes_single();

        // The semirequired edge 0<>1 becomes a single required edge 0->1
        assert_eq!(
            "\
            0 -> _3 R1\n\
            1 -> _3 _2\n\
            2 -> _0\n\
            3 ->\n\
            ",
            format!("{:?}", c)
        );

        // The first time we called it, we made a change
        assert!(updated);

        // Doing it again does nothing
        assert!(!c.all_other_edges_in_one_direction_means_pair_becomes_single());
    }

    #[test]
    fn all_others_incoming_means_semirequired_becomes_required() {
        // Given node 1 has a semirequired edge and all others are incoming
        let g = Graph::<(), ()>::from_edges(&[(1, 0), (0, 1), (2, 1), (3, 0), (3, 1)]);
        let mut c = Classification::new(&g);
        c.set_semirequired(node_index(0), node_index(1));
        assert_eq!(
            "\
            0 -> r1\n\
            1 -> r0\n\
            2 -> _1\n\
            3 -> _1 _0\n\
            ",
            format!("{:?}", c)
        );

        // When we classify its edges
        let updated = c.all_other_edges_in_one_direction_means_pair_becomes_single();

        // The semirequired edge 0<>1 becomes a single required edge 0->1
        assert_eq!(
            "\
            0 -> R1\n\
            1 ->\n\
            2 -> _1\n\
            3 -> _1 _0\n\
            ",
            format!("{:?}", c)
        );

        // The first time we called it, we made a change
        assert!(updated);

        // Doing it again does nothing
        assert!(!c.all_other_edges_in_one_direction_means_pair_becomes_single());
    }

    #[test]
    fn if_a_node_has_two_required_edges_the_others_are_deleted() {
        // Given node 1 has two edges that are required
        let g = Graph::<(), ()>::from_edges(&[(0, 1), (1, 2), (1, 3), (1, 4), (3, 1)]);
        let mut c = Classification::new(&g);
        c.set_required(node_index(0), node_index(1));
        c.set_required(node_index(1), node_index(2));
        assert_eq!(
            "\
            0 -> R1\n\
            1 -> _4 _3 R2\n\
            2 ->\n\
            3 -> _1\n\
            4 ->\n\
            ",
            format!("{:?}", c)
        );

        // When we classify its edges
        let updated = c.two_required_edges_means_delete_nonrequired();

        // The nonrequired edges of 1 are deleted
        assert_eq!(
            "\
            0 -> R1\n\
            1 -> R2\n\
            2 ->\n\
            3 ->\n\
            4 ->\n\
            ",
            format!("{:?}", c)
        );

        // The first time we called it, we made a change
        assert!(updated);

        // Doing it again does nothing
        assert!(!c.two_required_edges_means_delete_nonrequired());
    }

    #[test]
    fn if_an_incoming_node_is_required_other_incoming_are_deleted() {
        // Given node 1 has an incoming required edge
        let g = Graph::<(), ()>::from_edges(&[(0, 1), (1, 3), (2, 1), (3, 1)]);
        let mut c = Classification::new(&g);
        c.set_required(node_index(2), node_index(1));
        c.set_semirequired(node_index(1), node_index(3));
        assert_eq!(
            "\
            0 -> _1\n\
            1 -> r3\n\
            2 -> R1\n\
            3 -> r1\n\
            ",
            format!("{:?}", c)
        );

        // When we classify its edges
        let updated = c.required_edge_means_delete_others_in_same_direction();

        // The other incoming edges are deleted (and so the outgoing semirequired is required)
        assert_eq!(
            "\
            0 ->\n\
            1 -> R3\n\
            2 -> R1\n\
            3 ->\n\
            ",
            format!("{:?}", c)
        );

        // The first time we called it, we made a change
        assert!(updated);

        // Doing it again does nothing
        assert!(!c.required_edge_means_delete_others_in_same_direction());
    }

    #[test]
    fn if_an_outgoing_node_is_required_other_outgoing_are_deleted() {
        // Given node 1 has an incoming required edge
        let g = Graph::<(), ()>::from_edges(&[(1, 0), (3, 1), (1, 2), (1, 3)]);
        let mut c = Classification::new(&g);
        c.set_required(node_index(1), node_index(2));
        c.set_semirequired(node_index(1), node_index(3));
        assert_eq!(
            "\
            0 ->\n\
            1 -> r3 R2 _0\n\
            2 ->\n\
            3 -> r1\n\
            ",
            format!("{:?}", c)
        );

        // When we classify its edges
        let updated = c.required_edge_means_delete_others_in_same_direction();

        // The other outgoing edges are deleted (and so the incoming semirequired is required)
        assert_eq!(
            "\
            0 ->\n\
            1 -> R2\n\
            2 ->\n\
            3 -> R1\n\
            ",
            format!("{:?}", c)
        );

        // The first time we called it, we made a change
        assert!(updated);

        // Doing it again does nothing
        assert!(!c.required_edge_means_delete_others_in_same_direction());
    }

    #[test]
    fn any_edge_that_forms_a_circuit_with_required_edges_is_deleted() {
        // Given 3->1 forms a circuit 1->2->3->1
        let g = Graph::<(), ()>::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 1)]);
        let mut c = Classification::new(&g);
        c.set_required(node_index(0), node_index(1));
        c.set_required(node_index(1), node_index(2));
        c.set_required(node_index(2), node_index(3));
        assert_eq!(
            "\
            0 -> R1\n\
            1 -> R2\n\
            2 -> R3\n\
            3 -> _1\n\
            ",
            format!("{:?}", c)
        );

        // When we classify its edges
        let updated = c.delete_invalid_circuit_forming_edges();

        // The edge that forms the circuit is deleted
        assert_eq!(
            "\
            0 -> R1\n\
            1 -> R2\n\
            2 -> R3\n\
            3 ->\n\
            ",
            format!("{:?}", c)
        );

        // The first time we called it, we made a change
        assert!(updated);

        // Doing it again does nothing
        assert!(!c.delete_invalid_circuit_forming_edges());
    }

    #[test]
    fn edge_that_does_not_form_a_circuit_with_required_edges_is_not_deleted() {
        // Given 3->1 does not form a circuit because 1->2 is not required
        let g = Graph::<(), ()>::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 1)]);
        let mut c = Classification::new(&g);
        c.set_required(node_index(0), node_index(1));
        c.set_required(node_index(2), node_index(3));
        assert_eq!(
            "\
            0 -> R1\n\
            1 -> _2\n\
            2 -> R3\n\
            3 -> _1\n\
            ",
            format!("{:?}", c)
        );

        // When we classify its edges
        let updated = c.delete_invalid_circuit_forming_edges();

        // 3->1 is not deleted
        assert_eq!(
            "\
            0 -> R1\n\
            1 -> _2\n\
            2 -> R3\n\
            3 -> _1\n\
            ",
            format!("{:?}", c)
        );
        assert!(!updated);
    }

    #[test]
    fn edge_that_completes_the_hamiltonian_circuit_is_not_deleted() {
        // Given 3->0 does form a circuit, but it's the complete circuit
        let g = Graph::<(), ()>::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 0)]);
        let mut c = Classification::new(&g);
        c.set_required(node_index(0), node_index(1));
        c.set_required(node_index(1), node_index(2));
        c.set_required(node_index(2), node_index(3));
        assert_eq!(
            "\
            0 -> R1\n\
            1 -> R2\n\
            2 -> R3\n\
            3 -> _0\n\
            ",
            format!("{:?}", c)
        );

        // When we classify its edges
        let updated = c.delete_invalid_circuit_forming_edges();

        // 3->0 is not deleted
        assert_eq!(
            "\
            0 -> R1\n\
            1 -> R2\n\
            2 -> R3\n\
            3 -> _0\n\
            ",
            format!("{:?}", c)
        );
        assert!(!updated);
    }

    #[test]
    fn circuit_within_required_does_not_get_stuck() {
        // Given a graph with a circuit of required nodes
        let g = Graph::<(), ()>::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 1), (4, 2)]);
        let mut c = Classification::new(&g);
        c.set_required(node_index(0), node_index(1));
        c.set_required(node_index(1), node_index(2));
        c.set_required(node_index(2), node_index(3));
        c.set_required(node_index(3), node_index(1));
        assert_eq!(
            "\
            0 -> R1\n\
            1 -> R2\n\
            2 -> R3\n\
            3 -> R1\n\
            4 -> _2\n\
            ",
            format!("{:?}", c)
        );

        // When we classify its edges
        let updated = c.delete_invalid_circuit_forming_edges();

        // 4->2 is not deleted because it does not form a circuit with required nodes
        // But more importantly, we don't end up in an infinite loop
        assert_eq!(
            "\
            0 -> R1\n\
            1 -> R2\n\
            2 -> R3\n\
            3 -> R1\n\
            4 -> _2\n\
            ",
            format!("{:?}", c)
        );
        assert!(!updated);

        // When we do it again, nothing changes
        assert!(!c.delete_invalid_circuit_forming_edges());
    }

    #[test]
    fn single_node_is_isolated() {
        // Given a graph with a node with no edges
        let mut g = Graph::<(), ()>::from_edges(&[(0, 1), (1, 2)]);
        g.add_node(());
        let c = Classification::new(&g);
        assert_eq!(
            "\
            0 -> _1\n\
            1 -> _2\n\
            2 ->\n\
            3 ->\n\
            ",
            format!("{:?}", c)
        );

        // Then we detect the isolated node
        assert!(c.any_node_with_zero_or_one_edge());
    }

    #[test]
    fn node_with_one_edge_incoming_is_a_dead_end() {
        // Given node 4 has one edge (incoming)
        let g = Graph::<(), ()>::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 0), (3, 4)]);
        let c = Classification::new(&g);
        assert_eq!(
            "\
            0 -> _1\n\
            1 -> _2\n\
            2 -> _3\n\
            3 -> _4 _0\n\
            4 ->\n\
            ",
            format!("{:?}", c)
        );

        // Then we detect the dead end
        assert!(c.any_node_with_zero_or_one_edge());
    }

    #[test]
    fn node_with_one_edge_outgoing_is_a_dead_end() {
        // Given node 4 has one edge (outgoing)
        let g = Graph::<(), ()>::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 0), (4, 3)]);
        let c = Classification::new(&g);
        assert_eq!(
            "\
            0 -> _1\n\
            1 -> _2\n\
            2 -> _3\n\
            3 -> _0\n\
            4 -> _3\n\
            ",
            format!("{:?}", c)
        );

        // Then we detect the dead end
        assert!(c.any_node_with_zero_or_one_edge());
    }

    #[test]
    fn graph_with_no_isolated_nodes_or_dead_ends_is_detected() {
        // Given a graph with no nodes with <=1 edges
        let g = Graph::<(), ()>::from_edges(&[(0, 1), (0, 3), (1, 2), (2, 1), (2, 3)]);
        let c = Classification::new(&g);
        assert_eq!(
            "\
            0 -> _3 _1\n\
            1 -> _2\n\
            2 -> _3 _1\n\
            3 ->\n\
            ",
            format!("{:?}", c)
        );

        // Then we detect none are isolated
        assert!(!c.any_node_with_zero_or_one_edge());
    }

    #[test]
    fn node_with_no_incoming_edges_is_detected() {
        // Given 0 has no incoming edges
        let g = Graph::<(), ()>::from_edges(&[(0, 1)]);
        let c = Classification::new(&g);
        assert_eq!(
            "\
            0 -> _1\n\
            1 ->\n\
            ",
            format!("{:?}", c)
        );

        // Then we detect the problem
        assert!(c.any_node_with_no_incoming_or_no_outgoing());
    }

    #[test]
    fn node_with_no_outgoing_edges_is_detected() {
        // Given 0 has no outgoing edges
        let g = Graph::<(), ()>::from_edges(&[(1, 0)]);
        let c = Classification::new(&g);
        assert_eq!(
            "\
            0 ->\n\
            1 -> _0\n\
            ",
            format!("{:?}", c)
        );

        // Then we detect the problem
        assert!(c.any_node_with_no_incoming_or_no_outgoing());
    }

    #[test]
    fn graph_where_nodes_have_edges_in_both_directions_is_allowed() {
        // Given all nodes have incoming and outgoing edges
        let g = Graph::<(), ()>::from_edges(&[(0, 1), (1, 0)]);
        let c = Classification::new(&g);
        assert_eq!(
            "\
            0 -> _1\n\
            1 -> _0\n\
            ",
            format!("{:?}", c)
        );

        // Then wedon't flag this graph
        assert!(!c.any_node_with_no_incoming_or_no_outgoing());
    }

    #[test]
    fn node_with_two_required_incoming_is_detected() {
        // Given 2 has 2 incoming required edges
        let g = Graph::<(), ()>::from_edges(&[(0, 1), (1, 2), (2, 0), (3, 2)]);
        let mut c = Classification::new(&g);
        c.set_required(node_index(0), node_index(1));
        c.set_required(node_index(1), node_index(2));
        c.set_required(node_index(2), node_index(0));
        c.set_required(node_index(3), node_index(2));
        assert_eq!(
            "\
            0 -> R1\n\
            1 -> R2\n\
            2 -> R0\n\
            3 -> R2\n\
            ",
            format!("{:?}", c)
        );

        // This graph is not admissable
        assert!(c.any_node_with_two_required_edges_in_same_direction());
    }

    #[test]
    fn node_with_two_required_outgoing_is_detected() {
        // Given 2 has 2 outgoing required edges
        let g = Graph::<(), ()>::from_edges(&[(0, 1), (1, 2), (2, 0), (2, 3)]);
        let mut c = Classification::new(&g);
        c.set_required(node_index(0), node_index(1));
        c.set_required(node_index(1), node_index(2));
        c.set_required(node_index(2), node_index(0));
        c.set_required(node_index(2), node_index(3));
        assert_eq!(
            "\
            0 -> R1\n\
            1 -> R2\n\
            2 -> R3 R0\n\
            3 ->\n\
            ",
            format!("{:?}", c)
        );

        // This graph is not admissable
        assert!(c.any_node_with_two_required_edges_in_same_direction());
    }

    #[test]
    fn graph_where_nodes_have_less_than_two_required_edges_in_each_direction_is_allowed() {
        // Given no node has >1 required edge in a direction
        let g = Graph::<(), ()>::from_edges(&[(0, 1), (1, 2), (2, 0)]);
        let mut c = Classification::new(&g);
        c.set_required(node_index(0), node_index(1));
        c.set_required(node_index(1), node_index(2));
        c.set_required(node_index(2), node_index(0));
        assert_eq!(
            "\
            0 -> R1\n\
            1 -> R2\n\
            2 -> R0\n\
            ",
            format!("{:?}", c)
        );

        // Then we don't flag this graph
        assert!(!c.any_node_with_two_required_edges_in_same_direction());
    }

    #[test]
    fn node_with_three_semirequired_edge_pairs_is_detected() {
        // Given 0 has 3 pairs of semirequired edges
        let g = Graph::<(), ()>::from_edges(&[(0, 1), (1, 0), (0, 2), (2, 0), (0, 3), (3, 0)]);
        let mut c = Classification::new(&g);
        c.set_semirequired(node_index(0), node_index(1));
        c.set_semirequired(node_index(0), node_index(2));
        c.set_semirequired(node_index(0), node_index(3));
        assert_eq!(
            "\
            0 -> r3 r2 r1\n\
            1 -> r0\n\
            2 -> r0\n\
            3 -> r0\n\
            ",
            format!("{:?}", c)
        );

        // This graph is not admissable
        assert!(c.any_node_with_three_required_edges());
    }

    #[test]
    fn node_with_two_semirequired_edge_pairs_is_allowed() {
        // Given 0 has 3 pairs of edges but only 2 are semirequired
        let g = Graph::<(), ()>::from_edges(&[(0, 1), (1, 0), (0, 2), (2, 0), (0, 3), (3, 0)]);
        let mut c = Classification::new(&g);
        c.set_semirequired(node_index(0), node_index(1));
        c.set_semirequired(node_index(0), node_index(2));
        assert_eq!(
            "\
            0 -> _3 r2 r1\n\
            1 -> r0\n\
            2 -> r0\n\
            3 -> _0\n\
            ",
            format!("{:?}", c)
        );

        // This graph is allowed
        assert!(!c.any_node_with_three_required_edges());
    }

    #[test]
    fn node_with_three_required_edges_is_detected() {
        // Given 0 has 3 required edges
        let g = Graph::<(), ()>::from_edges(&[(0, 1), (2, 0), (0, 3), (4, 0)]);
        let mut c = Classification::new(&g);
        c.set_required(node_index(0), node_index(1));
        c.set_required(node_index(2), node_index(0));
        c.set_required(node_index(0), node_index(3));
        assert_eq!(
            "\
            0 -> R3 R1\n\
            1 ->\n\
            2 -> R0\n\
            3 ->\n\
            4 -> _0\n\
            ",
            format!("{:?}", c)
        );

        // This graph is not admissable
        assert!(c.any_node_with_three_required_edges());
    }

    #[test]
    fn node_with_a_mix_of_required_and_semirequired_is_detected() {
        // Given 0 has 3 required edges, but one is semirequired
        let g = Graph::<(), ()>::from_edges(&[(0, 1), (1, 0), (2, 0), (0, 3), (4, 0)]);
        let mut c = Classification::new(&g);
        c.set_semirequired(node_index(0), node_index(1));
        c.set_required(node_index(2), node_index(0));
        c.set_required(node_index(0), node_index(3));
        assert_eq!(
            "\
            0 -> R3 r1\n\
            1 -> r0\n\
            2 -> R0\n\
            3 ->\n\
            4 -> _0\n\
            ",
            format!("{:?}", c)
        );

        // This graph is not admissable
        assert!(c.any_node_with_three_required_edges());
    }

    #[test]
    fn node_with_one_required_and_one_semirequired_is_allowed() {
        // Given 0 has a required and a semirequired edge
        let g = Graph::<(), ()>::from_edges(&[(0, 1), (1, 0), (2, 0), (0, 3), (4, 0)]);
        let mut c = Classification::new(&g);
        c.set_semirequired(node_index(0), node_index(1));
        c.set_required(node_index(2), node_index(0));
        assert_eq!(
            "\
            0 -> _3 r1\n\
            1 -> r0\n\
            2 -> R0\n\
            3 ->\n\
            4 -> _0\n\
            ",
            format!("{:?}", c)
        );

        // This graph is allowed
        assert!(!c.any_node_with_three_required_edges());
    }

    #[test]
    fn circuit_of_required_nodes_is_detected() {
        // Given 0->1->2 is a circuit and they are all required
        let g = Graph::<(), ()>::from_edges(&[(0, 1), (1, 2), (2, 0), (0, 3), (4, 0)]);
        let mut c = Classification::new(&g);
        c.set_required(node_index(0), node_index(1));
        c.set_required(node_index(1), node_index(2));
        c.set_required(node_index(2), node_index(0));
        assert_eq!(
            "\
            0 -> _3 R1\n\
            1 -> R2\n\
            2 -> R0\n\
            3 ->\n\
            4 -> _0\n\
            ",
            format!("{:?}", c)
        );

        // This graph is not admissable
        assert!(c.any_circuit_of_required_except_hamiltonian());
    }

    #[test]
    fn later_circuit_of_required_nodes_is_detected() {
        // Given 2->3-4 is a circuit and they are all required
        let g = Graph::<(), ()>::from_edges(&[(0, 1), (1, 0), (2, 3), (3, 4), (4, 2)]);
        let mut c = Classification::new(&g);
        c.set_required(node_index(2), node_index(3));
        c.set_required(node_index(3), node_index(4));
        c.set_required(node_index(4), node_index(2));
        assert_eq!(
            "\
            0 -> _1\n\
            1 -> _0\n\
            2 -> R3\n\
            3 -> R4\n\
            4 -> R2\n\
            ",
            format!("{:?}", c)
        );

        // This graph is not admissable
        assert!(c.any_circuit_of_required_except_hamiltonian());
    }

    #[test]
    fn no_circuit_of_required_nodes_is_allowed() {
        // Given there is no circuit of required edges
        let g = Graph::<(), ()>::from_edges(&[(0, 1), (1, 2), (2, 0), (0, 3), (4, 0)]);
        let mut c = Classification::new(&g);
        c.set_required(node_index(0), node_index(1));
        c.set_required(node_index(1), node_index(2));
        assert_eq!(
            "\
            0 -> _3 R1\n\
            1 -> R2\n\
            2 -> _0\n\
            3 ->\n\
            4 -> _0\n\
            ",
            format!("{:?}", c)
        );

        // This graph is admissable
        assert!(!c.any_circuit_of_required_except_hamiltonian());
    }

    #[test]
    fn hamiltonian_circuit_of_required_nodes_is_allowed() {
        // Given there is a required circuit but it covers all nodes
        let g = Graph::<(), ()>::from_edges(&[(0, 1), (1, 2), (2, 0)]);
        let mut c = Classification::new(&g);
        c.set_required(node_index(0), node_index(1));
        c.set_required(node_index(1), node_index(2));
        c.set_required(node_index(2), node_index(0));
        assert_eq!(
            "\
            0 -> R1\n\
            1 -> R2\n\
            2 -> R0\n\
            ",
            format!("{:?}", c)
        );

        // This graph is admissable
        assert!(!c.any_circuit_of_required_except_hamiltonian());
    }

    // --- Tests for hamiltonian_circuits ---

    fn paths(raw: Vec<Vec<usize>>) -> Vec<Vec<<Graph<(), ()> as GraphBase>::NodeId>> {
        raw.iter()
            .map(|p| p.iter().map(|n| node_index(*n)).collect::<Vec<_>>())
            .collect()
    }

    #[test]
    fn empty_graph_has_no_circuits() {
        // Create an empty graph
        let g = Graph::<(), ()>::new();

        let circuits: Vec<_> = hamiltonian_circuits_directed(&g).collect();

        // It has no circuits
        assert_eq!(circuits, Vec::<Vec::<_>>::new());
    }

    #[test]
    fn graph_with_one_node_has_no_circuits() {
        // Create a 1 node graph
        let g = fully_connected_graph(1);

        let circuits: Vec<_> = hamiltonian_circuits_directed(&g).collect();

        // It has no circuits (by convention)
        assert_eq!(circuits, Vec::<Vec::<_>>::new());
    }

    #[test]
    fn fc_graph_with_2_nodes_has_no_circuits() {
        // Create a 2 node graph
        let g = fully_connected_graph(2);

        let circuits: Vec<_> = hamiltonian_circuits_directed(&g).collect();

        // It has no circuits (by convention)
        assert_eq!(circuits, Vec::<Vec::<_>>::new());
    }

    #[test]
    fn fc_3_graph_has_2_circuits() {
        // Create a fully-connected digraph with 3 nodes
        let g = fully_connected_graph(3);

        let circuits: Vec<_> = hamiltonian_circuits_directed(&g).collect();

        // It has 2 circuits
        assert_eq!(circuits, paths(vec![vec![0, 2, 1], vec![0, 1, 2]]));
    }

    #[test]
    fn fc_4_graph_has_6_circuits() {
        // Create a fully-connected digraph with 4 nodes
        let g = fully_connected_graph(4);

        let circuits: Vec<_> = hamiltonian_circuits_directed(&g).collect();

        // It has 6 circuits
        assert_eq!(
            circuits,
            paths(vec![
                vec![0, 3, 2, 1],
                vec![0, 3, 1, 2],
                vec![0, 2, 3, 1],
                vec![0, 2, 1, 3],
                vec![0, 1, 3, 2],
                vec![0, 1, 2, 3],
            ])
        );
    }

    #[test]
    fn fc_5_graph_has_24_circuits() {
        // Create a fully-connected digraph with 4 nodes
        let g = fully_connected_graph(5);

        let circuits: Vec<_> = hamiltonian_circuits_directed(&g).collect();

        // It has 24 circuits
        assert_eq!(circuits.len(), 24);
    }

    #[test]
    fn fc_6_graph_has_120_circuits() {
        // Create a fully-connected digraph with 4 nodes
        let g = fully_connected_graph(6);

        let circuits: Vec<_> = hamiltonian_circuits_directed(&g).collect();

        // It has 120 circuits
        assert_eq!(circuits.len(), 120);
    }

    #[test]
    fn complex_graph_with_9_nodes_has_many_circuits() {
        let g = Graph::<(), ()>::from_edges(&[
            (8, 8),
            (0, 1),
            (0, 2),
            (0, 3),
            (0, 4),
            (0, 5),
            (0, 6),
            (0, 7),
            (0, 8),
            (1, 0),
            (8, 7),
            (1, 2),
            (1, 3),
            (1, 4),
            (1, 5),
            (1, 6),
            (1, 7),
            (1, 8),
            (2, 0),
            (2, 1),
            (8, 6),
            (2, 3),
            (2, 4),
            (2, 5),
            (2, 6),
            (2, 7),
            (2, 8),
            (3, 0),
            (3, 1),
            (3, 2),
            (8, 5),
            (3, 4),
            (3, 5),
            (3, 6),
            (3, 7),
            (3, 8),
            (4, 0),
            (4, 1),
            (4, 2),
            (4, 3),
            (8, 4),
            (4, 5),
            (4, 6),
            (4, 7),
            (4, 8),
            (5, 0),
            (5, 1),
            (5, 2),
            (5, 3),
            (5, 4),
            (8, 3),
            (5, 6),
            (5, 7),
            (5, 8),
            (6, 0),
            (6, 1),
            (6, 2),
            (6, 3),
            (6, 4),
            (6, 5),
            (8, 2),
            (6, 7),
            (6, 8),
            (7, 0),
            (7, 1),
            (7, 2),
            (7, 3),
            (7, 4),
            (7, 5),
            (7, 6),
            (8, 1),
            (7, 8),
            (8, 0),
        ]);

        // Limit ourselves to 200
        let circuits: Vec<_> = hamiltonian_circuits_directed(&g).take(200).collect();

        // We find all 200
        assert_eq!(circuits.len(), 200);
    }

    #[test]
    fn tree_with_2_nodes_has_no_circuits() {
        // Create a 2 node tree
        let g = Graph::<(), ()>::from_edges(&[(0, 1)]);

        let circuits: Vec<_> = hamiltonian_circuits_directed(&g).collect();

        // It has no circuits
        assert_eq!(circuits, Vec::<Vec::<_>>::new());
    }

    #[test]
    fn long_single_circuit() {
        // 0->4->1->2->3->5
        // |              |
        //  \------<-----/
        let g = Graph::<(), ()>::from_edges(&[(0, 4), (4, 1), (1, 2), (2, 3), (3, 5), (5, 0)]);

        let circuits: Vec<_> = hamiltonian_circuits_directed(&g).collect();

        assert_eq!(circuits, paths(vec![vec![0, 4, 1, 2, 3, 5]]));
    }

    #[test]
    fn long_single_circuit_broken() {
        // 0->4->1->2  3->5
        // |              |
        //  \------<-----/
        let g = Graph::<(), ()>::from_edges(&[(0, 4), (4, 1), (1, 2), (3, 5), (5, 0)]);

        let circuits: Vec<_> = hamiltonian_circuits_directed(&g).collect();

        assert_eq!(circuits, Vec::<Vec::<_>>::new());
    }

    #[test]
    fn two_long_circuits() {
        let g = Graph::<(), ()>::from_edges(&[
            (0, 1),
            (1, 0),
            (1, 2),
            (2, 1),
            (2, 3),
            (3, 2),
            (3, 4), //     -<->1<-->2<-->3<>-
            (4, 3), //    /                  \
            (0, 5), //   0                    4
            (5, 0), //    \                  /
            (5, 6), //     -<->5<-->6<-->7<>-
            (6, 5),
            (6, 7),
            (7, 6),
            (7, 4),
            (4, 7),
        ]);

        let circuits: Vec<_> = hamiltonian_circuits_directed(&g).collect();

        assert_eq!(
            circuits,
            paths(vec![
                vec![0, 5, 6, 7, 4, 3, 2, 1],
                vec![0, 1, 2, 3, 4, 7, 6, 5],
            ])
        );
    }

    #[test]
    fn bipartite_graphs_circuit_numbers() {
        assert_eq!(
            hamiltonian_circuits_directed(&complete_bipartite_graph(1)).count(),
            0
        );
        assert_eq!(
            hamiltonian_circuits_directed(&complete_bipartite_graph(2)).count(),
            2
        );
        assert_eq!(
            hamiltonian_circuits_directed(&complete_bipartite_graph(3)).count(),
            12
        );
        assert_eq!(
            hamiltonian_circuits_directed(&complete_bipartite_graph(4)).count(),
            144
        );
    }

    #[test]
    fn circuit_with_one_backwards_edge_has_no_hamiltonian_circuits() {
        let g = Graph::<(), ()>::from_edges(&[
            (0, 1),
            (2, 1), // 2->1 is backwards
            (2, 3), //     --->1<---2->
            (2, 3), //    /            \
            (3, 4), //   0              3
            (4, 5), //    \            /
            (5, 0), //     -<--5<---4<-
        ]);

        let circuits: Vec<_> = hamiltonian_circuits_directed(&g).collect();

        assert_eq!(circuits, Vec::<Vec::<_>>::new());
    }

    #[test]
    fn one_edge_required_circuit_is_found() {
        let g = Graph::<(), ()>::from_edges(&[
            (0, 1),
            (0, 2),
            (0, 3),
            (1, 0), // The only edge from 1 is to 0.
            (2, 0), // Everything else is connected.
            (2, 1),
            (2, 3),
            (3, 0),
            (3, 1),
            (3, 2),
        ]);

        let circuits: Vec<_> = hamiltonian_circuits_directed(&g).collect();

        assert_eq!(circuits, paths(vec![vec![0, 3, 2, 1], vec![0, 2, 3, 1],]));
    }

    // --- Complexity tests ---

    fn num_paths_visited(g: Graph<(), ()>) -> usize {
        let mut hc = HamiltonianCircuits::new(&g);
        for _c in &mut hc {}
        hc.partial_paths_visited()
    }

    #[test]
    fn finding_circuits_in_connected_graphs_complexity() {
        assert_eq!(num_paths_visited(fully_connected_graph(3)), 6);
        assert_eq!(num_paths_visited(fully_connected_graph(4)), 17);
        assert_eq!(num_paths_visited(fully_connected_graph(5)), 66);
        assert_eq!(num_paths_visited(fully_connected_graph(6)), 327);
    }

    #[test]
    fn finding_circuits_in_bipartite_graphs_complexity() {
        assert_eq!(num_paths_visited(complete_bipartite_graph(2)), 8);
        assert_eq!(num_paths_visited(complete_bipartite_graph(3)), 47);
        assert_eq!(num_paths_visited(complete_bipartite_graph(4)), 558);
    }

    #[test]
    fn finding_circuits_in_acyclic_graph_complexity() {
        // Given a graph that trivially has no Hamiltonian circuit
        let g = Graph::<(), ()>::from_edges(&[
            (0, 1),
            (1, 2),
            (2, 3), // 0->1->2->3->4->5
            (3, 4),
            (4, 5),
        ]);

        // We should not waste much time figuring that out
        assert_eq!(num_paths_visited(g), 2);
    }

    #[test]
    fn finding_the_first_circuit_in_a_large_graph_is_doable() {
        // Given a large graph
        let g = fully_connected_graph(40);

        // When we find the first Hamiltonian circuit
        let mut hc = HamiltonianCircuits::new(&g);
        let circuit = hc.next().expect("No circuit found!");

        // It works without taking a long time
        assert_eq!(circuit.len(), 40);
        assert_eq!(hc.partial_paths_visited(), 40);
    }
}
