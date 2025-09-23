use alloc::{sync::Arc, vec::Vec};
use core::{
    fmt::Debug,
    hash::{BuildHasher, Hash, Hasher},
    marker::PhantomData,
    ops::RangeInclusive,
};

use proptest::{prelude::*, sample::SizeRange};

use crate::{
    graphmap::NodeTrait,
    prelude::GraphMap,
    stable_graph::{IndexType, StableGraph},
    EdgeType, Graph,
};

/// The minimum and maximum range/bounds on the density of a graph.
/// The interval must form a subset of `[0.0, f64::MAX]`.
///
/// The `Default` is `0.0..=1.0`.
#[derive(Clone, PartialEq, Debug)]
pub struct DensityRange {
    range: RangeInclusive<f64>,
}

/// Creates a `DensityRange` from some value that is convertible into it.
pub fn density_range(from: impl Into<DensityRange>) -> DensityRange {
    from.into()
}

impl Default for DensityRange {
    fn default() -> Self {
        Self { range: 0.0..=1.0 }
    }
}

impl DensityRange {
    pub fn as_range(&self) -> &RangeInclusive<f64> {
        &self.range
    }

    pub fn start(&self) -> f64 {
        *self.range.start()
    }

    pub fn end(&self) -> f64 {
        *self.range.end()
    }
}

impl From<f64> for DensityRange {
    fn from(value: f64) -> Self {
        Self {
            range: value..=value,
        }
    }
}

impl From<RangeInclusive<f64>> for DensityRange {
    fn from(range: RangeInclusive<f64>) -> Self {
        Self { range }
    }
}

pub type BoxedNodesStrategy<N> = BoxedStrategy<Vec<N>>;
pub type BoxedEdgesStrategy<E> = BoxedStrategy<Vec<(usize, usize, E)>>;

/// The number of edges at which a graph with non-parallel edges
/// in the provided configuration would reach full connectivity.
pub fn saturating_edge_count(node_count: usize, directed: bool, self_loops: bool) -> usize {
    if node_count == 0 {
        return 0;
    }

    let mut edge_count = node_count * node_count;

    if !directed {
        // remove one of the two matrix triangles (excluding diagonal) from the count:
        edge_count -= (node_count * (node_count - 1)) / 2;
    }

    if !self_loops {
        // remove the diagonal of the matrix:
        edge_count -= node_count;
    }

    edge_count
}

/// Returns the `SizeRange` that corresponds to a `DensityRange`,
/// given a certain saturating edge count.
pub fn size_for_density_range(
    saturating_edge_count: usize,
    density: impl Into<DensityRange>,
) -> SizeRange {
    let density: DensityRange = density.into();

    let min_density = density.start();
    let max_density = density.end();

    assert!(min_density >= 0.0, "density must be positive");
    assert!(max_density >= 0.0, "density must be positive");

    let start = (min_density * (saturating_edge_count as f64)).round() as usize;
    let end = (max_density * (saturating_edge_count as f64)).round() as usize;

    (start..=end).into()
}

fn min_node_count_for_edges(self_loops: bool) -> usize {
    if self_loops {
        1
    } else {
        2
    }
}

/// Generates a vec of arbitrary nodes.
pub fn arb_nodes<N, T>(node: T, size: impl Into<SizeRange>) -> BoxedNodesStrategy<N>
where
    N: Debug,
    T: 'static + Strategy<Value = N>,
{
    prop::collection::vec(node, size).boxed()
}

/// `f(x) = x²` for edge probability (bias towards lower)
pub fn arb_probability_squared() -> BoxedStrategy<f64> {
    (0.0..1.0_f64).prop_map(|p| p * p).boxed()
}

/// `f(x) = x` for edge probability (no bias)
pub fn arb_probability_linear() -> BoxedStrategy<f64> {
    (0.0..1.0_f64).boxed()
}

