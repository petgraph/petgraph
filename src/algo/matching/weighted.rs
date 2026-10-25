// Adapted from NetworkX's max_weight_matching, implementing Edmonds' algorithm
// as described by Zvi Galil, ACM Computing Surveys 18(1), 1986.
//
// Copyright (c) 2004-2026, NetworkX Developers
// Aric Hagberg <hagberg@lanl.gov>
// Dan Schult <dschult@colgate.edu>
// Pieter Swart <swart@lanl.gov>
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are met:
// * Redistributions of source code must retain the above copyright notice, this list of conditions
//   and the following disclaimer.
// * Redistributions in binary form must reproduce the above copyright notice, this list of
//   conditions and the following disclaimer in the documentation and/or other materials provided
//   with the distribution.
// * Neither the name of the NetworkX Developers nor the names of its contributors may be used to
//   endorse or promote products derived from this software without specific prior written
//   permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
// AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
// IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
// ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR CONTRIBUTORS BE
// LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
// CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
// SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
// INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
// CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
// ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
// POSSIBILITY OF SUCH DAMAGE.

use alloc::{vec, vec::Vec};

use hashbrown::HashMap;

use super::Matching;
use crate::{
    visit::{EdgeRef, GraphProp, IntoEdgeReferences, IntoNodeIdentifiers, NodeIndexable},
    Undirected,
};

/// Compute an exact maximum-weight matching of an undirected graph.
///
/// With `max_cardinality = false`, maximize the sum of edge weights; leaving
/// vertices unmatched is allowed, including returning an empty matching when
/// all edges have negative weights. With `max_cardinality = true`, first
/// maximize the number of matched edges, then maximize their total weight.
///
/// This is Edmonds' primal-dual blossom algorithm, following
/// [Galil (1986)](https://doi.org/10.1145/6462.6502). Odd alternating cycles
/// are contracted and expanded as the primal matching and dual solution evolve.
/// Internal arithmetic uses `i128`, so signed `i64` weights are processed
/// exactly, without floating-point tolerances. The callback is evaluated once
/// per non-loop edge. Self-loops are ignored. For parallel edges, the largest
/// weight is used; as with [`Matching`], the output identifies endpoint pairs.
///
/// Ties are resolved deterministically for a fixed node and edge iteration order.
/// Graphs with deleted node indices are supported. Blossom operations use explicit
/// stacks, so nesting does not consume the call stack.
///
/// # Complexity
/// Time **O(|V|³ + |E|)**, auxiliary space **O(B + |V| + |E|)**, where `B`
/// is the graph's node index bound. The edge bound includes parallel edges.
///
/// # Example
/// ```
/// use petgraph::{algo::maximum_weight_matching, graph::UnGraph};
/// let g = UnGraph::<(), i64>::from_edges([(0, 1, 5), (1, 2, 100), (2, 3, 5)]);
/// let m = maximum_weight_matching(&g, |e| *e.weight(), false);
/// assert_eq!(m.len(), 1);
/// assert!(m.contains_edge(1.into(), 2.into()));
/// let m = maximum_weight_matching(&g, |e| *e.weight(), true);
/// assert_eq!(m.len(), 2);
/// ```
pub fn maximum_weight_matching<G, F>(
    graph: G,
    mut edge_weight: F,
    max_cardinality: bool,
) -> Matching<G>
where
    G: IntoNodeIdentifiers + IntoEdgeReferences + NodeIndexable + GraphProp<EdgeType = Undirected>,
    F: FnMut(G::EdgeRef) -> i64,
{
    let nodes: Vec<_> = graph.node_identifiers().collect();
    let mut dense = vec![usize::MAX; graph.node_bound()];
    for (i, &v) in nodes.iter().enumerate() {
        dense[graph.to_index(v)] = i;
    }
    let mut edges: Vec<WeightedEdge> = Vec::new();
    let mut pairs = HashMap::new();
    for edge in graph.edge_references() {
        let u = dense[graph.to_index(edge.source())];
        let v = dense[graph.to_index(edge.target())];
        if u == v {
            continue;
        }
        let weight = i128::from(edge_weight(edge));
        let pair = (u.min(v), u.max(v));
        if let Some(&index) = pairs.get(&pair) {
            let previous: &mut WeightedEdge = &mut edges[index];
            previous.weight = previous.weight.max(weight);
        } else {
            pairs.insert(pair, edges.len());
            edges.push(WeightedEdge { u, v, weight });
        }
    }
    drop(pairs);
    drop(dense);
    let mut solver = Solver::new(nodes.len(), edges);
    solver.solve(max_cardinality);
    let mut mates = vec![None; graph.node_bound()];
    let mut n_matched = 0;
    for (i, mate) in solver.mate.iter().enumerate() {
        if let Some(link) = mate {
            mates[graph.to_index(nodes[i])] = Some(nodes[link.to]);
            n_matched += 1;
        }
    }
    Matching::new(graph, mates, n_matched / 2)
}

