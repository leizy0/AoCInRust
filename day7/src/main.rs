#[macro_use]
extern crate lazy_static;
extern crate regex;

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::{HashMap, HashSet};
use regex::Regex;

fn main() {
    let input_path = "./input.txt";
    let input_file = File::open(input_path).expect(&format!("Failed to open input file({})", input_path));
    let input_list: Vec<Edge> = BufReader::new(input_file).lines().map(|l| Edge::new(&l.unwrap()).unwrap()).collect();

    // Construct the whole dependency graph
    let mut input_graph = Graph::new();
    input_list.iter().for_each(|e| input_graph.add_edge(e));

    // From start nodes(without in nodes), generate work sequence


    
}

type Node = String;

struct Edge{
    from: Node,
    to: Node
}

impl Edge {
    pub fn new(desc: &str) -> Option<Edge> {
        lazy_static! {
            // Step W must be finished before step B can begin.
            static ref EDGE_PATTERN: Regex = Regex::new(r"Step (\w+) must be finished before step (\w+) can begin.").unwrap();
        }

        match EDGE_PATTERN.captures(desc) {
            Some(caps) => Some(Edge {
                from: caps.get(1).unwrap().as_str().to_string(),
                to: caps.get(2).unwrap().as_str().to_string()
            }),
            _ => None
        }
    }

    pub fn from_node(&self) -> &Node {
        &self.from
    }

    pub fn to_node(&self) -> &Node {
        &self.to
    }
}

struct InOutList {
    in_node_ids: HashSet<Node>,
    out_node_ids: HashSet<Node>
}

impl InOutList {
    pub fn new() -> InOutList {
        InOutList {
            in_node_ids: HashSet::new(),
            out_node_ids: HashSet::new()
        }
    }

    pub fn add_in_node(&mut self, node: &Node) {
        self.in_node_ids.insert(node.to_string());
    }

    pub fn add_out_node(&mut self, node: &Node) {
        self.out_node_ids.insert(node.to_string());
    }
}

struct Graph {
    edge_map: HashMap<Node, InOutList>
}

impl Graph {
    pub fn new() -> Graph {
        Graph {
            edge_map: HashMap::new()
        }
    }

    pub fn add_edge(&mut self, edge: &Edge) {
        let mut entry = self.edge_map.entry(edge.from_node().to_string()).or_insert(InOutList::new());
        entry.add_out_node(edge.from_node());

        entry = self.edge_map.entry(edge.to_node().to_string()).or_insert(InOutList::new());
        entry.add_in_node(edge.to_node());
    }
}