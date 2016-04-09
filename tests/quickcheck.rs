#![cfg(feature="quickcheck")]
extern crate quickcheck;
extern crate rand;
extern crate petgraph;

use rand::Rng;
use std::cmp::min;
use std::collections::HashMap;

use petgraph::{
    Graph, GraphMap, Undirected, Directed, EdgeType, Incoming, Outgoing,
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
};
use petgraph::visit::{Topo, SubTopo};
use petgraph::graph::{IndexType, node_index, edge_index, NodeIndex};
#[cfg(feature = "stable_graph")]
use petgraph::graph::stable::StableGraph;

fn prop(g: Graph<(), u32>) -> bool {
    // filter out isolated nodes
    let no_singles = g.filter_map(
        |nx, w| g.neighbors_undirected(nx).next().map(|_| w),
        |_, w| Some(w));
    for i in no_singles.node_indices() {
        assert!(no_singles.neighbors_undirected(i).count() > 0);
    }
    assert_eq!(no_singles.edge_count(), g.edge_count());
    let mst = min_spanning_tree(&no_singles);
    assert!(!is_cyclic_undirected(&mst));
    true
}

fn prop_undir(g: Graph<(), u32, Undirected>) -> bool {
    // filter out isolated nodes
    let no_singles = g.filter_map(
        |nx, w| g.neighbors_undirected(nx).next().map(|_| w),
        |_, w| Some(w));
    for i in no_singles.node_indices() {
        assert!(no_singles.neighbors_undirected(i).count() > 0);
    }
    assert_eq!(no_singles.edge_count(), g.edge_count());
    let mst = min_spanning_tree(&no_singles);
    assert!(!is_cyclic_undirected(&mst));
    true
}

#[test]
fn arbitrary() {
    quickcheck::quickcheck(prop as fn(_) -> bool);
    quickcheck::quickcheck(prop_undir as fn(_) -> bool);
}