#[derive(Clone, Copy, Debug)]
struct Link {
    from: usize,
    to: usize,
    edge: usize,
}

impl Link {
    fn reversed(self) -> Self {
        Self {
            from: self.to,
            to: self.from,
            edge: self.edge,
        }
    }
}

struct WeightedEdge {
    u: usize,
    v: usize,
    weight: i128,
}

struct Blossom {
    // Odd cyclic sequence, starting with the child containing the base.
    children: Vec<usize>,
    // edges[i] goes from children[i] to children[(i + 1) % len].
    edges: Vec<Link>,
    // One least-slack edge to each neighboring outer blossom. Consumed on contraction.
    best_edges: Option<Vec<Link>>,
}

// Vertices occupy [0, n); nontrivial blossoms use recyclable slots [n, 2n).
// Only top-level blossom labels are used for dual updates. Vertex labels inside
// inner blossoms additionally record entry points needed during expansion.
struct Solver {
    n: usize,
    edges: Vec<WeightedEdge>,
    adjacent: Vec<Vec<usize>>,
    mate: Vec<Option<Link>>,
    inside: Vec<usize>,
    parent: Vec<Option<usize>>,
    base: Vec<usize>,
    blossoms: Vec<Option<Blossom>>,
    free: Vec<usize>,
    label: Vec<u8>, // 0: unreached, 1: outer (S), 2: inner (T), 5: breadcrumb
    label_edge: Vec<Option<Link>>,
    best: Vec<Option<Link>>,
    dual: Vec<i128>, // twice the vertex dual variable
    blossom_dual: Vec<i128>,
    allowed: Vec<bool>,
    queue: Vec<usize>,
    #[cfg(test)]
    statistics: Statistics,
}

#[cfg(test)]
#[derive(Default, Debug)]
struct Statistics {
    contractions: usize,
    nested_contractions: usize,
    expansions: usize,
    inner_expansions: usize,
}

enum AugmentTask {
    Enter(usize, usize),
    Match(Link),
    Rotate(usize, usize, usize),
}

#[derive(Clone, Copy)]
enum Event {
    Stop,
    Edge(Link),
    Expand(usize),
}

impl Solver {
    fn new(n: usize, edges: Vec<WeightedEdge>) -> Self {
        let mut adjacent = vec![Vec::new(); n];
        let mut max_weight = 0;
        for (i, e) in edges.iter().enumerate() {
            adjacent[e.u].push(i);
            adjacent[e.v].push(i);
            max_weight = max_weight.max(e.weight);
        }
        Self {
            n,
            allowed: vec![false; edges.len()],
            edges,
            adjacent,
            mate: vec![None; n],
            inside: (0..n).collect(),
            parent: vec![None; 2 * n],
            base: (0..2 * n).collect(),
            blossoms: (0..2 * n).map(|_| None).collect(),
            free: (n..2 * n).rev().collect(),
            label: vec![0; 2 * n],
            label_edge: vec![None; 2 * n],
            best: vec![None; 2 * n],
            dual: vec![max_weight; n],
            blossom_dual: vec![0; 2 * n],
            queue: Vec::new(),
            #[cfg(test)]
            statistics: Statistics::default(),
        }
    }

    fn link(&self, edge: usize, from: usize) -> Link {
        let e = &self.edges[edge];
        Link {
            from,
            to: if e.u == from { e.v } else { e.u },
            edge,
        }
    }

    fn slack(&self, link: Link) -> i128 {
        self.dual[link.from] + self.dual[link.to] - 2 * self.edges[link.edge].weight
    }

