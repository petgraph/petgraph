#![cfg(feature="quickcheck")]
#[macro_use] extern crate quickcheck;
extern crate rand;
extern crate petgraph;

extern crate odds;

mod utils;

use utils::Small;

use odds::prelude::*;

use rand::Rng;

use petgraph::prelude::*;
use petgraph::{
    EdgeType, 
};
use petgraph::dot::{Dot, Config};
use petgraph::algo::{
    condensation,
    min_spanning_tree,
    is_cyclic_undirected,
    is_cyclic_directed,
    is_isomorphic,
    is_isomorphic_matching,
    toposort,
    scc,
    tarjan_scc,
    dijkstra,
};
use petgraph::visit::{Topo, Reversed};
use petgraph::data::FromElements;
use petgraph::graph::{IndexType, node_index, edge_index};
use petgraph::graphmap::{
    NodeTrait,
};

fn mst_graph<N, E, Ty, Ix>(g: &Graph<N, E, Ty, Ix>) -> Graph<N, E, Undirected, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
          N: Clone, E: Clone + PartialOrd,
{
    Graph::from_elements(min_spanning_tree(&g))
}

use std::fmt;

quickcheck! {
    fn mst_directed(g: Small<Graph<(), u32>>) -> bool {
        // filter out isolated nodes
        let no_singles = g.filter_map(
            |nx, w| g.neighbors_undirected(nx).next().map(|_| w),
            |_, w| Some(w));
        for i in no_singles.node_indices() {
            assert!(no_singles.neighbors_undirected(i).count() > 0);
        }
        assert_eq!(no_singles.edge_count(), g.edge_count());
        let mst = mst_graph(&no_singles);
        assert!(!is_cyclic_undirected(&mst));
        true
    }
}

quickcheck! {
    fn mst_undirected(g: Graph<(), u32, Undirected>) -> bool {
        // filter out isolated nodes
        let no_singles = g.filter_map(
            |nx, w| g.neighbors_undirected(nx).next().map(|_| w),
            |_, w| Some(w));
        for i in no_singles.node_indices() {
            assert!(no_singles.neighbors_undirected(i).count() > 0);
        }
        assert_eq!(no_singles.edge_count(), g.edge_count());
        let mst = mst_graph(&no_singles);
        assert!(!is_cyclic_undirected(&mst));
        true
    }
}

quickcheck! {
    fn reverse_undirected(g: Small<UnGraph<(), ()>>) -> bool {
        let mut h = (*g).clone();
        h.reverse();
        is_isomorphic(&g, &h)
    }
}

fn assert_graph_consistent<N, E, Ty, Ix>(g: &Graph<N, E, Ty, Ix>)
    where Ty: EdgeType,
          Ix: IndexType,
{
    assert_eq!(g.node_count(), g.node_indices().count());
    assert_eq!(g.edge_count(), g.edge_indices().count());
    for edge in g.raw_edges() {
        assert!(g.find_edge(edge.source(), edge.target()).is_some(),
                "Edge not in graph! {:?} to {:?}", edge.source(), edge.target());
    }
}

#[test]
fn reverse_directed() {
    fn prop<Ty: EdgeType>(mut g: Graph<(), (), Ty>) -> bool {
        let node_outdegrees = g.node_indices()
                                .map(|i| g.neighbors_directed(i, Outgoing).count())
                                .collect::<Vec<_>>();
        let node_indegrees = g.node_indices()
                                .map(|i| g.neighbors_directed(i, Incoming).count())
                                .collect::<Vec<_>>();

        g.reverse();
        let new_outdegrees = g.node_indices()
                                .map(|i| g.neighbors_directed(i, Outgoing).count())
                                .collect::<Vec<_>>();
        let new_indegrees = g.node_indices()
                                .map(|i| g.neighbors_directed(i, Incoming).count())
                                .collect::<Vec<_>>();
        assert_eq!(node_outdegrees, new_indegrees);
        assert_eq!(node_indegrees, new_outdegrees);
        assert_graph_consistent(&g);
        true
    }
    quickcheck::quickcheck(prop as fn(Graph<_, _, Directed>) -> bool);
}