/// Generates an arbitrary pair of edge endpoints for a given graph configuration.
///
/// The `node_count` must be `>= 1` if `self_loops == true`, otherwise `>= 2`.
pub fn arb_edge_endpoint_pair(
    node_count: usize,
    directed: bool,
    self_loops: bool,
) -> BoxedStrategy<(usize, usize)> {
    let min_required_node_count = min_node_count_for_edges(self_loops);

    assert!(
        node_count >= min_required_node_count,
        "node_count must be greater than or equal to {min_required_node_count}"
    );

    let diagonal_weight = node_count as u32;
    let triangle_weight = ((node_count * (node_count - 1)) / 2) as u32;

    // The two triangles of the graph's adjacency matrix, excluding the diagonal:
    let source_matrix_triangle = (1..node_count).prop_flat_map(|source| (Just(source), 0..source));
    let target_matrix_triangle = (1..node_count).prop_flat_map(|target| (0..target, Just(target)));

    // The diagonal of the graph's adjacency matrix:
    let matrix_diagonal = (0..node_count).prop_map(|idx| (idx, idx));

    match (directed, self_loops) {
        (true, true) => prop_oneof![
            triangle_weight => source_matrix_triangle,
            diagonal_weight => matrix_diagonal,
            triangle_weight => target_matrix_triangle,
        ]
        .boxed(),
        (true, false) => prop_oneof![
            triangle_weight => source_matrix_triangle,
            triangle_weight => target_matrix_triangle,
        ]
        .boxed(),
        (false, true) => prop_oneof![
            triangle_weight => source_matrix_triangle,
            diagonal_weight => matrix_diagonal,
        ]
        .boxed(),
        (false, false) => prop_oneof![
            triangle_weight => source_matrix_triangle,
        ]
        .boxed(),
    }
}

/// Generates a vec of arbitrary, unique edge endpoints for a given graph configuration.
pub fn arb_edge_endpoints(
    node_count: usize,
    directed: bool,
    self_loops: bool,
    size: impl Into<SizeRange>,
) -> BoxedStrategy<Vec<(usize, usize)>> {
    let min_required_node_count = min_node_count_for_edges(self_loops);

    if node_count < min_required_node_count {
        return Just(Vec::default()).boxed();
    }

    let size: SizeRange = size.into();

    let edge_count = saturating_edge_count(node_count, directed, self_loops);
    assert!(size.end_incl() <= edge_count, "sample size for a graph with {node_count} nodes must be less than of equal to {edge_count}");

    prop::collection::hash_set(
        arb_edge_endpoint_pair(node_count, directed, self_loops),
        size,
    )
    .prop_map(Vec::from_iter)
    .boxed()
}

/// Generates a vec of arbitrary, non-unique edge endpoints for a given graph configuration.
pub fn arb_edge_endpoints_parallel(
    node_count: usize,
    directed: bool,
    self_loops: bool,
    size: impl Into<SizeRange>,
) -> BoxedStrategy<Vec<(usize, usize)>> {
    let min_required_node_count = min_node_count_for_edges(self_loops);

    if node_count < min_required_node_count {
        return Just(Vec::default()).boxed();
    }

    prop::collection::vec(
        arb_edge_endpoint_pair(node_count, directed, self_loops),
        size,
    )
    .boxed()
}

/// Generates a vec of arbitrary, unique edge endpoints for a given graph configuration.
pub fn arb_edge_endpoints_with_density(
    node_count: usize,
    directed: bool,
    self_loops: bool,
    density: impl Into<DensityRange>,
) -> BoxedStrategy<Vec<(usize, usize)>> {
    let density: DensityRange = density.into();

    let edge_count = saturating_edge_count(node_count, directed, self_loops);
    assert!(
        density.start() >= 0.0,
        "sample density for a graph must be positive"
    );
    assert!(
        density.end() <= 1.0,
        "sample density for a graph without parallel edges must be less than of equal to 1.0"
    );

    let size: SizeRange = size_for_density_range(edge_count, density);

    arb_edge_endpoints(node_count, directed, self_loops, size)
}

/// Generates a vec of arbitrary, non-unique edge endpoints for a given graph configuration.
pub fn arb_edge_endpoints_with_density_parallel(
    node_count: usize,
    directed: bool,
    self_loops: bool,
    density: impl Into<DensityRange>,
) -> BoxedStrategy<Vec<(usize, usize)>> {
    let density: DensityRange = density.into();

    let edge_count = saturating_edge_count(node_count, directed, self_loops);
    assert!(
        density.start() >= 0.0,
        "sample density for a graph must be positive"
    );

    let size: SizeRange = size_for_density_range(edge_count, density);

    arb_edge_endpoints(node_count, directed, self_loops, size)
}

/// Generates a vec of arbitrary edges, by mapping from a strategy of edge endpoints.
pub fn arb_edges_from_endpoints<N, E, T, F>(
    nodes: &Arc<Vec<N>>,
    endpoints: T,
    edge: F,
) -> BoxedEdgesStrategy<E>
where
    N: 'static + Debug,
    E: 'static + Debug,
    T: 'static + Strategy<Value = Vec<(usize, usize)>>,
    F: 'static + Fn(usize, usize, Arc<Vec<N>>) -> BoxedStrategy<E>,
{
    let nodes = Arc::clone(nodes);
    let edge = Arc::new(edge);
    endpoints
        .prop_map(move |endpoints: Vec<(usize, usize)>| {
            endpoints
                .into_iter()
                .map(|(source, target)| {
                    let nodes = Arc::clone(&nodes);
                    let edge = edge(source, target, nodes);
                    (Just((source, target)), edge)
                        .prop_map(|((source, target), edge)| (source, target, edge))
                })
                .collect()
        })
        .prop_flat_map(|strategies: Vec<_>| strategies)
        .boxed()
}