    fn leaves(&self, b: usize) -> Vec<usize> {
        let mut leaves = Vec::new();
        let mut stack = vec![b];
        while let Some(t) = stack.pop() {
            if t < self.n {
                leaves.push(t);
            } else {
                stack.extend(self.blossoms[t].as_ref().unwrap().children.iter().rev());
            }
        }
        leaves
    }

    fn assign_label(&mut self, mut w: usize, mut label: u8, mut via: Option<Link>) {
        loop {
            let b = self.inside[w];
            debug_assert_eq!(self.label[w], 0);
            debug_assert_eq!(self.label[b], 0);
            self.label[w] = label;
            self.label[b] = label;
            self.label_edge[w] = via;
            self.label_edge[b] = via;
            self.best[w] = None;
            self.best[b] = None;
            if label == 1 {
                self.queue.extend(self.leaves(b));
                break;
            }
            // Only the base of an inner blossom has an external matched edge.
            let matched = self.mate[self.base[b]].unwrap();
            w = matched.to;
            label = 1;
            via = Some(matched);
        }
    }

    // Find a common base of two alternating paths; no common base means augmentation.
    fn scan_blossom(&mut self, v: usize, w: usize) -> Option<usize> {
        let (mut v, mut w) = (Some(v), Some(w));
        let mut path = Vec::new();
        let mut base = None;
        while let Some(vertex) = v {
            let b = self.inside[vertex];
            if self.label[b] == 5 {
                base = Some(self.base[b]);
                break;
            }
            debug_assert_eq!(self.label[b], 1);
            path.push(b);
            self.label[b] = 5;
            v = self.label_edge[b].map(|edge| {
                let inner = self.inside[edge.from];
                debug_assert_eq!(self.label[inner], 2);
                self.label_edge[inner].unwrap().from
            });
            if w.is_some() {
                core::mem::swap(&mut v, &mut w);
            }
        }
        for b in path {
            self.label[b] = 1;
        }
        base
    }

    fn add_blossom(&mut self, base: usize, link: Link) {
        let bb = self.inside[base];
        let (mut v, mut w) = (link.from, link.to);
        let (mut bv, mut bw) = (self.inside[v], self.inside[w]);
        let b = self.free.pop().unwrap();
        self.base[b] = base;
        self.parent[b] = None;
        self.parent[bb] = Some(b);
        let mut children = Vec::new();
        let mut edges = vec![link];
        while bv != bb {
            self.parent[bv] = Some(b);
            children.push(bv);
            let back = self.label_edge[bv].unwrap();
            edges.push(back);
            v = back.from;
            bv = self.inside[v];
        }
        children.push(bb);
        children.reverse();
        edges.reverse();
        while bw != bb {
            self.parent[bw] = Some(b);
            children.push(bw);
            let back = self.label_edge[bw].unwrap();
            edges.push(back.reversed());
            w = back.from;
            bw = self.inside[w];
        }
        debug_assert_eq!(children.len() % 2, 1);
        #[cfg(test)]
        {
            self.statistics.contractions += 1;
            self.statistics.nested_contractions +=
                usize::from(children.iter().any(|&c| c >= self.n));
        }
        self.blossoms[b] = Some(Blossom {
            children,
            edges,
            best_edges: None,
        });
        self.label[b] = 1;
        self.label_edge[b] = self.label_edge[bb];
        self.blossom_dual[b] = 0;
        for v in self.leaves(b) {
            if self.label[self.inside[v]] == 2 {
                self.queue.push(v);
            }
            self.inside[v] = b;
        }

        // Merge cached least-slack edges. Without this cache, repeatedly scanning
        // every leaf's adjacency on nested contractions would lose the cubic bound.
        let children = self.blossoms[b].as_ref().unwrap().children.clone();
        let mut best_to = vec![None; 2 * self.n];
        for child in children {
            let cached = self.blossoms[child]
                .as_mut()
                .and_then(|c| c.best_edges.take());
            let candidates = cached.unwrap_or_else(|| {
                let mut candidates = Vec::new();
                for v in self.leaves(child) {
                    candidates.extend(self.adjacent[v].iter().map(|&e| self.link(e, v)));
                }
                candidates
            });
            for mut edge in candidates {
                if self.inside[edge.to] == b {
                    edge = edge.reversed();
                }
                let neighbor = self.inside[edge.to];
                if neighbor != b && self.label[neighbor] == 1 {
                    self.keep_best(&mut best_to[neighbor], edge);
                }
            }
            self.best[child] = None;
        }
        let candidates: Vec<_> = best_to.into_iter().flatten().collect();
        let mut best = None;
        for &edge in &candidates {
            self.keep_best(&mut best, edge);
        }
        self.best[b] = best;
        self.blossoms[b].as_mut().unwrap().best_edges = Some(candidates);
    }