#[test]
fn retain_nodes() {
    fn prop<Ty: EdgeType>(mut g: Graph<i32, i32, Ty>) -> bool {
        // Remove all negative nodes, these should be randomly spread
        let og = g.clone();
        let nodes = g.node_count();
        let num_negs = g.raw_nodes().iter().filter(|n| n.weight < 0).count();
        let mut removed = 0;
        g.retain_nodes(|g, i| {
            let keep = g[i] >= 0;
            if !keep {
                removed += 1;
            }
            keep
        });
        let num_negs_post = g.raw_nodes().iter().filter(|n| n.weight < 0).count();
        let num_pos_post = g.raw_nodes().iter().filter(|n| n.weight >= 0).count();
        assert_eq!(num_negs_post, 0);
        assert_eq!(removed, num_negs);
        assert_eq!(num_negs + g.node_count(), nodes);
        assert_eq!(num_pos_post, g.node_count());

        // check against filter_map
        let filtered = og.filter_map(|_, w| if *w >= 0 { Some(*w) } else { None },
                                     |_, w| Some(*w));
        assert_eq!(g.node_count(), filtered.node_count());
        /*
        println!("Iso of graph with nodes={}, edges={}",
                 g.node_count(), g.edge_count());
                 */
        assert!(is_isomorphic_matching(&filtered, &g, PartialEq::eq, PartialEq::eq));

        true
    }
    quickcheck::quickcheck(prop as fn(Graph<_, _, Directed>) -> bool);
    quickcheck::quickcheck(prop as fn(Graph<_, _, Undirected>) -> bool);
}

#[test]
fn retain_edges() {
    fn prop<Ty: EdgeType>(mut g: Graph<(), i32, Ty>) -> bool {
        // Remove all negative edges, these should be randomly spread
        let og = g.clone();
        let edges = g.edge_count();
        let num_negs = g.raw_edges().iter().filter(|n| n.weight < 0).count();
        let mut removed = 0;
        g.retain_edges(|g, i| {
            let keep = g[i] >= 0;
            if !keep {
                removed += 1;
            }
            keep
        });
        let num_negs_post = g.raw_edges().iter().filter(|n| n.weight < 0).count();
        let num_pos_post = g.raw_edges().iter().filter(|n| n.weight >= 0).count();
        assert_eq!(num_negs_post, 0);
        assert_eq!(removed, num_negs);
        assert_eq!(num_negs + g.edge_count(), edges);
        assert_eq!(num_pos_post, g.edge_count());
        if og.edge_count() < 30 {
            // check against filter_map
            let filtered = og.filter_map(
                |_, w| Some(*w),
                |_, w| if *w >= 0 { Some(*w) } else { None });
            assert_eq!(g.node_count(), filtered.node_count());
            assert!(is_isomorphic(&filtered, &g));
        }
        true
    }
    quickcheck::quickcheck(prop as fn(Graph<_, _, Directed>) -> bool);
    quickcheck::quickcheck(prop as fn(Graph<_, _, Undirected>) -> bool);
}

#[test]
fn isomorphism_1() {
    // using small weights so that duplicates are likely
    fn prop<Ty: EdgeType>(g: Small<Graph<i8, i8, Ty>>) -> bool {
        let mut rng = rand::thread_rng();
        // several trials of different isomorphisms of the same graph
        // mapping of node indices
        let mut map = g.node_indices().collect::<Vec<_>>();
        let mut ng = Graph::<_, _, Ty>::with_capacity(g.node_count(), g.edge_count());
        for _ in 0..1 {
            rng.shuffle(&mut map);
            ng.clear();

            for _ in g.node_indices() {
                ng.add_node(0);
            }
            // Assign node weights
            for i in g.node_indices() {
                ng[map[i.index()]] = g[i];
            }
            // Add edges
            for i in g.edge_indices() {
                let (s, t) = g.edge_endpoints(i).unwrap();
                ng.add_edge(map[s.index()],
                            map[t.index()],
                            g[i]);
            }
            if g.node_count() < 20 && g.edge_count() < 50 {
                assert!(is_isomorphic(&g, &ng));
            }
            assert!(is_isomorphic_matching(&g, &ng, PartialEq::eq, PartialEq::eq));
        }
        true
    }
    quickcheck::quickcheck(prop::<Undirected> as fn(_) -> bool);
    quickcheck::quickcheck(prop::<Directed> as fn(_) -> bool);
}

