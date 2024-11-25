use crate::visit::{IntoNeighborsDirected, NodeCount, NodeIndexable};

use super::{Direction, UnitMeasure};

#[cfg(feature = "rayon")]
use rayon::prelude::*;

/// Norm used for score normalization.
#[derive(Clone, Copy)]
pub enum HitsNorm {
    One,
    Two,
}
impl Default for HitsNorm {
    fn default() -> Self {
        Self::One
    }
}

/// To compute square root of float-pointing numbers.
pub trait Sqrt {
    fn sqrt(&self) -> Self;
}
impl Sqrt for f32 {
    fn sqrt(&self) -> Self {
        Self::sqrt(*self)
    }
}
impl Sqrt for f64 {
    fn sqrt(&self) -> Self {
        Self::sqrt(*self)
    }
}

fn compute_normalized_score<N, H>(
    network: N,
    score1: &mut [H],
    score2: &[H],
    dir: Direction,
    norm: HitsNorm,
) -> H
where
    N: NodeCount + IntoNeighborsDirected + NodeIndexable,
    H: UnitMeasure + Sqrt + Copy,
{
    match norm {
        HitsNorm::One => (0..network.node_count())
            .map(|page| {
                score1[page] = network
                    .neighbors_directed(network.from_index(page), dir)
                    .map(|ix| score2[network.to_index(ix)])
                    .sum::<H>();
                score1[page]
            })
            .sum::<H>(),
        HitsNorm::Two => (0..network.node_count())
            .map(|page| {
                score1[page] = network
                    .neighbors_directed(network.from_index(page), dir)
                    .map(|ix| score2[network.to_index(ix)])
                    .sum::<H>();
                score1[page] * score1[page]
            })
            .sum::<H>()
            .sqrt(),
    }
}

fn normalize<H>(score: &[H], norm: H) -> Vec<H>
where
    H: UnitMeasure + Copy,
{
    score.iter().map(|s| *s / norm).collect::<Vec<H>>()
}

fn delta<H>(old_score: &[H], new_score: &[H]) -> H
where
    H: UnitMeasure + Copy,
{
    new_score
        .iter()
        .zip(old_score)
        .map(|(new, old)| (*new - *old) * (*new - *old))
        .sum::<H>()
}

fn max<H>(a: H, b: H) -> H
where
    H: PartialOrd,
{
    if a < b {
        b
    } else {
        a
    }
}

/// Hyperlink-Induced Topic Search algorithm [HITS][ht]
///
/// Computes the Authority and Hub scores of nodes in a directed graph.
///
/// # Complexity
/// Time complexity is **O(N|V||E|)**.
/// Space complexity is **O(|V| + |E|)**
/// where **N** is the number of iterations, **|V|** the number of vertices (i.e nodes) and **|E|** the number of edges.
///
/// [ht]: https://en.wikipedia.org/wiki/HITS_algorithm
/// # Example
/// ```rust
/// use petgraph::Graph;
/// use petgraph::algo::hits;
/// let mut g: Graph<(), usize> = Graph::new();
/// assert_eq!(hits(&g, Some(0.001_f64), 1, Default::default()), (vec![], vec![])); // empty graphs have no node hits scores.
/// //Example from https://www.geeksforgeeks.org/hyperlink-induced-topic-search-hits-algorithm-using-networkx-module-python/
/// g.add_node(());
/// g.add_node(());
/// g.add_node(());
/// g.add_node(());
/// g.add_node(());
/// g.add_node(());
/// g.add_node(());
/// g.extend_with_edges(&[
///     (0, 3),
///     (1, 2),
///     (1, 4),
///     (2, 0),
///     (3, 2),
///     (4, 3),
///     (4, 1),
///     (4, 5),
///     (4, 2),
///     (5, 2),
///     (5, 7),
///     (6, 0),
///     (6, 2),
///     (7, 0),
/// ]);
/// let (auths, hubs) = hits::<_, f32>(&g, None, 50, Default::default());
/// let expected_hubs = vec![0.046, 0.158, 0.037, 0.134, 0.259, 0.158, 0.171, 0.037];
/// let expected_auths = vec![0.109, 0.114, 0.388, 0.135, 0.070, 0.114, 0.0, 0.070];
/// assert_eq!(expected_hubs, hubs.iter().map(|h| (*h * 1000.).round()/1000.).collect::<Vec<_>>());
/// assert_eq!(expected_auths, auths.iter().map(|a| (*a * 1000.).round()/1000.).collect::<Vec<_>>());
/// ```
pub fn hits<N, H>(network: N, tol: Option<H>, nb_iter: usize, norm: HitsNorm) -> (Vec<H>, Vec<H>)
where
    N: NodeCount + IntoNeighborsDirected + NodeIndexable,
    H: UnitMeasure + Copy + Sqrt,
{
    let node_count = network.node_count();
    if node_count == 0 {
        return (vec![], vec![]);
    }
    let mut tolerance = H::default_tol();
    if let Some(_tol) = tol {
        tolerance = _tol;
    }
    let mut auth = vec![H::one(); node_count];
    let mut hub = vec![H::one(); node_count];

    for _ in 0..nb_iter {
        // Compute the normalized scores.
        let norm_sum_in_hubs =
            compute_normalized_score(network, &mut auth, &hub, Direction::Incoming, norm);

        let norm_sum_out_auths =
            compute_normalized_score(network, &mut hub, &auth, Direction::Outgoing, norm);

        // Update the scores.
        let new_auth = normalize(&auth, norm_sum_in_hubs);
        let new_hub = normalize(&hub, norm_sum_out_auths);
        if max(delta(&auth, &new_auth), delta(&hub, &new_hub)) <= tolerance {
            return (new_auth, new_hub);
        } else {
            auth = new_auth;
            hub = new_hub;
        }
    }
    (auth, hub)
}