    fn keep_best(&self, best: &mut Option<Link>, candidate: Link) {
        if best.map_or(true, |old| self.slack(candidate) < self.slack(old)) {
            *best = Some(candidate);
        }
    }

    fn cyclic(index: isize, len: usize) -> usize {
        index.rem_euclid(len as isize) as usize
    }

    fn expand_blossom(&mut self, b: usize, end_stage: bool) {
        let mut stack = vec![b];
        while let Some(b) = stack.pop() {
            #[cfg(test)]
            {
                self.statistics.expansions += 1;
                self.statistics.inner_expansions += usize::from(!end_stage);
            }
            let cycle = self.blossoms[b].take().unwrap();
            for &child in &cycle.children {
                self.parent[child] = None;
                if child >= self.n && end_stage && self.blossom_dual[child] == 0 {
                    stack.push(child);
                } else {
                    for v in self.leaves(child) {
                        self.inside[v] = child;
                    }
                }
            }
            if !end_stage && self.label[b] == 2 {
                let mut via = self.label_edge[b].unwrap();
                let entry = self.inside[via.to];
                let len = cycle.children.len();
                let mut j = cycle.children.iter().position(|&c| c == entry).unwrap() as isize;
                let step = if j % 2 == 1 {
                    j -= len as isize;
                    1
                } else {
                    -1
                };
                while j != 0 {
                    let edge = if step == 1 {
                        cycle.edges[Self::cyclic(j, len)]
                    } else {
                        cycle.edges[Self::cyclic(j - 1, len)].reversed()
                    };
                    self.label[via.to] = 0;
                    self.label[edge.to] = 0;
                    self.assign_label(via.to, 2, Some(via));
                    self.allowed[edge.edge] = true;
                    j += step;
                    via = if step == 1 {
                        cycle.edges[Self::cyclic(j, len)]
                    } else {
                        cycle.edges[Self::cyclic(j - 1, len)].reversed()
                    };
                    self.allowed[via.edge] = true;
                    j += step;
                }
                let child = cycle.children[Self::cyclic(j, len)];
                self.label[via.to] = 2;
                self.label[child] = 2;
                self.label_edge[via.to] = Some(via);
                self.label_edge[child] = Some(via);
                self.best[child] = None;
                j += step;
                while cycle.children[Self::cyclic(j, len)] != entry {
                    let child = cycle.children[Self::cyclic(j, len)];
                    if self.label[child] != 1 {
                        if let Some(v) =
                            self.leaves(child).into_iter().find(|&v| self.label[v] != 0)
                        {
                            debug_assert_eq!(self.label[v], 2);
                            let back = self.label_edge[v].unwrap();
                            self.label[v] = 0;
                            self.label[self.mate[self.base[child]].unwrap().to] = 0;
                            self.assign_label(v, 2, Some(back));
                        }
                    }
                    j += step;
                }
            }
            self.label[b] = 0;
            self.label_edge[b] = None;
            self.best[b] = None;
            self.parent[b] = None;
            self.blossom_dual[b] = 0;
            self.free.push(b);
        }
    }

