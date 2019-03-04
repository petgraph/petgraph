

use graph_impl::{DiGraph};
use graph_impl::{IndexType};
use ::Direction::Incoming;
use ::Direction::Outgoing;

use itertools::Itertools;

#[derive(Debug)]
pub enum TopologySorting<'a, N> {
    Cyclic,
    Ordered(Vec<&'a N>)
}

pub trait Topology<'a, N> {
    fn get_ordered_list(&'a self) -> TopologySorting<'a, N>;
}

impl<'a, N, E, Ix> Topology<'a, N> for DiGraph<N, E, Ix>
    where Ix: IndexType
{
    fn get_ordered_list(&'a self) -> TopologySorting<'a, N> {

        let mut graph = self.map(
            |_, weight| weight,
            |_, weight| weight,
        );

        let mut l_set = vec![];
        let mut s_set = graph.externals(Incoming).collect_vec();

        while !s_set.is_empty() {
            let node = s_set.remove(0);
            let no = &node;
            l_set.push(node);
            let g = graph.map(
                |_, weight| *weight,
                |_, weight| *weight,
            );
            let neighbors = g.neighbors_directed(*no, Outgoing);
            for neighbor in neighbors {
                let edge = graph.find_edge(*no, neighbor).unwrap();
                graph.remove_edge(edge);
                let incoming = graph.edges_directed(neighbor, Incoming);
                if incoming.count() == 0 {
                    s_set.push(neighbor);
                }
            }
        }
        if graph.edge_count() == 0 {
            let out: Vec<&'a N> = l_set.iter()
                .map(|ix| *graph.node_weight(*ix).unwrap())
                .collect();
            TopologySorting::Ordered(out)
        }
        else {
            TopologySorting::Cyclic
        }
    }
}