//! This example is taken from Advent of Code (Day 7)
//!
//! As input it takes day7.txt, an instruction list. The
//! instructions are connected in a graph and we use topological sort
//! to visit them in dependencies first order.
extern crate petgraph;

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

use petgraph::{
    Incoming,
};
use petgraph::visit::Topo;
use petgraph::graph::{
    Graph,
    NodeIndex,
};
use petgraph::dot::{Dot, Config};
use petgraph::builder::GraphBuilder;

// this is the scalar we compute by
type K = u16;

use Oper::*;
#[derive(Copy, Clone, Debug, PartialEq)]
enum Oper {
    Lit(K),
    And,
    Or,
    Lshift(K),
    Rshift(K),
    Not,
    Propagate, // No-op
    Unassigned,
}

impl Oper {
    fn input_count(&self) -> usize {
        match *self {
            Lit(_) => 0,
            Not => 1,
            And => 2,
            Or => 2,
            Lshift(_) => 1,
            Rshift(_) => 1,
            Propagate => 1,
            Unassigned => 0,
        }
    }

    /// Given `input_values`, compute the output of this operation.
    ///
    /// **Panics** if `self` is Unassigned or if `input_values` is too short.
    fn compute(&self, input_values: &[K]) -> K {
        match *self {
            Lit(x) => x,
            Not => !input_values[0],
            And => input_values[0] & input_values[1],
            Or => input_values[0] | input_values[1],
            Lshift(x) => input_values[0] << x,
            Rshift(x) => input_values[0] >> x,
            Propagate => input_values[0],
            Unassigned => unreachable!(),
        }
    }
}

type OpGraph<'a> = Graph<(&'a str, Oper, K), ()>;

static INPUT: &'static str = include_str!("day7.txt");
static _INPUT: &'static str = r#"
123 -> x
456 -> y
x AND y -> d
x OR y -> e
x LSHIFT 2 -> f
y RSHIFT 2 -> g
NOT x -> h
NOT y -> i"#;

macro_rules! try_unwrap(
    ($e:expr => $err:expr) => {
        match $e {
            Some(_value) => _value,
            None => return Err($err),
        }
    };
);

fn parse_graph(input_text: &str)
    -> Result<(HashMap<&str, NodeIndex>, OpGraph), &'static str>
{
    let mut g = GraphBuilder::from(OpGraph::with_capacity(0, 0));
    let mut part1_words = Vec::new();
    for line in input_text.trim().lines() {
        if line.trim().is_empty() {
            continue;
        }
        let mut parts = line.split("->");
        let part1 = try_unwrap!(parts.next() => "syntax error").trim();
        part1_words.clear();
        part1_words.extend(part1.split_whitespace());
        let part2 = try_unwrap!(parts.next() => "syntax error").trim();

        let op;

        let mut input_names = ["", ""];
        // parse operation and add inputs
        if part1_words.len() == 1 {
            input_names[0] = part1_words[0];
            op = Propagate;
        } else if part1_words[0] == "NOT" {
            input_names[0] = part1_words[1];
            op = Not;
        } else if part1_words[1] == "RSHIFT" {
            input_names[0] = part1_words[0];
            op = Rshift(try_unwrap!(part1_words[2].parse::<K>().ok()
                                    => "failed to parse Rshift operand"));
        } else if part1_words[1] == "LSHIFT" {
            input_names[0] = part1_words[0];
            op = Lshift(try_unwrap!(part1_words[2].parse::<K>().ok()
                                    => "failed to parse Lshift operand"));
        } else {
            input_names[0] = part1_words[0];
            input_names[1] = part1_words[2];
            op = match part1_words[1] {
                "AND" => And,
                "OR"=> Or,
                _ => return Err("parsing error: unexpected operation"),
            };
        }

        let mut inputs = [NodeIndex::end(), NodeIndex::end()];

        let output = {
            let mut mknod = |name| {
                let name: &str = name;
                if let Ok(x) = name.parse::<K>() {
                    // it's a literal -- make a literal node for it
                    g.ensure_node(name, (name, Lit(x), 0))
                } else {
                    // make a placeholder node for it
                    g.ensure_node(name, (name, Unassigned, 0))
                }
            };

            // add input nodes
            for i in 0..op.input_count() {
                inputs[i] = mknod(input_names[i]);
            }

            // Add the operation node
            mknod(part2)
        };

        if g[output].1 != Unassigned {
            return Err("parsing error: duplicate output");
        }
        g[output].1 = op;

        // Add edges from input to operation
        for &input in &inputs[..op.input_count()] {
            assert!(input != NodeIndex::end());
            g.update_edge(input, output, ());
        }
    }
    Ok(g.into_inner())
}

fn compute_graph(g: &mut OpGraph) {
    // use toposort to walk the graph in dependencies first order
    let mut topo = Topo::new(g);
    while let Some(nx) = topo.next(g) {
        let mut input_values = [!0, !0];
        for (i, n) in g.neighbors_directed(nx, Incoming).enumerate() {
            input_values[i] = g[n].2;
        }
        let (ref _name, ref oper, ref mut value) = g[nx];
        *value = oper.compute(&input_values);

        /*
        println!("Visited {:?}\t{:?} {:?}",
                 _name, oper, &input_values[..oper.input_count()]);
        */
    }
}

fn compute_part2(g: &mut OpGraph, map: &HashMap<&str, NodeIndex>) {
    // take the value in node a, and put it in node b (a literal) and run again
    let a_value = g[map["a"]].2;
    println!("Value in a is {:?}", a_value);
    g[map["b"]].1 = Lit(a_value);
    compute_graph(g);
    let a_value = g[map["a"]].2;
    println!("Value in a is {:?}", a_value);
}

fn main() {
    let (map, mut g) = parse_graph(INPUT).unwrap();
    compute_graph(&mut g);
    let mut output_dot = File::create("day7.dot").unwrap();
    writeln!(&mut output_dot, "{:?}",
             Dot::with_config(&g, &[Config::EdgeNoLabel])).unwrap();
    compute_part2(&mut g, &map);
}