    // Schedule continuations explicitly: a nested augmentation must finish
    // before its parent's matching edges and base rotation are changed.
    fn augment_blossom(&mut self, b: usize, v: usize) {
        let mut stack = vec![AugmentTask::Enter(b, v)];
        while let Some(task) = stack.pop() {
            match task {
                AugmentTask::Match(edge) => {
                    self.mate[edge.from] = Some(edge);
                    self.mate[edge.to] = Some(edge.reversed());
                }
                AugmentTask::Rotate(b, i, v) => {
                    let blossom = self.blossoms[b].as_mut().unwrap();
                    blossom.children.rotate_left(i);
                    blossom.edges.rotate_left(i);
                    self.base[b] = self.base[blossom.children[0]];
                    debug_assert_eq!(self.base[b], v);
                }
                AugmentTask::Enter(b, v) => {
                    let mut child = v;
                    while self.parent[child] != Some(b) {
                        child = self.parent[child].unwrap();
                    }
                    let blossom = self.blossoms[b].as_ref().unwrap();
                    let len = blossom.children.len();
                    let i = blossom.children.iter().position(|&c| c == child).unwrap();
                    let mut j = i as isize;
                    let step = if i % 2 == 1 {
                        j -= len as isize;
                        1
                    } else {
                        -1
                    };
                    let mut actions = Vec::new();
                    if child >= self.n {
                        actions.push(AugmentTask::Enter(child, v));
                    }
                    while j != 0 {
                        j += step;
                        let child = blossom.children[Self::cyclic(j, len)];
                        let edge = if step == 1 {
                            blossom.edges[Self::cyclic(j, len)]
                        } else {
                            blossom.edges[Self::cyclic(j - 1, len)].reversed()
                        };
                        if child >= self.n {
                            actions.push(AugmentTask::Enter(child, edge.from));
                        }
                        j += step;
                        let child = blossom.children[Self::cyclic(j, len)];
                        if child >= self.n {
                            actions.push(AugmentTask::Enter(child, edge.to));
                        }
                        actions.push(AugmentTask::Match(edge));
                    }
                    actions.push(AugmentTask::Rotate(b, i, v));
                    stack.extend(actions.into_iter().rev());
                }
            }
        }
    }

    fn augment_matching(&mut self, edge: Link) {
        for mut edge in [edge, edge.reversed()] {
            loop {
                let b = self.inside[edge.from];
                debug_assert_eq!(self.label[b], 1);
                if b >= self.n {
                    self.augment_blossom(b, edge.from);
                }
                self.mate[edge.from] = Some(edge);
                let back = match self.label_edge[b] {
                    Some(back) => back,
                    None => break,
                };
                let inner = self.inside[back.from];
                debug_assert_eq!(self.label[inner], 2);
                edge = self.label_edge[inner].unwrap();
                if inner >= self.n {
                    self.augment_blossom(inner, edge.to);
                }
                self.mate[edge.to] = Some(edge.reversed());
            }
        }
    }

    fn search(&mut self) -> bool {
        while let Some(v) = self.queue.pop() {
            debug_assert_eq!(self.label[self.inside[v]], 1);
            for i in 0..self.adjacent[v].len() {
                let edge = self.link(self.adjacent[v][i], v);
                let w = edge.to;
                let bv = self.inside[v];
                let bw = self.inside[w];
                if bv == bw {
                    continue;
                }
                let slack = self.slack(edge);
                if slack <= 0 {
                    self.allowed[edge.edge] = true;
                }
                if self.allowed[edge.edge] {
                    match self.label[bw] {
                        0 => self.assign_label(w, 2, Some(edge)),
                        1 => {
                            if let Some(base) = self.scan_blossom(v, w) {
                                self.add_blossom(base, edge);
                            } else {
                                self.augment_matching(edge);
                                return true;
                            }
                        }
                        _ if self.label[w] == 0 => {
                            self.label[w] = 2;
                            self.label_edge[w] = Some(edge);
                        }
                        _ => {}
                    }
                } else if self.label[bw] == 1 {
                    let mut best = self.best[bv];
                    self.keep_best(&mut best, edge);
                    self.best[bv] = best;
                } else if self.label[w] == 0 {
                    let mut best = self.best[w];
                    self.keep_best(&mut best, edge);
                    self.best[w] = best;
                }
            }
        }
        false
    }

    fn dual_event(&self, max_cardinality: bool) -> (i128, Event) {
        let mut choice = if max_cardinality {
            None
        } else {
            Some((*self.dual.iter().min().unwrap(), Event::Stop))
        };
        let mut consider = |delta: i128, event| {
            if choice.map_or(true, |(old, _)| delta < old) {
                choice = Some((delta, event));
            }
        };
        for v in 0..self.n {
            if self.label[self.inside[v]] == 0 {
                if let Some(edge) = self.best[v] {
                    consider(self.slack(edge), Event::Edge(edge));
                }
            }
        }
        for b in 0..2 * self.n {
            if self.parent[b].is_some() || (b >= self.n && self.blossoms[b].is_none()) {
                continue;
            }
            if self.label[b] == 1 {
                if let Some(edge) = self.best[b] {
                    let slack = self.slack(edge);
                    debug_assert_eq!(slack % 2, 0);
                    consider(slack / 2, Event::Edge(edge));
                }
            } else if b >= self.n && self.label[b] == 2 {
                consider(self.blossom_dual[b], Event::Expand(b));
            }
        }
        choice.unwrap_or_else(|| (0.max(*self.dual.iter().min().unwrap()), Event::Stop))
    }

