use petgraph::Graph;

// cargo run --example find_node

fn main() {
    println!("hello graph!");
    let mut g = Graph::<&str, i32>::new();

    // add nodes to graph
    let items = ["apple", "book", "cat"];
    for s in &items {
      let i = g.add_node(s);
      println!("added {:?} with index {:?}", s, i);
    }

    // maybe later you want to find the index of a node
    let index = g.node_indices().find(|i| g.node_weight(*i) == Some(&"book")).unwrap();
    println!("book index: {:?}", index);
}
