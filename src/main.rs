use petgraph::{csr::IndexType, graph6_encoder::Graph6, Graph, Undirected};

fn main() {
    println!("{}", random_graph::<u32>(2, 1.).graph6_string());
    println!("{}", random_graph::<u32>(3, 1.).graph6_string());
    println!("{}", random_graph::<u32>(5, 1.).graph6_string());
    println!("{}", random_graph::<u32>(7, 1.).graph6_string());
    println!("{}", random_graph::<u32>(2, 0.5).graph6_string());
    println!("{}", random_graph::<u32>(3, 0.5).graph6_string());
    println!("{}", random_graph::<u32>(5, 0.5).graph6_string());
    println!("{}", random_graph::<u32>(7, 0.5).graph6_string());
    println!("2_000");
    println!("{}", random_graph::<u32>(2_000, 0.3).graph6_string());
    println!("6_000");
    println!("{}", random_graph::<u32>(6_000, 0.3).graph6_string());
    println!("10_000");
    println!("{}", random_graph::<u32>(10_000, 0.3).graph6_string());
    println!("30_000");
    println!("{}", random_graph::<u32>(30_000, 0.3).graph6_string());
    println!("50_000");
    println!("{}", random_graph::<u32>(50_000, 0.5).graph6_string());
    println!("258_047");
    println!("{}", random_graph::<u32>(258_047, 0.5).graph6_string());
}

pub fn random_graph<Ix: IndexType>(order: usize, p: f64) -> Graph<(), (), Undirected, Ix> {
    let mut graph = Graph::with_capacity(order, 0);

    let mut nodes = vec![];
    for _ in 0..order {
        let node = graph.add_node(());
        nodes.push(node);
    }

    for u in 0..order {
        for v in (u + 1)..order {
            if rand::random::<f64>() <= p {
                graph.add_edge(nodes[u], nodes[v], ());
            }
        }
    }

    graph
}