#[cfg(feature = "rayon")]
fn par_compute_normalized_score<N, H>(
    network: N,
    score1: &mut [H],
    score2: &[H],
    dir: Direction,
    norm: HitsNorm,
) -> H
where
    N: NodeCount + IntoNeighborsDirected + NodeIndexable + std::marker::Sync,
    H: UnitMeasure + Sqrt + Copy + std::marker::Send + std::marker::Sync,
{
    score1.par_iter_mut().enumerate().for_each(|(page, score)| {
        *score = network
            .neighbors_directed(network.from_index(page), dir)
            .map(|ix| score2[network.to_index(ix)])
            .sum::<H>();
    });
    match norm {
        HitsNorm::One => score1.par_iter().map(|score| *score).sum::<H>(),
        HitsNorm::Two => score1
            .par_iter()
            .map(|score| *score * *score)
            .sum::<H>()
            .sqrt(),
    }
}

#[cfg(feature = "rayon")]
fn par_normalize<H>(score: &[H], norm: H) -> Vec<H>
where
    H: UnitMeasure + Copy + std::marker::Send + std::marker::Sync,
{
    score.par_iter().map(|s| *s / norm).collect::<Vec<H>>()
}

/// Parallel Hyperlink-Induced Topic Search algorithm.
///
/// See [`hits`].
#[cfg(feature = "rayon")]
pub fn parallel_hits<N, H>(
    network: N,
    tol: Option<H>,
    nb_iter: usize,
    norm: HitsNorm,
) -> (Vec<H>, Vec<H>)
where
    N: NodeCount + IntoNeighborsDirected + NodeIndexable + std::marker::Sync,
    H: UnitMeasure + Copy + Sqrt + std::marker::Send + std::marker::Sync,
{
    let node_count = network.node_count();
    if node_count == 0 {
        return (vec![], vec![]);
    }
    let mut tolerance = H::default_tol();
    if let Some(_tol) = tol {
        tolerance = _tol;
    }
    let mut auth: Vec<H> = (0..node_count).into_par_iter().map(|_i| H::one()).collect();
    let mut hub: Vec<H> = (0..node_count).into_par_iter().map(|_i| H::one()).collect();

    for _ in 0..nb_iter {
        // Compute the normalized scores.
        let norm_sum_in_hubs =
            par_compute_normalized_score(network, &mut auth, &hub, Direction::Incoming, norm);

        let norm_sum_out_auths =
            par_compute_normalized_score(network, &mut hub, &auth, Direction::Outgoing, norm);

        // Update the scores.
        let new_auth = par_normalize(&auth, norm_sum_in_hubs);
        let new_hub = par_normalize(&hub, norm_sum_out_auths);
        if max(delta(&auth, &new_auth), delta(&hub, &new_hub)) <= tolerance {
            return (new_auth, new_hub);
        } else {
            auth = new_auth;
            hub = new_hub;
        }
    }
    (auth, hub)
}