/// Generates a vec of arbitrary edges, by mapping from a strategy of edge endpoints.
pub fn arb_edges_with_endpoints<N, E, T, F>(
    nodes: &Arc<Vec<N>>,
    endpoints: T,
    edge: F,
) -> BoxedEdgesStrategy<E>
where
    N: 'static + Debug,
    E: 'static + Debug,
    T: 'static + Strategy<Value = Vec<(usize, usize)>>,
    F: 'static + Fn(usize, usize, Arc<Vec<N>>) -> BoxedStrategy<E>,
{
    let nodes = Arc::clone(nodes);
    let edge = Arc::new(edge);
    endpoints
        .prop_map(move |endpoints: Vec<(usize, usize)>| {
            endpoints
                .into_iter()
                .map(|(source, target)| {
                    let nodes = Arc::clone(&nodes);
                    let edge = edge(source, target, nodes);
                    (Just((source, target)), edge)
                        .prop_map(|((source, target), edge)| (source, target, edge))
                })
                .collect()
        })
        .prop_flat_map(|strategies: Vec<_>| strategies)
        .boxed()
}

/// Generates a vec of arbitrary, unique edges for a given graph configuration.
pub fn arb_edges_with_density<N, E, F>(
    nodes: &Arc<Vec<N>>,
    directed: bool,
    self_loops: bool,
    density: impl Into<DensityRange>,
    edge: F,
) -> BoxedEdgesStrategy<E>
where
    N: 'static + Clone + Debug,
    E: 'static + Clone + Debug,
    F: 'static + Fn(usize, usize, Arc<Vec<N>>) -> BoxedStrategy<E>,
{
    let saturated_edge_count = saturating_edge_count(nodes.len(), directed, self_loops);
    let size = size_for_density_range(saturated_edge_count, density);

    arb_edges(nodes, directed, self_loops, size, edge)
}

/// Generates a vec of arbitrary, unique edges for a given graph configuration.
pub fn arb_edges<N, E, F>(
    nodes: &Arc<Vec<N>>,
    directed: bool,
    self_loops: bool,
    size: impl Into<SizeRange>,
    edge: F,
) -> BoxedEdgesStrategy<E>
where
    N: 'static + Clone + Debug,
    E: 'static + Clone + Debug,
    F: 'static + Fn(usize, usize, Arc<Vec<N>>) -> BoxedStrategy<E>,
{
    let nodes = Arc::clone(nodes);
    let node_count = nodes.len();

    let min_required_node_count = min_node_count_for_edges(self_loops);

    if node_count < min_required_node_count {
        return Just(Vec::default()).boxed();
    }

    let size: SizeRange = size.into();

    let saturating_edge_count = saturating_edge_count(node_count, directed, self_loops);
    assert!(size.end_incl() <= saturating_edge_count, "sample size for a graph with {node_count} nodes must be less than of equal to {saturating_edge_count}");

    // We want to internally sample edges as `(usize, usize, E)` into a `HashSetStrategy`,
    // to ensure non-parallel (i.e. unique) edges, which we then map into a `VecStrategy`.
    // Unfortunately `prop::collection::hash_map` expects separate key/value strategies.
    // So what we do instead is use `prop::collection::hash_set` with a custom type that
    // ignores the `E`
    #[derive(Debug)]
    struct EndpointsOrdEdge<E> {
        source: usize,
        target: usize,
        edge: E,
    }

    impl<E> Eq for EndpointsOrdEdge<E> {}

    impl<E> PartialEq for EndpointsOrdEdge<E> {
        fn eq(&self, other: &Self) -> bool {
            self.source == other.source && self.target == other.target
        }
    }

    impl<E> Hash for EndpointsOrdEdge<E> {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.source.hash(state);
            self.target.hash(state);
        }
    }

    prop::collection::hash_set(
        arb_edge_endpoint_pair(nodes.len(), directed, self_loops)
            .prop_map(move |(source, target)| {
                let nodes = Arc::clone(&nodes);
                (Just((source, target)), edge(source, target, nodes))
            })
            .prop_flat_map(|(endpoints, edge)| (endpoints, edge))
            .prop_map(|((source, target), edge)| EndpointsOrdEdge {
                source,
                target,
                edge,
            }),
        size,
    )
    .prop_map(|hash_set| {
        hash_set
            .into_iter()
            .map(|endpoints_ord_edge| {
                let EndpointsOrdEdge {
                    source,
                    target,
                    edge,
                } = endpoints_ord_edge;
                (source, target, edge)
            })
            .collect()
    })
    .boxed()
}

