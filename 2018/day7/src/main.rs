#[macro_use]
extern crate lazy_static;
extern crate regex;

use regex::Regex;
use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};

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
    const WORKER_N: u32 = 5;
    let (total_time, work_seq) = WorkSimulator::new(dep_graph, WORKER_N).simulate();
    let seq_str = work_seq.iter().fold(String::new(), |mut s, n| {
        s.push_str(n.id());
        s
    });

    println!("Given the dependences and {} workers, it'll take {} seconds to finish, and the final work flow is {}", WORKER_N, total_time, seq_str);
}

#[derive(Debug, Clone, Hash)]
struct Node {
    id: String,
}

impl Node {
    pub fn new(idr: &str) -> Node {
        Node {
            id: idr.to_string(),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn time(&self) -> u32 {
        debug_assert!(self.id.len() == 1);
        let code = self.id.bytes().next().unwrap();
        61 + (code - 65) as u32 // 'A' is 65
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

#[test]
fn test_node_time() {
    let node_a = Node::new("A");
    let node_z = Node::new("Z");

    assert_eq!(node_a.time(), 61);
    assert_eq!(node_z.time(), 86);
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

struct WorkSimulator {
    dep_graph: Graph,
    workers: Vec<Worker>,
    candidates: BTreeSet<Node>,
    done_hist: HashSet<Node>,
}

impl WorkSimulator {
    pub fn new(graph: Graph, worker_n: u32) -> WorkSimulator {
        let start_nodes = graph
            .nodes()
            .iter()
            .filter(|(_, io_list)| io_list.in_count() == 0)
            .map(|(n, _)| n.clone())
            .collect();

        WorkSimulator {
            dep_graph: graph,
            workers: vec![Worker::new(); worker_n as usize],
            candidates: start_nodes,
            done_hist: HashSet::new(),
        }
    }

    pub fn simulate(&mut self) -> (u32, Vec<Node>) {
        // Iterate until canidates is empty
        let mut pass_time = 0u32;
        let mut work_seq = Vec::new();
        while !self.candidates.is_empty() || self.workers.iter().any(|w| !w.is_idle()) {
            // println!("Candidates: {:?}, done_hist: {:?}", self.candidates, self.done_hist);

            // Assign nodes to idle workers
            for worker in &mut self.workers {
                if worker.is_idle() {
                    if let Some(next_node) = self.candidates.iter().next().cloned() {
                        // println!("Assign node({:?}) to worker", next_node);
                        self.candidates.remove(&next_node);
                        worker.work_on(next_node);
                    }
                }
            }

            // Sort the worker by left time, pass the least left time
            let min_complete_time = self
                .workers
                .iter()
                .filter(|w| !w.is_idle())
                .min_by_key(|w| w.left_time())
                .unwrap()
                .left_time();
            // println!("Pass time({})", min_complete_time);
            for w in &mut self.workers {
                if !w.is_idle() {
                    if let Some(done) = w.sim_tick(min_complete_time) {
                        // Iterate done node, add new node to candidates according to dependence graph
                        self.done_hist.insert(done.clone());
                        let affect_nodes = self.dep_graph.out_nodes(&done);
                        for node in affect_nodes {
                            if !self.done_hist.contains(node) {
                                let need_nodes = self.dep_graph.in_nodes(node);
                                // println!("node {:?} need nodes: {:?}", node, need_nodes);
                                if need_nodes.is_subset(&self.done_hist) {
                                    self.candidates.insert(node.clone());
                                }
                            }
                        }

                        work_seq.push(done);
                    }
                }
            }
            pass_time += min_complete_time;
        }

        // From the end state of above loop, generate outputs
        (pass_time, work_seq)
    }
}

#[derive(Clone)]
struct Worker {
    working_node: Option<Node>,
    left_time: u32,
}

impl Worker {
    pub fn new() -> Worker {
        Worker {
            working_node: None,
            left_time: 0,
        }
    }

    pub fn is_idle(&self) -> bool {
        self.working_node == None
    }

    pub fn left_time(&self) -> u32 {
        if self.is_idle() {
            0
        } else {
            self.left_time
        }
    }

    pub fn work_on(&mut self, node: Node) {
        if !self.is_idle() {
            panic!(
                "Working on node({:?}), can't start node({:?}),",
                self.working_node, node
            );
        }

        self.left_time = node.time();
        self.working_node = Some(node);
    }

    pub fn sim_tick(&mut self, tick_n: u32) -> Option<Node> {
        if self.is_idle() {
            return None;
        }

        return if self.left_time <= tick_n {
            let done_node = self.working_node.clone().unwrap();
            self.working_node = None;
            self.left_time = 0;
            Some(done_node)
        } else {
            self.left_time -= tick_n;
            None
        };
    }
}
