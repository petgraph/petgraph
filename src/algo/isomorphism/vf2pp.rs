use super::{
    label::{NoNodeLabel, NodeLabel, DEFAULT_NODE_LABEL},
    semantic::{EdgeMatcher, NoSemanticMatch, NodeMatcher},
};
use crate::data::DataMap;
use crate::visit::{
    GraphBase, GraphProp, IntoEdgesDirected, IntoNeighborsDirected, IntoNeighborsUnirected,
    IntoNodeIdentifiers, NodeCount, NodeIndexable,
};
use crate::{Incoming, Outgoing};
use std::{cmp::Ordering, collections::BinaryHeap, marker::PhantomData};

const NOT_IN_MAPPING: usize = usize::MAX;
const G0_CON_IN_MAPPING: usize = usize::MAX;
const G1_CON_IN_MAPPING: usize = usize::MAX;

pub struct Vf2ppIsomorphismMatcher<'a, G0, G1, NM, EM>
where
    G0: GraphBase,
    G1: GraphBase + 'a,
    NM: NodeMatcher<G0, G1>,
    EM: EdgeMatcher<G0, G1>,
{
    vf2pp: VF2PP<G0, G1>,
    node_matcher: NM,
    edge_matcher: EM,
    mapping: Vec<usize>,
    depth: usize,
    g1_candidate_nodes_iter: Vec<Box<dyn Iterator<Item = G1::NodeId> + 'a>>,
    // g1_cons[i] mean the number of neighbors of `i`, i is node_idx value of g1
    // usize::MAX indicate it already exist in the mapping
    g1_node_cons: Vec<usize>,
    subgraph: bool,
}

impl<'a, G0, G1, NM, EM> Vf2ppIsomorphismMatcher<'a, G0, G1, NM, EM>
where
    G0: IntoNeighborsDirected
        + IntoNeighborsUnirected
        + IntoNodeIdentifiers
        + NodeIndexable
        + NodeCount
        + GraphProp,
    G1: IntoNeighborsDirected
        + IntoNeighborsUnirected
        + IntoNodeIdentifiers
        + NodeIndexable
        + NodeCount
        + GraphProp
        + 'a,
    NM: NodeMatcher<G0, G1>,
    EM: EdgeMatcher<G0, G1>,
{
    fn new(vf2pp: VF2PP<G0, G1>, node_matcher: NM, edge_matcher: EM, subgraph: bool) -> Self {
        let g0_node_count = vf2pp.g0.node_count();
        let g1_node_count = vf2pp.g1.node_count();

        Self {
            vf2pp,
            node_matcher,
            edge_matcher,
            mapping: vec![usize::MAX; g0_node_count],
            depth: 0usize,
            g1_node_cons: vec![0usize; g1_node_count],
            g1_candidate_nodes_iter: vec![],
            subgraph,
        }
    }

    pub fn is_match(&mut self) -> bool {
        isomorphism_match(
            self.vf2pp.g0,
            self.vf2pp.g1,
            &self.vf2pp.roots,
            &self.vf2pp.matching_order,
            &mut self.mapping,
            &self.vf2pp.r_in_out,
            &self.vf2pp.r_new,
            &self.vf2pp.g0_node_labels,
            &self.vf2pp.g1_node_labels,
            &mut self.g1_node_cons,
            &mut self.g1_candidate_nodes_iter,
            &mut self.depth,
            self.vf2pp.max_label,
            &mut self.node_matcher,
            &mut self.edge_matcher,
            self.subgraph,
        )
        .is_some()
    }
}

impl<'a, G0, G1, NM, EM> Iterator for Vf2ppIsomorphismMatcher<'a, G0, G1, NM, EM>
where
    G0: IntoNeighborsDirected
        + IntoNeighborsUnirected
        + IntoNodeIdentifiers
        + NodeIndexable
        + NodeCount
        + GraphProp,
    G1: IntoNeighborsDirected
        + IntoNeighborsUnirected
        + IntoNodeIdentifiers
        + NodeIndexable
        + NodeCount
        + GraphProp
        + 'a,
    NM: NodeMatcher<G0, G1>,
    EM: EdgeMatcher<G0, G1>,
{
    type Item = Vec<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        // pick candidate according the ordering we build in step 1, and for root node, we need to check
        // every possible node in g1, but for non-root node, the only possible nodes is the mapping of its sibling
        // node's neighbors in g1. And check fesibility and cut impossible label based in RnewRoutin info.
        isomorphism_match(
            self.vf2pp.g0,
            self.vf2pp.g1,
            &self.vf2pp.roots,
            &self.vf2pp.matching_order,
            &mut self.mapping,
            &self.vf2pp.r_in_out,
            &self.vf2pp.r_new,
            &self.vf2pp.g0_node_labels,
            &self.vf2pp.g1_node_labels,
            &mut self.g1_node_cons,
            &mut self.g1_candidate_nodes_iter,
            &mut self.depth,
            self.vf2pp.max_label,
            &mut self.node_matcher,
            &mut self.edge_matcher,
            self.subgraph,
        )
    }
}