/// Generates a vec of arbitrary, non-unique edges for a given graph configuration.
pub fn arb_edges_with_density_parallel<N, E, F>(
    nodes: &Arc<Vec<N>>,
    directed: bool,
    self_loops: bool,
    density: impl Into<DensityRange>,
    edge: F,
) -> BoxedEdgesStrategy<E>
where
    N: 'static + Clone + Debug,
    E: 'static + Clone + Debug,
    F: 'static + Fn(usize, usize, Arc<Vec<N>>) -> BoxedStrategy<E>,
{
    let saturated_edge_count = saturating_edge_count(nodes.len(), directed, self_loops);
    let size = size_for_density_range(saturated_edge_count, density);

    arb_edges(nodes, directed, self_loops, size, edge)
}

/// Generates a vec of arbitrary, non-unique edges for a given graph configuration.
pub fn arb_edges_parallel<N, E, F>(
    nodes: &Arc<Vec<N>>,
    directed: bool,
    self_loops: bool,
    size: impl Into<SizeRange>,
    edge: F,
) -> BoxedEdgesStrategy<E>
where
    N: 'static + Clone + Debug,
    E: 'static + Clone + Debug,
    F: 'static + Fn(usize, usize, Arc<Vec<N>>) -> BoxedStrategy<E>,
{
    let nodes = Arc::clone(nodes);
    let node_count = nodes.len();

    let min_required_node_count = min_node_count_for_edges(self_loops);

    if node_count < min_required_node_count {
        return Just(Vec::default()).boxed();
    }

    prop::collection::vec(
        arb_edge_endpoint_pair(node_count, directed, self_loops)
            .prop_map(move |(source, target)| {
                let nodes = Arc::clone(&nodes);
                (Just((source, target)), edge(source, target, nodes))
            })
            .prop_flat_map(|(endpoints, edge)| (endpoints, edge))
            .prop_map(|((source, target), edge)| (source, target, edge)),
        size,
    )
    .boxed()
}

/// Generates a `Graph<N, E, Ty, Ix>` from a provided pair of node and edge strategies.
pub fn arb_graph<N, E, Ty, Ix, F>(
    nodes_strategy: BoxedNodesStrategy<N>,
    edges_strategy: &Arc<F>,
) -> BoxedStrategy<Graph<N, E, Ty, Ix>>
where
    N: 'static + Clone + Debug,
    E: 'static + Clone + Debug,
    Ty: EdgeType,
    Ix: IndexType,
    F: 'static + Fn(Arc<Vec<N>>) -> BoxedEdgesStrategy<E> + ?Sized,
{
    let edges_strategy = edges_strategy.clone();

    nodes_strategy
        .prop_flat_map(move |nodes: Vec<N>| {
            let node_count = nodes.len();
            let nodes = Arc::new(nodes);

            edges_strategy(Arc::clone(&nodes)).prop_map(move |edges: Vec<(usize, usize, E)>| {
                let edge_count = edges.len();

                let mut graph = Graph::<N, E, Ty, Ix>::with_capacity(node_count, edge_count);

                let node_indices: Vec<_> =
                    nodes.iter().map(|n| graph.add_node(n.clone())).collect();

                for (source, target, edge) in edges.into_iter() {
                    graph.add_edge(node_indices[source], node_indices[target], edge);
                }

                graph
            })
        })
        .boxed()
}

/// Generates a `StableGraph<N, E, Ty, Ix>` from a provided pair of node and edge strategies.
pub fn arb_stable_graph<N, E, Ty, Ix, F>(
    nodes_strategy: BoxedNodesStrategy<N>,
    edges_strategy: F,
) -> BoxedStrategy<StableGraph<N, E, Ty, Ix>>
where
    N: 'static + Clone + Debug,
    E: 'static + Clone + Debug,
    Ty: EdgeType,
    Ix: IndexType,
    F: 'static + Fn(Arc<Vec<N>>) -> BoxedEdgesStrategy<E>,
{
    nodes_strategy
        .prop_flat_map(move |nodes: Vec<N>| {
            let node_count = nodes.len();
            let nodes = Arc::new(nodes);

            edges_strategy(nodes.clone()).prop_map(move |edges: Vec<(usize, usize, E)>| {
                let edge_count = edges.len();

                let mut graph = StableGraph::<N, E, Ty, Ix>::with_capacity(node_count, edge_count);

                let node_indices: Vec<_> =
                    nodes.iter().map(|n| graph.add_node(n.clone())).collect();

                for (source, target, edge) in edges.into_iter() {
                    graph.add_edge(node_indices[source], node_indices[target], edge);
                }

                graph
            })
        })
        .boxed()
}