#[test]
fn isomorphism_modify() {
    // using small weights so that duplicates are likely
    fn prop<Ty: EdgeType>(g: Small<Graph<i16, i8, Ty>>, node: u8, edge: u8) -> bool {
        println!("graph {:#?}", g);
        let mut ng = (*g).clone();
        let i = node_index(node as usize);
        let j = edge_index(edge as usize);
        if i.index() < g.node_count() {
            ng[i] = (g[i] == 0) as i16;
        }
        if j.index() < g.edge_count() {
            ng[j] = (g[j] == 0) as i8;
        }
        if i.index() < g.node_count() || j.index() < g.edge_count() {
            assert!(!is_isomorphic_matching(&g, &ng, PartialEq::eq, PartialEq::eq));
        } else {
            assert!(is_isomorphic_matching(&g, &ng, PartialEq::eq, PartialEq::eq));
        }
        true
    }
    quickcheck::quickcheck(prop::<Undirected> as fn(_, _, _) -> bool);
    quickcheck::quickcheck(prop::<Directed> as fn(_, _, _) -> bool);
}

#[test]
fn graph_remove_edge() {
    fn prop<Ty: EdgeType>(mut g: Graph<(), (), Ty>, a: u8, b: u8) -> bool {
        let a = node_index(a as usize);
        let b = node_index(b as usize);
        let edge = g.find_edge(a, b);
        if !g.is_directed() {
            assert_eq!(edge.is_some(), g.find_edge(b, a).is_some());
        }
        if let Some(ex) = edge {
            assert!(g.remove_edge(ex).is_some());
        }
        assert_graph_consistent(&g);
        assert!(g.find_edge(a, b).is_none());
        assert!(g.neighbors(a).find(|x| *x == b).is_none());
        if !g.is_directed() {
            assert!(g.neighbors(b).find(|x| *x == a).is_none());
        }
        true
    }
    quickcheck::quickcheck(prop as fn(Graph<_, _, Undirected>, _, _) -> bool);
    quickcheck::quickcheck(prop as fn(Graph<_, _, Directed>, _, _) -> bool);
}

#[cfg(feature = "stable_graph")]
#[test]
fn stable_graph_remove_edge() {
    fn prop<Ty: EdgeType>(mut g: StableGraph<(), (), Ty>, a: u8, b: u8) -> bool {
        let a = node_index(a as usize);
        let b = node_index(b as usize);
        let edge = g.find_edge(a, b);
        if !g.is_directed() {
            assert_eq!(edge.is_some(), g.find_edge(b, a).is_some());
        }
        if let Some(ex) = edge {
            assert!(g.remove_edge(ex).is_some());
        }
        //assert_graph_consistent(&g);
        assert!(g.find_edge(a, b).is_none());
        assert!(g.neighbors(a).find(|x| *x == b).is_none());
        if !g.is_directed() {
            assert!(g.find_edge(b, a).is_none());
            assert!(g.neighbors(b).find(|x| *x == a).is_none());
        }
        true
    }
    quickcheck::quickcheck(prop as fn(StableGraph<_, _, Undirected>, _, _) -> bool);
    quickcheck::quickcheck(prop as fn(StableGraph<_, _, Directed>, _, _) -> bool);
}