    fn solve(&mut self, max_cardinality: bool) {
        if self.n == 0 {
            return;
        }
        loop {
            self.label.fill(0);
            self.label_edge.fill(None);
            self.best.fill(None);
            self.allowed.fill(false);
            self.queue.clear();
            for blossom in self.blossoms.iter_mut().flatten() {
                blossom.best_edges = None;
            }
            for v in 0..self.n {
                if self.mate[v].is_none() && self.label[self.inside[v]] == 0 {
                    self.assign_label(v, 1, None);
                }
            }
            let augmented = loop {
                if self.search() {
                    break true;
                }
                let (delta, event) = self.dual_event(max_cardinality);
                debug_assert!(delta >= 0);
                for v in 0..self.n {
                    match self.label[self.inside[v]] {
                        1 => self.dual[v] -= delta,
                        2 => self.dual[v] += delta,
                        _ => {}
                    }
                }
                for b in self.n..2 * self.n {
                    if self.blossoms[b].is_some() && self.parent[b].is_none() {
                        match self.label[b] {
                            1 => self.blossom_dual[b] += delta,
                            2 => self.blossom_dual[b] -= delta,
                            _ => {}
                        }
                    }
                }
                match event {
                    Event::Stop => break false,
                    Event::Edge(edge) => {
                        self.allowed[edge.edge] = true;
                        self.queue.push(edge.from);
                    }
                    Event::Expand(b) => self.expand_blossom(b, false),
                }
            };
            if !augmented {
                break;
            }
            for b in self.n..2 * self.n {
                if self.blossoms[b].is_some()
                    && self.parent[b].is_none()
                    && self.label[b] == 1
                    && self.blossom_dual[b] == 0
                {
                    self.expand_blossom(b, true);
                }
            }
        }
        #[cfg(debug_assertions)]
        self.verify_optimum(max_cardinality);
    }

    #[cfg(debug_assertions)]
    fn ancestors(&self, v: usize) -> Vec<usize> {
        let mut path = vec![v];
        while let Some(parent) = self.parent[*path.last().unwrap()] {
            path.push(parent);
        }
        path.reverse();
        path
    }