/// Generates a `GraphMap<N, E, Ty, S>` from a provided pair of node and edge strategies.
pub fn arb_graph_map<N, E, Ty, S, F>(
    nodes_strategy: BoxedNodesStrategy<N>,
    edges_strategy: F,
) -> BoxedStrategy<GraphMap<N, E, Ty, S>>
where
    N: 'static + Clone + Debug + NodeTrait,
    E: 'static + Clone + Debug,
    Ty: EdgeType,
    S: Default + BuildHasher,
    F: 'static + Fn(Arc<Vec<N>>) -> BoxedEdgesStrategy<E>,
{
    nodes_strategy
        .prop_flat_map(move |nodes: Vec<N>| {
            let node_count = nodes.len();
            let nodes = Arc::new(nodes);

            edges_strategy(nodes.clone()).prop_map(move |edges: Vec<(usize, usize, E)>| {
                let edge_count = edges.len();

                let mut graph = GraphMap::<N, E, Ty, S>::with_capacity(node_count, edge_count);

                let node_indices: Vec<_> = nodes.iter().map(|n| graph.add_node(*n)).collect();

                for (source, target, edge) in edges.into_iter() {
                    graph.add_edge(node_indices[source], node_indices[target], edge);
                }

                graph
            })
        })
        .boxed()
}

/// Generates edges for graphs with small-world properties, including short average path lengths
/// and high clustering, based on the **Watts–Strogatz model**, where `k_over_2` (`1..(nodes.len() / 2)`)
/// denotes the desired initial out-degree per node, and `beta` (`0.0..1.0`) the probability that a
/// given edge will be rewired to point to a randomly selected target node (excluding self-loops).
///
/// https://en.wikipedia.org/wiki/Watts%E2%80%93Strogatz_model
pub fn arb_watts_strogatz_edges_undirected_parallel<N, E, F>(
    nodes: &Arc<Vec<N>>,
    self_loops: bool,
    k_over_2: usize,
    beta: f64,
    edge: F,
) -> BoxedStrategy<Vec<(usize, usize, E)>>
where
    N: 'static + Clone + Debug,
    E: 'static + Clone + Debug,
    F: 'static + Fn(usize, usize, Arc<Vec<N>>) -> BoxedStrategy<E> + Clone + 'static,
{
    let node_count = nodes.len();

    assert!(k_over_2 > 0, "`k_over_2` must be greater than `0`");
    assert!(
        k_over_2 < node_count / 2,
        "`k_over_2` must be less than `node_count / 2`"
    );

    assert!(beta >= 0.0, "`beta` must be greater than or equal to `0.0`");
    assert!(beta <= 1.0, "`beta` must be greater than or equal to `1.0`");

    // Generate all edge endpoints for the ring lattice (undirected, no duplicates)
    let edge_endpoints: Vec<(usize, usize)> = (0..node_count)
        .flat_map(move |i| (1..=k_over_2).map(move |j| (i, (i + j) % node_count)))
        .collect::<Vec<_>>();

    let edge_count = edge_endpoints.len();

    // Generate randomized targets, along with corresponding weighted dice-rolls:
    let rewired_targets =
        prop::collection::vec((prop::bool::weighted(beta), 0..node_count), edge_count).prop_map(
            |random_targets| {
                random_targets
                    .into_iter()
                    .map(|(rewire, random_target)| rewire.then_some(random_target))
                    .collect::<Vec<_>>()
            },
        );

    // Generate randomly rewired edge endpoints:
    let rewired_edge_endpoints = (Just(edge_endpoints), rewired_targets).prop_map(
        move |(edge_endpoints, random_targets)| {
            edge_endpoints
                .into_iter()
                .zip(random_targets.into_iter())
                .map(|((source, target), random_target)| {
                    let rewired_target = random_target.unwrap_or(target);
                    if self_loops || (rewired_target != source) {
                        (source, rewired_target)
                    } else {
                        (source, target)
                    }
                })
                .collect::<Vec<_>>()
        },
    );

    let edge = Arc::new(edge);

    rewired_edge_endpoints
        .prop_flat_map({
            let nodes = Arc::clone(&nodes);
            let edge = Arc::clone(&edge);
            move |endpoints| {
                endpoints
                    .into_iter()
                    .map(|(source, target)| {
                        let nodes = Arc::clone(&nodes);
                        let edge_strategy = edge(source, target, nodes);
                        edge_strategy.prop_map(move |edge| (source, target, edge))
                    })
                    .collect::<Vec<_>>()
            }
        })
        .boxed()
}