struct VF2PP<G0, G1>
where
    G0: GraphBase,
    G1: GraphBase,
{
    g0: G0,
    g1: G1,
    max_label: usize,
    g0_node_labels: Vec<usize>,
    g1_node_labels: Vec<usize>,
    matching_order: Vec<usize>,
    r_in_out: Vec<Vec<(usize, usize)>>,
    r_new: Vec<Vec<(usize, usize)>>,
    roots: Vec<bool>,
}

impl<G0, G1> VF2PP<G0, G1>
where
    G0: IntoNeighborsDirected
        + IntoNeighborsUnirected
        + IntoNodeIdentifiers
        + NodeIndexable
        + NodeCount
        + GraphProp,
    G1: IntoNeighborsDirected
        + IntoNeighborsUnirected
        + IntoNodeIdentifiers
        + NodeIndexable
        + NodeCount
        + GraphProp,
{
    fn new<F0, F1>(g0: G0, g1: G1, mut label0: F0, mut label1: F1) -> Self
    where
        F0: NodeLabel<G0>,
        F1: NodeLabel<G1>,
    {
        // First, we need to sort the G1's nodes according the following way,
        // * label rarity of each node, the more rarity, the ordering is more high
        // * the degree of each node, the more degree mean ordering is higher
        // * use a bfs way to decide the ordering of nodes, for every level of nodes,
        // except the root level, more connections mean higher ordering, if the connections num is the same,
        // then the degrees decide the ordering, last is the label rarity
        let (g0_node_labels, g0_max_label) = Self::init_graph_nodes_labels(g0, &mut label0);
        let (g1_node_labels, g1_max_label) = Self::init_graph_nodes_labels(g1, &mut label1);
        let max_label = std::cmp::max(g0_max_label, g1_max_label);

        let mut roots = vec![false; g0.node_count()];
        let matching_order =
            Self::init_matching_ordering(g0, &g0_node_labels, &mut roots, g0_max_label);
        // Second, init RnewRinout nums for every nodes in G1, this will be used to cut labels in third stage.
        let (r_new, r_in_out) =
            Self::init_r_new_inout(g0, &matching_order, &g0_node_labels, g0_max_label);

        Self {
            g0,
            g1,
            max_label,
            g0_node_labels,
            g1_node_labels,
            matching_order,
            r_in_out,
            r_new,
            roots,
        }
    }

    /// initilize all graph node labels, node_labels[i] mean the corresponding label of node_idx `i`.
    /// And the number of a node label is less indicate the label is more rare
    fn init_graph_nodes_labels<G, F>(g: G, label: &mut F) -> (Vec<usize>, usize)
    where
        G: IntoNodeIdentifiers + NodeCount + NodeIndexable,
        F: NodeLabel<G>,
    {
        let node_count = g.node_count();
        let mut max_label = 0;
        let mut node_lables = vec![DEFAULT_NODE_LABEL; node_count];

        for node in g.node_identifiers() {
            let node_idx = g.to_index(node);
            let node_label = NodeLabel::get_node_label(label, g, node);
            node_lables[node_idx] = node_label;
            if max_label < node_label {
                max_label = node_label;
            }
        }

        (node_lables, max_label)
    }

    /// initilize the ordering of g0, the `label_tmp[i]` mean the number of nodes with same label `i`
    fn init_matching_ordering<G>(
        g: G,
        node_labels: &Vec<usize>,
        roots: &mut Vec<bool>,
        max_label: usize,
    ) -> Vec<usize>
    where
        G: IntoNeighborsDirected
            + IntoNeighborsUnirected
            + IntoNodeIdentifiers
            + NodeCount
            + NodeIndexable
            + GraphProp,
    {
        let node_count = g.node_count();
        // init every node label's number
        let mut node_label_num = vec![0usize; max_label + 1];
        for label in node_labels.iter() {
            node_label_num[*label] += 1;
        }

        // init every node's connection number, it's okay to only iterate outcoming edges,
        // cause we will increment for both node and its neighbor.
        let mut node_con_num = vec![0; node_count];
        for node in g.node_identifiers() {
            let node_id = g.to_index(node);
            for neighbor in g.neighbors_directed(node, Outgoing) {
                let neighbor_id = g.to_index(neighbor);
                node_con_num[neighbor_id] += 1;
                node_con_num[node_id] += 1;
            }
        }

        let mut matching_order = vec![usize::MAX; node_count];
        let mut added = vec![false; node_count];
        let mut order_idx = 0usize;
        // We need to choose all possible root nodes, then do BFS search using the choosen node as root node
        while order_idx < node_count {
            // select a root node for bfs search ordering
            let mut root_node_id = usize::MAX;
            for node in g.node_identifiers() {
                let node_id = g.to_index(node);
                if !added[node_id] {
                    if root_node_id == usize::MAX {
                        root_node_id = node_id;
                    } else {
                        match node_label_num[node_labels[node_id]]
                            .cmp(&node_label_num[node_labels[root_node_id]])
                        {
                            Ordering::Less => {
                                root_node_id = node_id;
                            }
                            Ordering::Equal => {
                                if node_con_num[node_id] > node_con_num[root_node_id] {
                                    root_node_id = node_id;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }

            roots[root_node_id] = true;
            bfs_search_ordering(
                g,
                root_node_id,
                &mut order_idx,
                &mut added,
                &mut matching_order,
                &node_con_num,
                node_labels,
                &mut node_label_num,
            );
        }

        debug_assert!(!matching_order.iter().any(|&v| v == usize::MAX));

        struct BfsLayerNode {
            node_idx: usize,
            bfs_cons_num: usize,
            all_cons_num: usize,
            label_num: usize,
        }

        impl Ord for BfsLayerNode {
            fn cmp(&self, other: &Self) -> Ordering {
                match self.bfs_cons_num.cmp(&other.bfs_cons_num) {
                    Ordering::Greater => Ordering::Greater,
                    Ordering::Less => Ordering::Less,
                    Ordering::Equal => {
                        if self.all_cons_num > other.all_cons_num {
                            return Ordering::Greater;
                        } else if self.all_cons_num < other.all_cons_num {
                            return Ordering::Less;
                        } else {
                            return self.label_num.cmp(&other.label_num);
                        }
                    }
                }
            }
        }
        impl PartialOrd for BfsLayerNode {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(Ord::cmp(self, other))
            }
        }
        impl Eq for BfsLayerNode {}
        impl PartialEq for BfsLayerNode {
            fn eq(&self, other: &Self) -> bool {
                self.bfs_cons_num == other.bfs_cons_num
                    && self.all_cons_num == other.all_cons_num
                    && self.label_num == other.label_num
            }
        }

        #[inline]
        fn bfs_search_ordering<G>(
            g: G,
            node_idx: usize,
            order_idx: &mut usize,
            added: &mut Vec<bool>,
            matching_order: &mut Vec<usize>,
            node_con_num: &Vec<usize>,
            node_labels: &Vec<usize>,
            label_nums: &mut Vec<usize>,
        ) where
            G: IntoNeighborsUnirected + NodeIndexable + NodeCount,
        {
            let mut cur_order = *order_idx;
            let (mut start_of_layer, mut end_of_layer, mut last_added_pos) =
                (cur_order, cur_order, cur_order);
            let mut bfs_tree_node_cons = vec![0; g.node_count()];
            matching_order[cur_order] = node_idx;
            added[node_idx] = true;

            // add all current layer's nodes
            while cur_order <= last_added_pos {
                for order in start_of_layer..end_of_layer + 1 {
                    let cur_node_idx = matching_order[order];
                    for neighbor in g.neighbors_undirected(g.from_index(cur_node_idx)) {
                        let neighbor_id = g.to_index(neighbor);
                        bfs_tree_node_cons[neighbor_id] += 1;
                        if !added[neighbor_id] {
                            last_added_pos += 1;
                            added[neighbor_id] = true;
                            matching_order[last_added_pos] = neighbor_id;
                        }
                    }
                    cur_order += 1;
                }

                // reorder current layer based on ordering rule, use Proiority Queue is more efficient than
                // iterate with two for loops, but the same time, the third ordering condition may be not accurate,
                // it's trade-offs from performance aspect.
                let mut layer_ordering = BinaryHeap::new();
                for order in start_of_layer..end_of_layer + 1 {
                    let cur_node_idx = matching_order[order];
                    layer_ordering.push(BfsLayerNode {
                        node_idx: cur_node_idx,
                        bfs_cons_num: bfs_tree_node_cons[cur_node_idx],
                        all_cons_num: node_con_num[cur_node_idx],
                        label_num: label_nums[node_labels[cur_node_idx]],
                    });
                }
                let mut order_counter = start_of_layer;
                while let Some(layer_node) = layer_ordering.pop() {
                    matching_order[order_counter] = layer_node.node_idx;
                    label_nums[node_labels[layer_node.node_idx]] -= 1;
                    order_counter += 1;
                }

                start_of_layer = end_of_layer + 1;
                end_of_layer = last_added_pos;
            }

            *order_idx = cur_order;
        }

        matching_order
    }

    /// init the nums of Rnew and Rinout for every node in the g0
    fn init_r_new_inout<G>(
        g: G,
        order: &Vec<usize>,
        node_labels: &Vec<usize>,
        max_label: usize,
    ) -> (Vec<Vec<(usize, usize)>>, Vec<Vec<(usize, usize)>>)
    where
        G: NodeCount + NodeIndexable + IntoNeighborsUnirected,
    {
        // used to mark how many time a node has be visited, usize::MAX mean it already exist in the mapping
        let mut g0_node_cons = vec![0usize; g.node_count()];
        let mut r_new = vec![vec![]; g.node_count()];
        let mut r_in_out = vec![vec![]; g.node_count()];
        // `label_tmp` is used to record the label and num of same labels of current node's neighbor
        let mut label_in_out_tmp = vec![0; max_label + 1];
        let mut label_new_tmp = vec![0; max_label + 1];

        let max_order = order.len();
        let mut cur_order = 0;
        while max_order > cur_order {
            let cur_node_idx = order[cur_order];

            for neighbor in g.neighbors_undirected(g.from_index(cur_node_idx)) {
                let neighbor_id = g.to_index(neighbor);
                let neighbor_cons = g0_node_cons[neighbor_id];
                if neighbor_cons == G0_CON_IN_MAPPING || neighbor_id == cur_node_idx {
                    continue;
                }
                if neighbor_cons > 0 {
                    label_in_out_tmp[node_labels[neighbor_id]] += 1;
                } else if neighbor_cons == 0 {
                    label_new_tmp[node_labels[neighbor_id]] += 1;
                }
            }

            for neighbor in g.neighbors_undirected(g.from_index(cur_node_idx)) {
                let neighbor_idx = g.to_index(neighbor);
                let neighbor_label = node_labels[neighbor_idx];
                if label_in_out_tmp[neighbor_label] > 0 {
                    r_in_out[cur_node_idx].push((neighbor_label, label_in_out_tmp[neighbor_label]));
                    label_in_out_tmp[neighbor_label] = 0;
                }
                if label_new_tmp[neighbor_label] > 0 {
                    r_new[cur_node_idx].push((neighbor_label, label_new_tmp[neighbor_label]));
                    label_new_tmp[neighbor_label] = 0;
                }

                // mark the unvisited neighbor nodes of current node as visited
                if g0_node_cons[neighbor_idx] != G0_CON_IN_MAPPING && neighbor_idx != cur_node_idx {
                    g0_node_cons[neighbor_idx] += 1;
                }
            }

            g0_node_cons[cur_node_idx] = G0_CON_IN_MAPPING;
            cur_order += 1;
        }

        (r_new, r_in_out)
    }
}

pub struct Vf2ppMatcherBuilder<G0, G1, F0, F1, NM, EM> {
    node_matcher: NM,
    edge_matcher: EM,
    label: (F0, F1),
    subgraph: bool,
    phantomdata: PhantomData<(G0, G1)>,
}

impl<G0, G1>
    Vf2ppMatcherBuilder<G0, G1, NoNodeLabel, NoNodeLabel, NoSemanticMatch, NoSemanticMatch>
{
    pub fn new() -> Self
    where
        G0: GraphBase,
        G1: GraphBase,
    {
        Vf2ppMatcherBuilder {
            node_matcher: NoSemanticMatch,
            edge_matcher: NoSemanticMatch,
            label: (NoNodeLabel, NoNodeLabel),
            subgraph: false,
            phantomdata: PhantomData,
        }
    }
}

impl<G0, G1, F0, F1, NM, EM> Vf2ppMatcherBuilder<G0, G1, F0, F1, NM, EM>
where
    G0: GraphBase,
    G1: GraphBase,
    F0: NodeLabel<G0>,
    F1: NodeLabel<G1>,
    NM: NodeMatcher<G0, G1>,
    EM: EdgeMatcher<G0, G1>,
{
    pub fn set_subgraph(mut self, match_subgraph: bool) -> Self {
        self.subgraph = match_subgraph;
        self
    }
}

impl<G0, G1, F0, F1, NM, EM> Vf2ppMatcherBuilder<G0, G1, F0, F1, NM, EM>
where
    G0: IntoNeighborsDirected
        + IntoNeighborsUnirected
        + IntoNodeIdentifiers
        + NodeIndexable
        + NodeCount
        + GraphProp,
    G1: IntoNeighborsDirected
        + IntoNeighborsUnirected
        + IntoNodeIdentifiers
        + NodeIndexable
        + NodeCount
        + GraphProp,
    F0: NodeLabel<G0>,
    F1: NodeLabel<G1>,
    NM: NodeMatcher<G0, G1>,
    EM: EdgeMatcher<G0, G1>,
{
    pub fn build<'a>(self, g0: G0, g1: G1) -> Vf2ppIsomorphismMatcher<'a, G0, G1, NM, EM> {
        let vf2pp = VF2PP::new(g0, g1, self.label.0, self.label.1);

        Vf2ppIsomorphismMatcher::new(vf2pp, self.node_matcher, self.edge_matcher, self.subgraph)
    }
}

impl<G0, G1, F0, F1, NM, EM> Vf2ppMatcherBuilder<G0, G1, F0, F1, NM, EM>
where
    G0: GraphBase + DataMap,
    G1: GraphBase + DataMap,
    F0: NodeLabel<G0>,
    F1: NodeLabel<G1>,
    NM: NodeMatcher<G0, G1>,
    EM: EdgeMatcher<G0, G1>,
{
    pub fn set_label<LF0, LF1>(
        self,
        label: (LF0, LF1),
    ) -> Vf2ppMatcherBuilder<G0, G1, LF0, LF1, NM, EM>
    where
        LF0: FnMut(G0::NodeWeight) -> usize,
        LF1: FnMut(G1::NodeWeight) -> usize,
    {
        Vf2ppMatcherBuilder {
            node_matcher: self.node_matcher,
            edge_matcher: self.edge_matcher,
            label,
            subgraph: self.subgraph,
            phantomdata: PhantomData,
        }
    }
}

impl<'a, 'b, G0, G1, F0, F1, NM, EM> Vf2ppMatcherBuilder<G0, G1, F0, F1, NM, EM>
where
    G0: DataMap + IntoEdgesDirected,
    G1: DataMap + IntoEdgesDirected,
    F0: NodeLabel<G0>,
    F1: NodeLabel<G1>,
    NM: NodeMatcher<G0, G1>,
    EM: EdgeMatcher<G0, G1>,
{
    pub fn set_node_matcher<N>(self, nm: N) -> Vf2ppMatcherBuilder<G0, G1, F0, F1, N, EM>
    where
        N: FnMut(&G0::NodeWeight, &G1::NodeWeight) -> bool,
    {
        Vf2ppMatcherBuilder {
            node_matcher: nm,
            edge_matcher: self.edge_matcher,
            label: self.label,
            subgraph: self.subgraph,
            phantomdata: PhantomData,
        }
    }
}

impl<'a, 'b, G0, G1, F0, F1, NM, EM> Vf2ppMatcherBuilder<G0, G1, F0, F1, NM, EM>
where
    G0: DataMap + IntoEdgesDirected + Copy,
    G1: DataMap + IntoEdgesDirected + Copy,
    F0: NodeLabel<G0>,
    F1: NodeLabel<G1>,
    NM: NodeMatcher<G0, G1>,
    EM: EdgeMatcher<G0, G1>,
{
    pub fn set_edge_matcher<E>(self, em: E) -> Vf2ppMatcherBuilder<G0, G1, F0, F1, NM, E>
    where
        E: FnMut(&G0::EdgeWeight, &G1::EdgeWeight) -> bool,
    {
        Vf2ppMatcherBuilder {
            node_matcher: self.node_matcher,
            edge_matcher: em,
            label: self.label,
            subgraph: self.subgraph,
            phantomdata: PhantomData,
        }
    }
}

/// match according the ordering of g0, this should be a pure function so that it could be reused
fn isomorphism_match<'a, G0, G1, NM, EM>(
    g0: G0,
    g1: G1,
    roots: &Vec<bool>,
    order: &Vec<usize>,
    mapping: &mut Vec<usize>,
    r_in_out: &Vec<Vec<(usize, usize)>>,
    r_new: &Vec<Vec<(usize, usize)>>,
    g0_node_labels: &Vec<usize>,
    g1_node_labels: &Vec<usize>,
    g1_node_cons: &mut Vec<usize>,
    g1_candidate_nodes_iter: &mut Vec<Box<dyn Iterator<Item = G1::NodeId> + 'a>>,
    depth: &mut usize,
    max_label: usize,
    node_matcher: &mut NM,
    edge_matcher: &mut EM,
    subgraph: bool,
) -> Option<Vec<usize>>
where
    G0: IntoNeighborsDirected + IntoNeighborsUnirected + NodeIndexable + NodeCount + GraphProp,
    G1: IntoNeighborsDirected
        + IntoNeighborsUnirected
        + NodeIndexable
        + IntoNodeIdentifiers
        + NodeCount
        + GraphProp
        + 'a,
    NM: NodeMatcher<G0, G1>,
    EM: EdgeMatcher<G0, G1>,
{
    if !subgraph && g0.node_count() != g1.node_count() || g0.node_count() > g1.node_count() {
        return None;
    }
    'outer: while *depth != usize::MAX {
        let cur_depth = *depth;

        // a matched resutl found
        if cur_depth == order.len() {
            let result = mapping.iter().map(|v| v.clone()).collect();
            if cur_depth > 0 {
                unmark_node_pair(
                    depth,
                    order,
                    mapping,
                    g1_candidate_nodes_iter,
                    g1_node_cons,
                    g1,
                    true,
                );
            } else {
                *depth = usize::MAX;
            }
            return Some(result);
        }

        let n = order[cur_depth];
        let is_root = roots[n];
        // There are two possible cases:
        // 1. A fresh candidate in this depth
        // 2. A candidate from `g1_candidates_nodes_iter`, and need to forward with previous candidate iter
        if cur_depth >= g1_candidate_nodes_iter.len() {
            debug_assert!(cur_depth == g1_candidate_nodes_iter.len());
            // it's a fresh start, we first need to check the neighbor of n
            if is_root {
                g1_candidate_nodes_iter.push(Box::new(g1.node_identifiers()));
            } else {
                let mut has_neighbor = false;
                'neighbor: for d in [Outgoing, Incoming] {
                    if !g0.is_directed() && d == Incoming {
                        break;
                    }
                    for neighbor in g0.neighbors_directed(g0.from_index(n), d) {
                        let neighbor_mapping_in_g1 = mapping[g0.to_index(neighbor)];
                        if neighbor_mapping_in_g1 != NOT_IN_MAPPING {
                            let reverse_dir = if d == Outgoing { Incoming } else { Outgoing };
                            let iter = g1.neighbors_directed(
                                g1.from_index(neighbor_mapping_in_g1),
                                reverse_dir,
                            );
                            let mut visited: Vec<usize> = Vec::new();

                            g1_candidate_nodes_iter.push(Box::new(iter.filter(move |&node_id| {
                                let node_idx = g1.to_index(node_id);
                                if !visited.contains(&node_idx) {
                                    visited.push(node_idx);
                                    true
                                } else {
                                    false
                                }
                            })));
                            has_neighbor = true;
                            break 'neighbor;
                        }
                    }
                }

                debug_assert!(has_neighbor);
            }
        }

        let cur_g1_iter = g1_candidate_nodes_iter.last_mut().unwrap();
        while let Some(m_id) = cur_g1_iter.next() {
            let m = g1.to_index(m_id);
            let m_cons = g1_node_cons[m];
            if m_cons != G1_CON_IN_MAPPING
                && (is_root && m_cons == 0 || !is_root && m_cons != 0)
                && check_feasibility(
                    g0.from_index(n),
                    g1.from_index(m),
                    g0,
                    g1,
                    mapping,
                    g1_node_cons,
                    r_in_out,
                    r_new,
                    g0_node_labels,
                    g1_node_labels,
                    max_label,
                    node_matcher,
                    edge_matcher,
                    subgraph,
                )
            {
                mark_node_pair(n, m, depth, mapping, g1_node_cons, g1);
                continue 'outer;
            }
        }
        // no matching found, go backward
        unmark_node_pair(
            depth,
            order,
            mapping,
            g1_candidate_nodes_iter,
            g1_node_cons,
            g1,
            false,
        );
    }

    None
}

#[inline]
fn mark_node_pair<G>(
    n: usize,
    m: usize,
    depth: &mut usize,
    mapping: &mut Vec<usize>,
    g1_node_cons: &mut Vec<usize>,
    g1: G,
) where
    G: IntoNeighborsUnirected + NodeIndexable,
{
    *depth += 1;
    mapping[n] = m;
    // println!("mark node pair: {} {}", n, m);
    g1_node_cons[m] = G1_CON_IN_MAPPING;
    for neighbor in g1.neighbors_undirected(g1.from_index(m)) {
        let neighbor_idx = g1.to_index(neighbor);
        if g1_node_cons[neighbor_idx] != G1_CON_IN_MAPPING {
            g1_node_cons[neighbor_idx] += 1;
        }
    }
}

#[inline]
fn unmark_node_pair<'a, G>(
    depth: &mut usize,
    matching_order: &Vec<usize>,
    mapping: &mut Vec<usize>,
    g1_candidate_nodes_iter: &mut Vec<Box<dyn Iterator<Item = G::NodeId> + 'a>>,
    g1_node_cons: &mut Vec<usize>,
    g1: G,
    is_last: bool,
) where
    G: IntoNeighborsUnirected + NodeIndexable,
{
    if *depth == 0 {
        *depth = usize::MAX;
        return;
    } else {
        *depth -= 1;
    }
    let n = matching_order[*depth];
    let m = mapping[n];
    // println!("unmark node pair: {} {}", n, m);
    mapping[n] = NOT_IN_MAPPING;
    g1_node_cons[m] = 0;
    for neighbor in g1.neighbors_undirected(g1.from_index(m)) {
        let neighbor_id = g1.to_index(neighbor);
        let neighbor_con = g1_node_cons[neighbor_id];
        if neighbor_con != G1_CON_IN_MAPPING && neighbor_id != m {
            g1_node_cons[neighbor_id] -= 1;
        } else if neighbor_con == G1_CON_IN_MAPPING {
            g1_node_cons[m] += 1;
        }
    }
    if !is_last {
        g1_candidate_nodes_iter.pop();
    }
}