    // Check a dual certificate against original weights and the laminar forest,
    // independently of the search's best-edge caches and allowed-edge flags.
    #[cfg(debug_assertions)]
    fn verify_optimum(&self, max_cardinality: bool) {
        let offset = if max_cardinality {
            0.max(-self.dual.iter().min().unwrap())
        } else {
            0
        };
        for v in 0..self.n {
            let ancestors = self.ancestors(v);
            assert_eq!(*ancestors.first().unwrap(), self.inside[v]);
            assert!(self.dual[v] + offset >= 0);
            if let Some(edge) = self.mate[v] {
                assert_eq!(self.mate[edge.to].unwrap().to, v);
            } else {
                assert_eq!(self.dual[v] + offset, 0);
            }
        }
        for (i, edge) in self.edges.iter().enumerate() {
            let mut slack = self.slack(self.link(i, edge.u));
            for (a, b) in self
                .ancestors(edge.u)
                .into_iter()
                .zip(self.ancestors(edge.v))
            {
                if a != b {
                    break;
                }
                slack += 2 * self.blossom_dual[a];
            }
            assert!(
                slack >= 0,
                "negative dual slack on ({}, {})",
                edge.u,
                edge.v
            );
            if self.mate[edge.u].map_or(false, |m| m.to == edge.v) {
                assert_eq!(slack, 0);
            }
        }
        for b in self.n..2 * self.n {
            if let Some(blossom) = &self.blossoms[b] {
                assert!(self.blossom_dual[b] >= 0);
                assert_eq!(blossom.children.len() % 2, 1);
                assert_eq!(blossom.children.len(), blossom.edges.len());
                if self.blossom_dual[b] > 0 {
                    for edge in blossom.edges.iter().skip(1).step_by(2) {
                        assert_eq!(self.mate[edge.from].unwrap().to, edge.to);
                        assert_eq!(self.mate[edge.to].unwrap().to, edge.from);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Fixture = (&'static str, &'static [(usize, usize, i64)], i128);

    #[test]
    fn nested_contraction_relabeling_expansion_and_augmentation() {
        // Fixtures adapted from NetworkX test_matching (BSD license above).
        let cases: &[Fixture] = &[
            (
                "test_s_blossom",
                &[(0, 1, 8), (0, 2, 9), (1, 2, 10), (2, 3, 7)],
                15,
            ),
            (
                "test_s_t_blossom",
                &[
                    (0, 1, 9),
                    (0, 2, 8),
                    (1, 2, 10),
                    (0, 3, 5),
                    (3, 4, 4),
                    (0, 5, 3),
                ],
                17,
            ),
            (
                "test_nested_s_blossom",
                &[
                    (0, 1, 9),
                    (0, 2, 9),
                    (1, 2, 10),
                    (1, 3, 8),
                    (2, 4, 8),
                    (3, 4, 10),
                    (4, 5, 6),
                ],
                23,
            ),
            (
                "test_nested_s_blossom_relabel",
                &[
                    (0, 1, 10),
                    (0, 6, 10),
                    (1, 2, 12),
                    (2, 3, 20),
                    (2, 4, 20),
                    (3, 4, 25),
                    (4, 5, 10),
                    (5, 6, 10),
                    (6, 7, 8),
                ],
                48,
            ),
            (
                "test_nested_s_blossom_expand",
                &[
                    (0, 1, 8),
                    (0, 2, 8),
                    (1, 2, 10),
                    (1, 3, 12),
                    (2, 4, 12),
                    (3, 4, 14),
                    (3, 5, 12),
                    (4, 6, 12),
                    (5, 6, 14),
                    (6, 7, 12),
                ],
                44,
            ),
            (
                "test_s_blossom_relabel_expand",
                &[
                    (0, 1, 23),
                    (0, 4, 22),
                    (0, 5, 15),
                    (1, 2, 25),
                    (2, 3, 22),
                    (3, 4, 25),
                    (3, 7, 14),
                    (4, 6, 13),
                ],
                67,
            ),
            (
                "test_nested_s_blossom_relabel_expand",
                &[
                    (0, 1, 19),
                    (0, 2, 20),
                    (0, 7, 8),
                    (1, 2, 25),
                    (1, 3, 18),
                    (2, 4, 18),
                    (3, 4, 13),
                    (3, 6, 7),
                    (4, 5, 7),
                ],
                47,
            ),
            (
                "test_nasty_blossom1",
                &[
                    (0, 1, 45),
                    (0, 4, 45),
                    (1, 2, 50),
                    (2, 3, 45),
                    (3, 4, 50),
                    (0, 5, 30),
                    (2, 8, 35),
                    (3, 7, 35),
                    (4, 6, 26),
                    (8, 9, 5),
                ],
                146,
            ),
            (
                "test_nasty_blossom2",
                &[
                    (0, 1, 45),
                    (0, 4, 45),
                    (1, 2, 50),
                    (2, 3, 45),
                    (3, 4, 50),
                    (0, 5, 30),
                    (2, 8, 35),
                    (3, 7, 26),
                    (4, 6, 40),
                    (8, 9, 5),
                ],
                151,
            ),
            (
                "test_nasty_blossom_least_slack",
                &[
                    (0, 1, 45),
                    (0, 4, 45),
                    (1, 2, 50),
                    (2, 3, 45),
                    (3, 4, 50),
                    (0, 5, 30),
                    (2, 8, 35),
                    (3, 7, 28),
                    (4, 6, 26),
                    (8, 9, 5),
                ],
                139,
            ),
            (
                "test_nasty_blossom_augmenting",
                &[
                    (0, 1, 45),
                    (0, 6, 45),
                    (1, 2, 50),
                    (2, 3, 45),
                    (3, 4, 95),
                    (3, 5, 94),
                    (4, 5, 94),
                    (5, 6, 50),
                    (0, 7, 30),
                    (2, 10, 35),
                    (4, 8, 36),
                    (6, 9, 26),
                    (10, 11, 5),
                ],
                241,
            ),
            (
                "test_nasty_blossom_expand_recursively",
                &[
                    (0, 1, 40),
                    (0, 2, 40),
                    (1, 2, 60),
                    (1, 3, 55),
                    (2, 4, 55),
                    (3, 4, 50),
                    (0, 7, 15),
                    (4, 6, 30),
                    (6, 5, 10),
                    (7, 9, 10),
                    (3, 8, 30),
                ],
                145,
            ),
        ];
        let mut nested = 0;
        let mut inner_expansions = 0;
        for &(name, edges, expected) in cases {
            let n = edges.iter().map(|&(u, v, _)| u.max(v)).max().unwrap() + 1;
            for shift in 0..n {
                for reverse in [false, true] {
                    for cardinality_first in [false, true] {
                        let mut input: Vec<_> = edges
                            .iter()
                            .map(|&(u, v, weight)| WeightedEdge {
                                u: (u + shift) % n,
                                v: (v + shift) % n,
                                weight: i128::from(weight),
                            })
                            .collect();
                        if reverse {
                            input.reverse();
                        }
                        let mut solver = Solver::new(n, input);
                        solver.solve(cardinality_first);
                        let actual: i128 = solver
                            .mate
                            .iter()
                            .flatten()
                            .filter(|e| e.from < e.to)
                            .map(|e| solver.edges[e.edge].weight)
                            .sum();
                        assert_eq!(actual, expected, "{name}, shift={shift}, reverse={reverse}");
                        nested += solver.statistics.nested_contractions;
                        inner_expansions += solver.statistics.inner_expansions;
                    }
                }
            }
        }
        assert!(nested > 0, "fixtures must exercise nested contractions");
        assert!(
            inner_expansions > 0,
            "fixtures must exercise mid-stage inner expansion"
        );
    }

    #[test]
    fn deep_blossom_operations_use_heap_stacks() {
        std::thread::Builder::new()
            .stack_size(64 * 1024)
            .spawn(|| {
                let depth = if cfg!(miri) { 32 } else { 3000 };
                let n = 2 * depth + 1;
                let mut edges = Vec::new();
                for i in 0..depth {
                    let (a, b) = (2 * i + 1, 2 * i + 2);
                    edges.push(WeightedEdge {
                        u: 0,
                        v: a,
                        weight: 1,
                    });
                    edges.push(WeightedEdge {
                        u: a,
                        v: b,
                        weight: 1,
                    });
                    edges.push(WeightedEdge {
                        u: b,
                        v: 0,
                        weight: 1,
                    });
                }
                let mut solver = Solver::new(n, edges);
                let mut child = 0;
                for i in 0..depth {
                    let blossom = solver.free.pop().unwrap();
                    let (a, b) = (2 * i + 1, 2 * i + 2);
                    let links = vec![
                        solver.link(3 * i, 0),
                        solver.link(3 * i + 1, a),
                        solver.link(3 * i + 2, b),
                    ];
                    solver.mate[a] = Some(links[1]);
                    solver.mate[b] = Some(links[1].reversed());
                    for c in [child, a, b] {
                        solver.parent[c] = Some(blossom);
                    }
                    solver.base[blossom] = 0;
                    solver.blossoms[blossom] = Some(Blossom {
                        children: vec![child, a, b],
                        edges: links,
                        best_edges: None,
                    });
                    child = blossom;
                }
                solver.inside.fill(child);
                assert_eq!(solver.leaves(child).len(), n);
                solver.augment_blossom(child, n - 1);
                solver.mate[n - 1] = None; // the caller would attach this new base externally
                assert_eq!(solver.base[child], n - 1);
                for v in 0..n - 1 {
                    let mate = solver.mate[v].unwrap();
                    assert_eq!(solver.mate[mate.to].unwrap().to, v);
                }
                solver.expand_blossom(child, true);
                assert!(solver.blossoms.iter().all(Option::is_none));
                assert!(solver.parent.iter().all(Option::is_none));
                assert_eq!(solver.free.len(), n);
                for v in 0..n {
                    assert_eq!(solver.inside[v], v);
                }
            })
            .unwrap()
            .join()
            .unwrap();
    }
}