/// Arbitrary node parameters.
pub enum ArbitraryGraphNodesParameters<N> {
    Size {
        node: BoxedStrategy<N>,
        size: SizeRange,
    },
    Custom {
        strategy: BoxedNodesStrategy<N>,
    },
}

impl<N> Default for ArbitraryGraphNodesParameters<N>
where
    N: 'static + Default + Arbitrary,
{
    fn default() -> Self {
        Self::Size {
            node: N::arbitrary().boxed(),
            size: (0..25).into(),
        }
    }
}

impl<N> ArbitraryGraphNodesParameters<N> {
    pub fn to_strategy(&self) -> BoxedNodesStrategy<N>
    where
        N: 'static + Clone + Debug,
    {
        match self {
            Self::Size { node, size } => arb_nodes(node.clone(), size.clone()),
            Self::Custom { strategy } => strategy.clone(),
        }
    }
}

/// Arbitrary edge parameters.
pub struct ArbitraryGraphEdgesParameters<N, E, Ty> {
    pub kind: ArbitraryGraphEdgesParametersKind<N, E>,
    _phantom: PhantomData<Ty>,
}

impl<N, E, Ty> Default for ArbitraryGraphEdgesParameters<N, E, Ty>
where
    N: 'static + Default + Arbitrary,
    E: 'static + Default + Arbitrary,
    Ty: EdgeType,
{
    fn default() -> Self {
        Self {
            kind: ArbitraryGraphEdgesParametersKind::new(Ty::is_directed()),
            _phantom: Default::default(),
        }
    }
}

impl<N, E, Ty> ArbitraryGraphEdgesParameters<N, E, Ty>
where
    N: 'static + Default + Arbitrary,
    E: 'static + Default + Arbitrary,
    Ty: EdgeType,
{
    fn to_strategy(&self) -> Arc<dyn Fn(Arc<Vec<N>>) -> BoxedEdgesStrategy<E>>
    where
        N: 'static + Clone + Debug,
        E: 'static + Clone + Debug,
        Ty: EdgeType,
    {
        self.kind.to_strategy::<Ty>()
    }
}

/// Arbitrary edge parameters kind
pub enum ArbitraryGraphEdgesParametersKind<N, E> {
    Size {
        edge: BoxedStrategy<E>,
        directed: bool,
        self_loops: bool,
        parallel_edges: bool,
        size: SizeRange,
    },
    Density {
        edge: BoxedStrategy<E>,
        directed: bool,
        self_loops: bool,
        parallel_edges: bool,
        density: DensityRange,
    },
    Custom {
        strategy: Arc<dyn Fn(Arc<Vec<N>>) -> BoxedEdgesStrategy<E>>,
    },
}

impl<N, E> ArbitraryGraphEdgesParametersKind<N, E>
where
    N: 'static + Default + Arbitrary,
    E: 'static + Default + Arbitrary,
{
    fn new(directed: bool) -> Self {
        Self::Density {
            edge: E::arbitrary().boxed(),
            directed,
            self_loops: false,
            parallel_edges: false,
            density: (0.0..=0.25).into(),
        }
    }
}

impl<N, E> ArbitraryGraphEdgesParametersKind<N, E> {
    pub fn to_strategy<Ty>(&self) -> Arc<dyn Fn(Arc<Vec<N>>) -> BoxedEdgesStrategy<E>>
    where
        N: 'static + Clone + Debug,
        E: 'static + Clone + Debug,
        Ty: EdgeType,
    {
        match self {
            Self::Size {
                edge,
                directed,
                self_loops,
                parallel_edges,
                size,
            } => {
                let edge = edge.clone();
                let directed = *directed;
                let self_loops = *self_loops;
                let parallel_edges = *parallel_edges;
                let size = size.clone();
                Arc::new(move |nodes: Arc<Vec<N>>| {
                    let edge = edge.clone();
                    let edge = move |_, _, _| edge.clone();
                    if parallel_edges {
                        arb_edges_parallel(&nodes, directed, self_loops, size.clone(), edge)
                    } else {
                        arb_edges(&nodes, directed, self_loops, size.clone(), edge)
                    }
                })
            }
            Self::Density {
                edge,
                directed,
                self_loops,
                parallel_edges,
                density,
            } => {
                let edge = edge.clone();
                let directed = *directed;
                let self_loops = *self_loops;
                let parallel_edges = *parallel_edges;
                let density = density.clone();
                Arc::new(move |nodes: Arc<Vec<N>>| {
                    let edge = edge.clone();
                    let edge = move |_, _, _| edge.clone();
                    if parallel_edges {
                        arb_edges_with_density_parallel(
                            &nodes,
                            directed,
                            self_loops,
                            density.clone(),
                            edge,
                        )
                    } else {
                        arb_edges_with_density(&nodes, directed, self_loops, density.clone(), edge)
                    }
                })
            }
            Self::Custom { strategy } => Arc::clone(strategy),
        }
    }
}

