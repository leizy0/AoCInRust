#[macro_use]
extern crate lazy_static;
extern crate regex;

use regex::Regex;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::cmp::Ordering;

fn main() {
    let input_path = "./input.txt";
    let input_file =
        File::open(input_path).expect(&format!("Failed to open input file({})", input_path));
    let input_list: Vec<Edge> = BufReader::new(input_file)
        .lines()
        .map(|l| Edge::new(&l.unwrap()).unwrap())
        .collect();
    // println!("Input edges are: {:?}", input_list);

    // Construct the whole dependency graph
    let mut dep_graph = Graph::new();
    input_list.iter().for_each(|e| dep_graph.add_edge(e));
    // println!("Dependence graph is: {:?}", dep_graph);

    // From start nodes(without in nodes), generate work sequence
    let work_seq = DepSolver::new(dep_graph).fold(String::new(), |mut res, step| {
        res.push_str(step.id());
        res
    });

    println!("Given the dependences, final work flow is {}", work_seq);
}

#[derive(Debug, Clone, Hash)]
struct Node {
    id: String
}

impl Node {
    pub fn new(idr: &str) -> Node {
        Node {
            id: idr.to_string()
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn time(&self) -> u32 {
        debug_assert!(self.id.len() == 1);
        let code = self.id.bytes().next().unwrap();
        60 + (code - 65) as u32 // 'A' is 65
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Node {}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

#[derive(Debug)]
struct Edge {
    from: Node,
    to: Node,
}

impl Edge {
    pub fn new(desc: &str) -> Option<Edge> {
        lazy_static! {
            // Step W must be finished before step B can begin.
            static ref EDGE_PATTERN: Regex = Regex::new(r"Step (\w+) must be finished before step (\w+) can begin.").unwrap();
        }

        match EDGE_PATTERN.captures(desc) {
            Some(caps) => Some(Edge {
                from: Node::new(caps.get(1).unwrap().as_str()),
                to: Node::new(caps.get(2).unwrap().as_str()),
            }),
            _ => None,
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
    out_node_ids: HashSet<Node>,
}

impl InOutList {
    pub fn new() -> InOutList {
        InOutList {
            in_node_ids: HashSet::new(),
            out_node_ids: HashSet::new(),
        }
    }

    pub fn add_in_node(&mut self, node: &Node) {
        self.in_node_ids.insert(node.clone());
    }

    pub fn add_out_node(&mut self, node: &Node) {
        self.out_node_ids.insert(node.clone());
    }

    pub fn in_count(&self) -> u32 {
        self.in_node_ids.len() as u32
    }

    pub fn in_nodes(&self) -> &HashSet<Node> {
        &self.in_node_ids
    }

    pub fn out_nodes(&self) -> &HashSet<Node> {
        &self.out_node_ids
    }
}

#[derive(Debug)]
struct Graph {
    edge_map: HashMap<Node, InOutList>,
}

impl Graph {
    pub fn new() -> Graph {
        Graph {
            edge_map: HashMap::new(),
        }
    }

    pub fn add_edge(&mut self, edge: &Edge) {
        let mut entry = self
            .edge_map
            .entry(edge.from_node().clone())
            .or_insert(InOutList::new());
        entry.add_out_node(&edge.to_node());

        entry = self
            .edge_map
            .entry(edge.to_node().clone())
            .or_insert(InOutList::new());
        entry.add_in_node(&edge.from_node());
    }

    pub fn nodes(&self) -> &HashMap<Node, InOutList> {
        &self.edge_map
    }

    pub fn in_nodes<'a>(&'a self, node: &Node) -> &'a HashSet<Node> {
        self.edge_map.get(node).unwrap().in_nodes()
    }

    pub fn out_nodes<'a>(&'a self, node: &Node) -> &'a HashSet<Node> {
        self.edge_map.get(node).unwrap().out_nodes()
    }
}

struct DepSolver {
    dep_graph: Graph,
    candidates: BTreeSet<Node>,
    done_hist: HashSet<Node>,
}

impl DepSolver {
    pub fn new(graph: Graph) -> DepSolver {
        let start_nodes = graph
            .nodes()
            .iter()
            .filter(|(_, io_list)| io_list.in_count() == 0)
            .map(|(n, _)| n.clone())
            .collect();

        DepSolver {
            dep_graph: graph,
            candidates: start_nodes,
            done_hist: HashSet::new(),
        }
    }
}

impl Iterator for DepSolver {
    type Item = Node;
    fn next(&mut self) -> Option<Node> {
        // println!("Candidates: {:?}, done_hist: {:?}", self.candidates, self.done_hist);

        let done = match self.candidates.iter().next() {
            None => return None,
            Some(cand) => cand.clone(),
        };
        self.candidates.remove(&done);

        self.done_hist.insert(done.clone());
        let affect_nodes = self.dep_graph.out_nodes(&done);
        for node in affect_nodes {
            if !self.done_hist.contains(node) {
                let need_nodes = self.dep_graph.in_nodes(node);
                // println!("node {} need nodes: {:?}", node, need_nodes);
                if need_nodes.is_subset(&self.done_hist) {
                    self.candidates.insert(node.clone());
                }
            }
        }

        Some(done)
    }
}