#[cfg(feature = "stable_graph")]
#[test]
fn stable_graph_add_remove_edges() {
    fn prop<Ty: EdgeType>(mut g: StableGraph<(), (), Ty>, edges: Vec<(u8, u8)>) -> bool {
        for &(a, b) in &edges {
            let a = node_index(a as usize);
            let b = node_index(b as usize);
            let edge = g.find_edge(a, b);

            if edge.is_none() && g.contains_node(a) && g.contains_node(b) {
                let _index = g.add_edge(a, b, ());
                continue;
            }

            if !g.is_directed() {
                assert_eq!(edge.is_some(), g.find_edge(b, a).is_some());
            }
            if let Some(ex) = edge {
                assert!(g.remove_edge(ex).is_some());
            }
            //assert_graph_consistent(&g);
            assert!(g.find_edge(a, b).is_none(), "failed to remove edge {:?} from graph {:?}", (a, b), g);
            assert!(g.neighbors(a).find(|x| *x == b).is_none());
            if !g.is_directed() {
                assert!(g.find_edge(b, a).is_none());
                assert!(g.neighbors(b).find(|x| *x == a).is_none());
            }
        }
        true
    }
    quickcheck::quickcheck(prop as fn(StableGraph<_, _, Undirected>, _) -> bool);
    quickcheck::quickcheck(prop as fn(StableGraph<_, _, Directed>, _) -> bool);
}

fn assert_graphmap_consistent<N, E, Ty>(g: &GraphMap<N, E, Ty>)
    where Ty: EdgeType,
          N: NodeTrait + fmt::Debug,
{
    for (a, b, _weight) in g.all_edges() {
        assert!(g.contains_edge(a, b),
                "Edge not in graph! {:?} to {:?}", a, b);
        assert!(g.neighbors(a).find(|x| *x == b).is_some(),
                "Edge {:?} not in neighbor list for {:?}", (a, b), a);
        if !g.is_directed() {
            assert!(g.neighbors(b).find(|x| *x == a).is_some(),
                    "Edge {:?} not in neighbor list for {:?}", (b, a), b);
        }
    }
}

#[test]
fn graphmap_remove() {
    fn prop<Ty: EdgeType>(mut g: GraphMap<i8, (), Ty>, a: i8, b: i8) -> bool {
        //if g.edge_count() > 20 { return true; }
        assert_graphmap_consistent(&g);
        let contains = g.contains_edge(a, b);
        if !g.is_directed() {
            assert_eq!(contains, g.contains_edge(b, a));
        }
        assert_eq!(g.remove_edge(a, b).is_some(), contains);
        assert!(!g.contains_edge(a, b) &&
            g.neighbors(a).find(|x| *x == b).is_none());
            //(g.is_directed() || g.neighbors(b).find(|x| *x == a).is_none()));
        assert!(g.remove_edge(a, b).is_none());
        assert_graphmap_consistent(&g);
        true
    }
    quickcheck::quickcheck(prop as fn(DiGraphMap<_, _>, _, _) -> bool);
    quickcheck::quickcheck(prop as fn(UnGraphMap<_, _>, _, _) -> bool);
}

#[test]
fn graphmap_add_remove() {
    fn prop(mut g: UnGraphMap<i8, ()>, a: i8, b: i8) -> bool {
        assert_eq!(g.contains_edge(a, b), g.add_edge(a, b, ()).is_some());
        g.remove_edge(a, b);
        !g.contains_edge(a, b) &&
            g.neighbors(a).find(|x| *x == b).is_none() &&
            g.neighbors(b).find(|x| *x == a).is_none()
    }
    quickcheck::quickcheck(prop as fn(_, _, _) -> bool);
}

fn sort_sccs<T: Ord>(v: &mut [Vec<T>]) {
    for scc in &mut *v {
        scc.sort();
    }
    v.sort();
}

quickcheck! {
    fn graph_sccs(g: Graph<(), ()>) -> bool {
        let mut sccs = scc(&g);
        let mut tsccs = tarjan_scc(&g);
        sort_sccs(&mut sccs);
        sort_sccs(&mut tsccs);
        if sccs != tsccs {
            println!("{:?}",
                     Dot::with_config(&g, &[Config::EdgeNoLabel,
                                      Config::NodeIndexLabel]));
            println!("Sccs {:?}", sccs);
            println!("Sccs (Tarjan) {:?}", tsccs);
            return false;
        }
        true
    }
}