/// Arbitrary graph parameters.
pub struct ArbitraryGraphParameters<N, E, Ty> {
    pub nodes: ArbitraryGraphNodesParameters<N>,
    pub edges: ArbitraryGraphEdgesParameters<N, E, Ty>,
}

impl<N, E, Ty> Default for ArbitraryGraphParameters<N, E, Ty>
where
    N: 'static + Default + Arbitrary,
    E: 'static + Default + Arbitrary,
    Ty: EdgeType,
{
    fn default() -> Self {
        Self {
            nodes: Default::default(),
            edges: Default::default(),
        }
    }
}

impl<N, E, Ty, Ix> Arbitrary for Graph<N, E, Ty, Ix>
where
    N: 'static + Default + Arbitrary + Clone + Debug,
    E: 'static + Default + Arbitrary + Clone + Debug,
    Ty: EdgeType,
    Ix: IndexType,
{
    type Parameters = ArbitraryGraphParameters<N, E, Ty>;
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(params: Self::Parameters) -> Self::Strategy {
        let ArbitraryGraphParameters { nodes, edges } = params;

        let nodes_strategy = nodes.to_strategy();
        let edges_strategy = edges.to_strategy();

        arb_graph(nodes_strategy, &edges_strategy)
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloc::format;
    use std::collections::{HashMap, HashSet};

    use proptest::prelude::*;

    use crate::{graph::NodeIndex, visit::EdgeRef as _, Directed};

    use super::*;

    type Node = usize;
    type Edge = usize;

    fn arb_node() -> BoxedStrategy<Node> {
        Node::arbitrary().boxed()
    }

    fn arb_nodes(size: impl Into<SizeRange>) -> BoxedStrategy<Vec<Node>> {
        prop::collection::vec(arb_node(), size).boxed()
    }

    fn arb_edge(_source: usize, _target: usize, _nodes: Arc<Vec<Node>>) -> BoxedStrategy<Edge> {
        Edge::arbitrary().boxed()
    }

    proptest! {
        #[test]
        fn test_arb_edge_endpoint_pair(node_count in 2..10_usize, directed in prop::bool::ANY, self_loops in prop::bool::ANY) {
            let strategy = arb_edge_endpoint_pair(
                node_count,
                directed,
                self_loops,
            );

            proptest::test_runner::Config::with_cases(100);

            for _ in 0..100 {
                let mut runner = proptest::test_runner::TestRunner::default();

                let value_tree = strategy
                    .new_tree(&mut runner)
                    .expect("failed to generate value tree");
                let (source, target) = value_tree.current();

                prop_assert!(source < node_count, "source < node_count");
                prop_assert!(target < node_count, "target < node_count");

                if !directed {
                    prop_assert!(source >= target, "source >= target");
                }

                if !self_loops {
                    prop_assert!(source != target, "source != target");
                }
            }
        }
    }

    proptest! {
        #[test]
        fn test_arb_edge_endpoints(node_count in 2..10_usize, directed in prop::bool::ANY, self_loops in prop::bool::ANY) {
            let strategy = arb_edge_endpoints(
                node_count,
                directed,
                self_loops,
                0..node_count
            );

            proptest::test_runner::Config::with_cases(100);

            for _ in 0..100 {
                let mut runner = proptest::test_runner::TestRunner::default();

                let value_tree = strategy
                    .new_tree(&mut runner)
                    .expect("failed to generate value tree");
                let endpoints = value_tree.current();

                for (source, target) in endpoints {
                    prop_assert!(source < node_count, "source < node_count");
                    prop_assert!(target < node_count, "target < node_count");

                    if !directed {
                        prop_assert!(source >= target, "source >= target");
                    }

                    if !self_loops {
                        prop_assert!(source != target, "source != target");
                    }
                }
            }
        }
    }

    proptest! {
        #[test]
        fn test_arb_edge_endpoints_parallel(node_count in 2..10_usize, directed in prop::bool::ANY, self_loops in prop::bool::ANY) {
            let strategy = arb_edge_endpoints_parallel(
                node_count,
                directed,
                self_loops,
                0..node_count
            );

            proptest::test_runner::Config::with_cases(100);

            for _ in 0..100 {
                let mut runner = proptest::test_runner::TestRunner::default();

                let value_tree = strategy
                    .new_tree(&mut runner)
                    .expect("failed to generate value tree");
                let endpoints = value_tree.current();

                for (source, target) in endpoints {
                    prop_assert!(source < node_count, "source < node_count");
                    prop_assert!(target < node_count, "target < node_count");

                    if !directed {
                        prop_assert!(source >= target, "source >= target");
                    }

                    if !self_loops {
                        prop_assert!(source != target, "source != target");
                    }
                }
            }
        }
    }

    proptest! {
        #[test]
        fn test_arb_edges(nodes in arb_nodes(0..10_usize), directed in prop::bool::ANY, self_loops in prop::bool::ANY) {
            let node_count = nodes.len();
            let max_expected_edges = saturating_edge_count(node_count, directed, self_loops);

            let strategy = arb_edges(
                &Arc::new(nodes),
                directed,
                self_loops,
                0..node_count,
                arb_edge);

            proptest::test_runner::Config::with_cases(100);

            for _ in 0..100 {
                let mut runner = proptest::test_runner::TestRunner::default();

                let value_tree = strategy
                    .new_tree(&mut runner)
                    .expect("failed to generate value tree");
                let endpoints = value_tree.current();

                let endpoint_count = endpoints.len();

                prop_assert!(endpoint_count <= max_expected_edges);
                prop_assert_eq!(HashSet::<_>::from_iter(endpoints).len(), endpoint_count, "duplicate endpoints detected");
            }
        }
    }

    proptest! {
        #[test]
        fn test_arb_edges_parallel(nodes in arb_nodes(0..10_usize), directed in prop::bool::ANY, self_loops in prop::bool::ANY) {
            let node_count = nodes.len();
            let max_edge_count = node_count * node_count;

            let strategy = arb_edges_parallel(
                &Arc::new(nodes),
                directed,
                self_loops,
                0..max_edge_count,
                arb_edge);

            proptest::test_runner::Config::with_cases(100);

            for _ in 0..100 {
                let mut runner = proptest::test_runner::TestRunner::default();

                let value_tree = strategy
                    .new_tree(&mut runner)
                    .expect("failed to generate value tree");
                let endpoints = value_tree.current();

                prop_assert!(endpoints.len() <= max_edge_count);
            }
        }
    }

    proptest! {
        #[test]
        fn arbitrary_graph(node_count in 0..10_usize, directed in prop::bool::ANY, self_loops in prop::bool::ANY) {
            let max_edge_count = node_count * node_count;

            let nodes = arb_nodes(node_count);
            let edges = Arc::new(move |nodes: Arc<Vec<Node>>| {
                arb_edges_parallel(
                    &nodes,
                    directed,
                    self_loops,
                    0..max_edge_count,
                    arb_edge)
            });

            let strategy = arb_graph(nodes, &edges);

            proptest::test_runner::Config::with_cases(100);

            for _ in 0..100 {
                let mut runner = proptest::test_runner::TestRunner::default();

                let value_tree = strategy
                    .new_tree(&mut runner)
                    .expect("failed to generate value tree");
                let graph: Graph<Node, Edge, Directed, u32> = value_tree.current();

                prop_assert!(graph.node_count() == node_count);
                prop_assert!(graph.edge_count() <= max_edge_count);
            }
        }
    }

    proptest! {
        #[test]
        fn arbitrary_small_world_graph(
            node_count in 4..20_usize,
            self_loops in prop::bool::ANY,
            k_over_2 in (1..10_usize),
            beta in (0.0..1.0_f64)
        ) {
            let k_over_2 = k_over_2.min((node_count / 2) - 1);

            let nodes = arb_nodes(node_count);
            let edges = Arc::new(move |nodes: Arc<Vec<Node>>| {
                arb_watts_strogatz_edges_undirected_parallel(
                    &nodes,
                    self_loops,
                    k_over_2,
                    beta,
                    arb_edge
                )
            });

            let strategy = arb_graph(nodes, &edges);

            proptest::test_runner::Config::with_cases(100);

            for _ in 0..100 {
                let mut runner = proptest::test_runner::TestRunner::default();

                let value_tree = strategy
                    .new_tree(&mut runner)
                    .expect("failed to generate value tree");
                let graph: Graph<Node, Edge, Directed, u32> = value_tree.current();

                prop_assert!(graph.node_count() == node_count);
                prop_assert!(graph.edge_count() <= node_count * k_over_2);

                let mut node_out_degrees: HashMap<NodeIndex, usize> = HashMap::default();

                for edge_ref in graph.edge_references() {
                    *node_out_degrees.entry(edge_ref.source()).or_default() += 1;
                }

                for out_degree in node_out_degrees.into_values() {
                    prop_assert_eq!(out_degree, k_over_2);
                }
            }
        }
    }
}