#[test]
fn reverse_undirected() {
    fn prop<Ty: EdgeType>(g: Graph<(), (), Ty>) -> bool {
        if g.edge_count() > 30 {
            return true; // iso too slow
        }
        let mut h = g.clone();
        h.reverse();
        is_isomorphic(&g, &h)
    }
    quickcheck::quickcheck(prop as fn(Graph<_, _, Undirected>) -> bool);
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
    fn prop<Ty: EdgeType>(g: Graph<i8, i8, Ty>) -> bool {
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
    fn prop<Ty: EdgeType>(g: Graph<i16, i8, Ty>, node: u8, edge: u8) -> bool {
        let mut ng = g.clone();
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

#[test]
fn graphmap_remove() {
    fn prop(mut g: GraphMap<i8, ()>, a: i8, b: i8) -> bool {
        let contains = g.contains_edge(a, b);
        assert_eq!(contains, g.contains_edge(b, a));
        assert_eq!(g.remove_edge(a, b).is_some(), contains);
        assert!(!g.contains_edge(a, b) &&
            g.neighbors(a).find(|x| *x == b).is_none() &&
            g.neighbors(b).find(|x| *x == a).is_none());
        assert!(g.remove_edge(a, b).is_none());
        true
    }
    quickcheck::quickcheck(prop as fn(_, _, _) -> bool);
}

#[test]
fn graphmap_add_remove() {
    fn prop(mut g: GraphMap<i8, ()>, a: i8, b: i8) -> bool {
        assert_eq!(g.contains_edge(a, b), g.add_edge(a, b, ()).is_some());
        g.remove_edge(a, b);
        !g.contains_edge(a, b) &&
            g.neighbors(a).find(|x| *x == b).is_none() &&
            g.neighbors(b).find(|x| *x == a).is_none()
    }
    quickcheck::quickcheck(prop as fn(_, _, _) -> bool);
}

fn tarjan_scc(g: &Graph<(), ()>) -> Vec<Vec<NodeIndex>> {
    #[derive(Copy, Clone)]
    #[derive(Debug)]
    struct NodeData {
        index: Option<usize>,
        lowlink: usize,
        on_stack: bool,
    }
    #[derive(Debug)]
    struct Data<'a> {
        index: usize,
        nodes: Vec<NodeData>,
        stack: Vec<NodeIndex>,
        sccs: &'a mut Vec<Vec<NodeIndex>>,
    }

    let mut sccs = Vec::new();
    {
        let map = g.node_indices().map(|_| {
            NodeData { index: None, lowlink: !0, on_stack: false }
        }).collect();

        let mut data = Data {
            index: 0,
            nodes: map,
            stack: Vec::new(),
            sccs: &mut sccs,
        };
        for n in g.node_indices() {
            scc_visit(n, g, &mut data);
        }
    }
    fn scc_visit(v: NodeIndex, g: &Graph<(), ()>, data: &mut Data) {
        macro_rules! node {
            ($node:expr) => (data.nodes[$node.index()])
        }
        if node![v].index.is_some() {
            // already visited
            return;
        }
        let v_index = data.index;
        node![v].index = Some(v_index);
        node![v].lowlink = v_index;
        node![v].on_stack = true;
        data.stack.push(v);
        data.index += 1;

        for w in g.neighbors(v) {
            match node![w].index {
                None => {
                    scc_visit(w, g, data);
                    node![v].lowlink = min(node![v].lowlink, node![w].lowlink);
                }
                Some(w_index) => {
                    if node![w].on_stack {
                        // Successor w is in stack S and hence in the current SCC
                        let v_lowlink = &mut node![v].lowlink;
                        *v_lowlink = min(*v_lowlink, w_index);
                    }
                }
            }
        }

        // If v is a root node, pop the stack and generate an SCC
        if let Some(v_index) = node![v].index {
            if node![v].lowlink == v_index {
                let mut cur_scc = Vec::new();
                loop {
                    let w = data.stack.pop().unwrap();
                    node![w].on_stack = false;
                    cur_scc.push(w);
                    if w == v { break; }
                }
                data.sccs.push(cur_scc);
            }
        }
    }
    sccs
}

#[test]
fn graph_sccs() {
    fn prop(g: Graph<(), ()>) -> bool {
        let mut sccs = scc(&g);
        let mut tsccs = tarjan_scc(&g);
        // normalize sccs
        for scc in &mut sccs { scc.sort(); }
        for scc in &mut tsccs { scc.sort(); }
        sccs.sort();
        tsccs.sort();
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
    quickcheck::quickcheck(prop as fn(_) -> bool);
}

#[test]
fn graph_condensation_acyclic() {
    fn prop(g: Graph<(), ()>) -> bool {
        !is_cyclic_directed(&condensation(g, /* make_acyclic */ true))
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
        let ai = order.iter().position(|x| *x == a).unwrap();
        let bi = order.iter().position(|x| *x == b).unwrap();
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
        let order = toposort(&gr);
        is_topo_order(&gr, &order)
    }
    quickcheck::quickcheck(prop as fn(_) -> bool);
}

#[test]
fn full_topo_generic() {
    fn prop_generic(DAG(mut gr): DAG<usize>) -> bool {
        assert!(!is_cyclic_directed(&gr));
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

#[test]
fn sub_topo() {
    fn prop(DAG(mut gr): DAG<usize>) -> bool {
        if gr.node_count() == 0 {
            return true;
        }
        assert!(!is_cyclic_directed(&gr));
        let graph_index = rand::thread_rng().gen_range(0, gr.node_count());
        let graph_index = NodeIndex::new(graph_index);
        let mut sub = Graph::new();
        let sub_index = sub.add_node(graph_index);
        let mut graph_to_sub = HashMap::new();
        graph_to_sub.insert(graph_index, sub_index);
        let mut stack = vec![(graph_index, sub_index)];
        // TODO: Replace this with Bfs/Dfs that gives edges.
        while let Some((graph_index, sub_index)) = stack.pop() {
            for graph_neighbor in gr.neighbors_directed(graph_index, Outgoing) {
                if graph_to_sub.contains_key(&graph_neighbor) {
                    continue;
                }
                let sub_neighbor = sub.add_node(graph_neighbor);
                graph_to_sub.insert(graph_neighbor, sub_neighbor);
                sub.add_edge(sub_index, sub_neighbor, ());
                stack.push((graph_neighbor, sub_neighbor));
            }
        }
        let mut index = 0;
        let mut topo = SubTopo::from_node(&gr, graph_index);
        while let Some(nx) = topo.next(&gr) {
            gr[nx] = index;
            index += 1;
        }

        let mut order = Vec::new();
        index = 0;
        let mut topo = SubTopo::from_node(&gr, graph_index);
        while let Some(nx) = topo.next(&gr) {
            order.push(nx);
            assert_eq!(gr[nx], index);
            index += 1;
        }
        let mapped_order = order.iter().map(|o| *graph_to_sub.get(o).unwrap()).collect::<Vec<_>>();
        if !is_topo_order(&sub, &mapped_order) {
            println!("Subgraph for node {} is {:?} and the order for it is: {:?}", graph_index.index(), sub, order);
            return false;
        }

        {
            order.clear();
            let mut topo = SubTopo::from_node(&gr, graph_index);
            while let Some(nx) = topo.next(&gr) {
                order.push(nx);
            }
            let mapped_order = order.iter().map(|o| *graph_to_sub.get(o).unwrap()).collect::<Vec<_>>();
            if !is_topo_order(&sub, &mapped_order) {
                println!("Subgraph for node {} is {:?} and the order for it is: {:?}", graph_index.index(), sub, order);
                return false;
            }
        }
        true
    }
    quickcheck::quickcheck(prop as fn(_) -> bool);
}