quickcheck! {
    fn kosaraju_scc_is_topo_sort(g: Graph<(), ()>) -> bool {
        let tsccs = scc(&g);
        let firsts = vec(tsccs.iter().rev().map(|v| v[0]));
        subset_is_topo_order(&g, &firsts)
    }
}

quickcheck! {
    fn tarjan_scc_is_topo_sort(g: Graph<(), ()>) -> bool {
        let tsccs = tarjan_scc(&g);
        let firsts = vec(tsccs.iter().rev().map(|v| v[0]));
        subset_is_topo_order(&g, &firsts)
    }
}


quickcheck! {
    // Reversed edges gives the same sccs (when sorted)
    fn graph_reverse_sccs(g: Graph<(), ()>) -> bool {
        let mut sccs = scc(&g);
        let mut tsccs = scc(Reversed(&g));
        sort_sccs(&mut sccs);
        sort_sccs(&mut tsccs);
        if sccs != tsccs {
            println!("{:?}",
                     Dot::with_config(&g, &[Config::EdgeNoLabel,
                                      Config::NodeIndexLabel]));
            println!("Sccs {:?}", sccs);
            println!("Sccs (Reversed) {:?}", tsccs);
            return false;
        }
        true
    }
}

quickcheck! {
    // Reversed edges gives the same sccs (when sorted)
    fn graphmap_reverse_sccs(g: DiGraphMap<u16, ()>) -> bool {
        let mut sccs = scc(&g);
        let mut tsccs = scc(Reversed(&g));
        sort_sccs(&mut sccs);
        sort_sccs(&mut tsccs);
        if sccs != tsccs {
            println!("{:?}",
                     Dot::with_config(&g, &[Config::EdgeNoLabel,
                                      Config::NodeIndexLabel]));
            println!("Sccs {:?}", sccs);
            println!("Sccs (Reversed) {:?}", tsccs);
            return false;
        }
        true
    }
}

#[test]
fn graph_condensation_acyclic() {
    fn prop(g: Graph<(), ()>) -> bool {
        !is_cyclic_directed(&condensation(g, /* make_acyclic */ true), None)
    }
    quickcheck::quickcheck(prop as fn(_) -> bool);
}

#[derive(Debug, Clone)]
struct DAG<N: Default + Clone + Send + 'static>(Graph<N, ()>);

impl<N: Default + Clone + Send + 'static> quickcheck::Arbitrary for DAG<N> {
    fn arbitrary<G: quickcheck::Gen>(g: &mut G) -> Self {
        let nodes = usize::arbitrary(g);
        if nodes == 0 {
            return DAG(Graph::with_capacity(0, 0));
        }
        let split = g.gen_range(0., 1.);
        let max_width = f64::sqrt(nodes as f64) as usize;
        let tall = (max_width as f64 * split) as usize;
        let fat = max_width - tall;

        let edge_prob = 1. - (1. - g.gen_range(0., 1.)) * (1. - g.gen_range(0., 1.));
        let edges = ((nodes as f64).powi(2) * edge_prob) as usize;
        let mut gr = Graph::with_capacity(nodes, edges);
        let mut nodes = 0;
        for _ in 0..tall {
            let cur_nodes = g.gen_range(0, fat);
            for _ in 0..cur_nodes {
                gr.add_node(N::default());
            }
            for j in 0..nodes {
                for k in 0..cur_nodes {
                    if g.gen_range(0., 1.) < edge_prob {
                        gr.add_edge(NodeIndex::new(j), NodeIndex::new(k + nodes), ());
                    }
                }
            }
            nodes += cur_nodes;
        }
        DAG(gr)
    }

    // shrink the graph by splitting it in two by a very
    // simple algorithm, just even and odd node indices
    fn shrink(&self) -> Box<Iterator<Item=Self>> {
        let self_ = self.clone();
        Box::new((0..2).filter_map(move |x| {
            let gr = self_.0.filter_map(|i, w| {
                if i.index() % 2 == x {
                    Some(w.clone())
                } else {
                    None
                }
            },
            |_, w| Some(w.clone())
            );
            // make sure we shrink
            if gr.node_count() < self_.0.node_count() {
                Some(DAG(gr))
            } else {
                None
            }
        }))
    }
}