/// check the feasiblity of candidate node pair, also cut labels in this process, the whole process
/// is basically the same with VF2 feasibility rule.
fn check_feasibility<G0, G1, NM, EM>(
    n: G0::NodeId,
    m: G1::NodeId,
    g0: G0,
    g1: G1,
    mapping: &Vec<usize>,
    g1_node_cons: &Vec<usize>,
    r_in_out: &Vec<Vec<(usize, usize)>>,
    r_new: &Vec<Vec<(usize, usize)>>,
    g0_node_labels: &Vec<usize>,
    g1_node_labels: &Vec<usize>,
    max_label: usize,
    node_matcher: &mut NM,
    edge_matcher: &mut EM,
    subgraph: bool,
) -> bool
where
    NM: NodeMatcher<G0, G1>,
    EM: EdgeMatcher<G0, G1>,
    G0: IntoNeighborsDirected + IntoNeighborsUnirected + NodeIndexable + GraphProp,
    G1: IntoNeighborsDirected + IntoNeighborsUnirected + NodeIndexable + NodeCount + GraphProp,
{
    // PreCondition, check if the node label of candidate node pair is consistent
    let n_idx = g0.to_index(n);
    let m_idx = g1.to_index(m);
    if g0_node_labels[n_idx] != g1_node_labels[m_idx] {
        return false;
    }

    if NM::enabled() && !node_matcher.eq(&g0, &g1, n, m) {
        return false;
    }

    let edge_matcher_enabled = EM::enabled();

    // First, we need to check all neighbors of n which is in the mapping, and ensure all of
    // them exist in m, and check the number of connections for same neighbor in n is less than m.
    let mut g1_mapping_out_tmp = vec![0usize; g1.node_count()];
    let mut g1_mapping_in_tmp = vec![0usize; g1.node_count()];
    let mut g1_mapping_self = 0usize;

    // check all neighbors of n which is in the Rinout and Rnew, and ensure the number of
    // each neighbor is less than m.
    let mut label_in_out_tmp = vec![0usize; max_label + 1];
    let mut label_new_tmp = vec![0usize; max_label + 1];

    for d in [Outgoing, Incoming] {
        if !g1.is_directed() && d == Incoming {
            break;
        }
        let g1_mapping = if d == Outgoing {
            &mut g1_mapping_out_tmp
        } else {
            &mut g1_mapping_in_tmp
        };
        for neighbor in g1.neighbors_directed(m, d) {
            let neighbor_id = g1.to_index(neighbor);
            let neighbor_con = g1_node_cons[neighbor_id];
            if neighbor_con == G1_CON_IN_MAPPING {
                g1_mapping[neighbor_id] += 1;
            } else if neighbor_id == m_idx {
                g1_mapping_self += 1;
            } else if neighbor_con > 0 {
                label_in_out_tmp[g1_node_labels[neighbor_id]] += 1;
            } else {
                label_new_tmp[g1_node_labels[neighbor_id]] += 1;
            }
        }
    }

    // check all mapping in g0
    for d in [Outgoing, Incoming] {
        if !g0.is_directed() && d == Incoming {
            break;
        }
        let g1_mapping = if d == Outgoing {
            &mut g1_mapping_out_tmp
        } else {
            &mut g1_mapping_in_tmp
        };
        for neighbor in g0.neighbors_directed(n, d) {
            let neighbor_id = g0.to_index(neighbor);
            let mapping_neighbor_id = mapping[neighbor_id];
            if mapping_neighbor_id != NOT_IN_MAPPING {
                if g1_mapping[mapping_neighbor_id] == 0 {
                    return false;
                } else {
                    g1_mapping[mapping_neighbor_id] -= 1;
                    if edge_matcher_enabled {
                        let g0_nodes = if d == Outgoing {
                            (n, neighbor)
                        } else {
                            (neighbor, n)
                        };
                        let g1_nodes = if d == Outgoing {
                            (m, g1.from_index(mapping_neighbor_id))
                        } else {
                            (g1.from_index(mapping_neighbor_id), m)
                        };
                        if !edge_matcher.eq(&g0, &g1, g0_nodes, g1_nodes) {
                            return false;
                        }
                    }
                }
            } else if neighbor_id == n_idx {
                if g1_mapping_self == 0
                    || edge_matcher_enabled && !edge_matcher.eq(&g0, &g1, (n, n), (m, m))
                {
                    return false;
                }
                g1_mapping_self -= 1;
            }
        }
    }

    // for induced subgraph or isomorphism, the mapping must match exactly
    if g1_mapping_self != 0 {
        return false;
    }

    // check Rinout and Rnew node labels
    for i in [0, 1] {
        let r = if i == 0 { r_in_out } else { r_new };
        let label_tmp = if i == 0 {
            &mut label_in_out_tmp
        } else {
            &mut label_new_tmp
        };

        for (label, label_num) in r[n_idx].iter() {
            match label_tmp[*label].cmp(label_num) {
                Ordering::Less => {
                    return false;
                }
                Ordering::Greater => {
                    if !subgraph {
                        return false;
                    }
                    label_tmp[*label] -= *label_num;
                }
                _ => {
                    label_tmp[*label] = 0;
                }
            }
        }
    }

    // check label and neighbor mapping completeness
    for d in [Outgoing, Incoming] {
        if !g1.is_directed() && d == Incoming {
            break;
        }
        let g1_mapping = if d == Outgoing {
            &mut g1_mapping_out_tmp
        } else {
            &mut g1_mapping_in_tmp
        };
        for neighbor in g1.neighbors_directed(m, d) {
            let neighbor_id = g1.to_index(neighbor);
            let neighbor_con = g1_node_cons[neighbor_id];
            if neighbor_con == G1_CON_IN_MAPPING && g1_mapping[neighbor_id] != 0 {
                return false;
            } else if neighbor_con > 0
                && !subgraph
                && label_in_out_tmp[g1_node_labels[neighbor_id]] > 0
            {
                return false;
            } else if neighbor_con == 0
                && !subgraph
                && label_new_tmp[g1_node_labels[neighbor_id]] > 0
            {
                return false;
            }
        }
    }

    true
}

