#[macro_use]
extern crate lazy_static;
extern crate regex;

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::{HashMap, HashSet, BTreeSet};
use regex::Regex;

fn main() {
    let input_path = "./input.txt";
    let input_file = File::open(input_path).expect(&format!("Failed to open input file({})", input_path));
    let input_list: Vec<Edge> = BufReader::new(input_file).lines().map(|l| Edge::new(&l.unwrap()).unwrap()).collect();
    // println!("Input edges are: {:?}", input_list);

    // Construct the whole dependency graph
    let mut dep_graph = Graph::new();
    input_list.iter().for_each(|e| dep_graph.add_edge(e));
    // println!("Dependence graph is: {:?}", dep_graph);

    // From start nodes(without in nodes), generate work sequence
    let work_seq = DepSolver::new(dep_graph).fold(String::new(), |mut res, step| {
        res.push_str(&step); 
        res
    });

    println!("Given the dependences, final work flow is {}", work_seq);
}

type Node = String;

#[derive(Debug)]
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

#[derive(Debug)]
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

    pub fn in_count(&self) -> u32 {
        self.in_node_ids.len() as u32
    }

    pub fn in_nodes(&self) -> HashSet<Node> {
        self.in_node_ids.clone()
    }

    pub fn out_nodes(&self) -> HashSet<Node> {
        self.out_node_ids.clone()
    }
}

#[derive(Debug)]
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
        let mut entry = self.edge_map.entry(Node::from(edge.from_node())).or_insert(InOutList::new());
        entry.add_out_node(edge.to_node());

        entry = self.edge_map.entry(Node::from(edge.to_node())).or_insert(InOutList::new());
        entry.add_in_node(edge.from_node());
    }

    pub fn nodes(&self) -> &HashMap<Node, InOutList> {
        &self.edge_map
    }

    pub fn in_nodes(&self, node: &Node) -> HashSet<Node> {
        self.edge_map.get(node).unwrap().in_nodes()
    }

    pub fn out_nodes(& self, node: &Node) -> HashSet<Node> {
        self.edge_map.get(node).unwrap().out_nodes()
    }
}

struct DepSolver {
    dep_graph: Graph,
    candidates: BTreeSet<String>,
    done_hist: HashSet<String>
}

impl DepSolver {
    pub fn new(graph: Graph) -> DepSolver {
        let start_nodes = graph.nodes().iter().filter(|(_, io_list)| io_list.in_count() == 0).map(
            |(id, _)| id.to_string()).collect();

        DepSolver {
            dep_graph: graph,
            candidates: start_nodes,
            done_hist: HashSet::new()
        }
    }
}

impl Iterator for DepSolver {
    type Item = Node;
    fn next(&mut self) -> Option<Node> {
        // println!("Candidates: {:?}, done_hist: {:?}", self.candidates, self.done_hist);

        let done = match self.candidates.iter().next() {
            None => return None,
            Some(cand) => cand.to_string()
        };
        self.candidates.remove(&done);

        self.done_hist.insert(done.clone());
        let affect_nodes = self.dep_graph.out_nodes(&done);
        for node in affect_nodes {
            if !self.done_hist.contains(&node) {
                let need_nodes = self.dep_graph.in_nodes(&node);
                // println!("node {} need nodes: {:?}", node, need_nodes);
                if need_nodes.is_subset(&self.done_hist) {
                    self.candidates.insert(node);
                }
            }
        }

        Some(done)
    }
}