fn is_topo_order<N>(gr: &Graph<N, (), Directed>, order: &[NodeIndex]) -> bool {
    if gr.node_count() != order.len() {
        println!("Graph ({}) and count ({}) had different amount of nodes.", gr.node_count(), order.len());
        return false;
    }
    // check all the edges of the graph
    for edge in gr.raw_edges() {
        let a = edge.source();
        let b = edge.target();
        let ai = order.find(&a).unwrap();
        let bi = order.find(&b).unwrap();
        if ai >= bi {
            println!("{:?} > {:?} ", a, b);
            return false;
        }
    }
    true
}


fn subset_is_topo_order<N>(gr: &Graph<N, (), Directed>, order: &[NodeIndex]) -> bool {
    if gr.node_count() < order.len() {
        println!("Graph (len={}) had less nodes than order (len={})", gr.node_count(), order.len());
        return false;
    }
    // check all the edges of the graph
    for edge in gr.raw_edges() {
        let a = edge.source();
        let b = edge.target();
        if a == b {
            continue;
        }
        // skip those that are not in the subset
        let ai = match order.find(&a) {
            Some(i) => i,
            None => continue,
        };
        let bi = match order.find(&b) {
            Some(i) => i,
            None => continue,
        };
        if ai >= bi {
            println!("{:?} > {:?} ", a, b);
            return false;
        }
    }
    true
}

#[test]
fn full_topo() {
    fn prop(DAG(gr): DAG<()>) -> bool {
        let order = toposort(&gr, None);
        is_topo_order(&gr, &order)
    }
    quickcheck::quickcheck(prop as fn(_) -> bool);
}

#[test]
fn full_topo_generic() {
    fn prop_generic(DAG(mut gr): DAG<usize>) -> bool {
        assert!(!is_cyclic_directed(&gr, None));
        let mut index = 0;
        let mut topo = Topo::new(&gr);
        while let Some(nx) = topo.next(&gr) {
            gr[nx] = index;
            index += 1;
        }

        let mut order = Vec::new();
        index = 0;
        let mut topo = Topo::new(&gr);
        while let Some(nx) = topo.next(&gr) {
            order.push(nx);
            assert_eq!(gr[nx], index);
            index += 1;
        }
        if !is_topo_order(&gr, &order) {
            println!("{:?}", gr);
            return false;
        }

        {
            order.clear();
            let mut topo = Topo::new(&gr);
            while let Some(nx) = topo.next(&gr) {
                order.push(nx);
            }
            if !is_topo_order(&gr, &order) {
                println!("{:?}", gr);
                return false;
            }
        }
        true
    }
    quickcheck::quickcheck(prop_generic as fn(_) -> bool);
}

quickcheck! {
    // checks that the distances computed by dijkstra satisfy the triangle
    // inequality.
    fn dijkstra_triangle_ineq(g: Graph<u32, u32>, node: usize) -> bool {
        if g.node_count() == 0 {
            return true;
        }
        let v = node_index(node % g.node_count());
        let distances = dijkstra(
            &g,
            v,
            None,
            |g, v| g.edges(v).map(|(v, &e)| (v, e))
        );
        for v2 in distances.keys() {
            let dv2 = distances[v2];
            // triangle inequality:
            // d(v,u) <= d(v,v2) + w(v2,u)
            for (u,w) in g.edges(*v2) {
                if distances.contains_key(&u) && distances[&u] > dv2 + w {
                    return false;
                }
            }
        }
        true
    }
}