/// without label or semantice matching
pub fn vf2pp_is_isomorphism_matching<G0, G1>(g0: G0, g1: G1, subgraph: bool) -> bool
where
    G0: IntoNeighborsDirected
        + IntoNeighborsUnirected
        + IntoNodeIdentifiers
        + NodeIndexable
        + NodeCount
        + GraphProp,
    G1: IntoNeighborsDirected
        + IntoNeighborsUnirected
        + IntoNodeIdentifiers
        + NodeIndexable
        + NodeCount
        + GraphProp,
{
    Vf2ppMatcherBuilder::new()
        .set_subgraph(subgraph)
        .build(g0, g1)
        .is_match()
}

pub fn vf2pp_is_isomorphism_semantic_matching<G0, G1, NM, EM>(
    g0: G0,
    g1: G1,
    node_matcher: NM,
    edge_matcher: EM,
    subgraph: bool,
) -> bool
where
    G0: IntoNeighborsDirected
        + IntoNeighborsUnirected
        + IntoNodeIdentifiers
        + NodeIndexable
        + NodeCount
        + GraphProp
        + DataMap
        + IntoEdgesDirected
        + Copy,
    G1: IntoNeighborsDirected
        + IntoNeighborsUnirected
        + IntoNodeIdentifiers
        + NodeIndexable
        + NodeCount
        + GraphProp
        + DataMap
        + IntoEdgesDirected
        + Copy,
    NM: FnMut(&G0::NodeWeight, &G1::NodeWeight) -> bool,
    EM: FnMut(&G0::EdgeWeight, &G1::EdgeWeight) -> bool,
{
    Vf2ppMatcherBuilder::new()
        .set_subgraph(subgraph)
        .set_node_matcher(node_matcher)
        .set_edge_matcher(edge_matcher)
        .build(g0, g1)
        .is_match()
}

pub fn vf2pp_isomorphism_semantic_matching_iter<'a, G0, G1, NM, EM>(
    g0: G0,
    g1: G1,
    node_matcher: NM,
    edge_matcher: EM,
    subgraph: bool,
) -> Vf2ppIsomorphismMatcher<'a, G0, G1, NM, EM>
where
    G0: IntoNeighborsDirected
        + IntoNeighborsUnirected
        + IntoNodeIdentifiers
        + NodeIndexable
        + NodeCount
        + GraphProp
        + DataMap
        + IntoEdgesDirected
        + Copy,
    G1: IntoNeighborsDirected
        + IntoNeighborsUnirected
        + IntoNodeIdentifiers
        + NodeIndexable
        + NodeCount
        + GraphProp
        + DataMap
        + IntoEdgesDirected
        + Copy
        + 'a,
    NM: FnMut(&G0::NodeWeight, &G1::NodeWeight) -> bool,
    EM: FnMut(&G0::EdgeWeight, &G1::EdgeWeight) -> bool,
{
    Vf2ppMatcherBuilder::new()
        .set_subgraph(subgraph)
        .set_node_matcher(node_matcher)
        .set_edge_matcher(edge_matcher)
        .build(g0, g1)
}
