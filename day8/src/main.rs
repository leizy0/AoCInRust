use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() {
    let input_path = "input.txt";
    let input_file =
        File::open(input_path).expect(&format!("Failed to read input file({})", input_path));
    let input_list: Vec<u32> = BufReader::new(input_file)
        .lines()
        .flat_map(|l| {
            l.unwrap()
                .split_ascii_whitespace()
                .map(|s| s.parse::<u32>().unwrap())
                .collect::<Vec<u32>>()
        })
        .collect();

    let root = Node::parse_tree(&input_list);
    println!("Tree is {:?}", root);
    println!("Value of root node is {}", root.value());
}

#[derive(Debug)]
struct Node {
    metas: Vec<u32>,
    children: Option<Vec<Node>>
}

impl Node {
    pub fn parse_tree(num_desc: &[u32]) -> Node {
        let mut nodes = Node::parse_nodes(num_desc);
        assert_eq!(nodes.len(), 1);

        nodes.pop().unwrap()
    }

    fn parse_nodes(num_desc: &[u32]) -> Vec<Node> {
        let mut children = Vec::new();
        let mut start_pos = 0;

        while start_pos < num_desc.len() {
            // println!("Parse nodes start at {}", start_pos);

            let (first_node, end_pos) = Node::parse_first_node(&num_desc[start_pos..]);
            // println!("Parse nodes, Get first node({:?})", first_node);

            children.push(first_node);
            start_pos += end_pos;
        }
        
        assert_eq!(start_pos, num_desc.len());

        children
    }

    fn parse_first_node(num_desc: &[u32]) -> (Node, usize) {
        let chd_n = num_desc[0];
        let meta_n = num_desc[1];

        match chd_n {
            0 => {
                let end_pos = 2 + (meta_n as usize);
                (Node {
                    metas: Vec::from(&num_desc[2..end_pos]),
                    children: None
                }, end_pos)
            },
            n => {
                let mut chd_list = Vec::new();
                let mut start_pos = 2;
                for _ in 0..n {
                    // println!("Parse first node start at {}", start_pos);

                    let (node, end_pos) = Node::parse_first_node(&num_desc[start_pos..]);
                    // println!("Parse first node get node({:?})", node);

                    chd_list.push(node);
                    start_pos += end_pos;
                }

                let end_pos = start_pos + (meta_n as usize);

                (Node {
                    metas: Vec::from(&num_desc[start_pos..end_pos]),
                    children: Some(chd_list)
                }, end_pos)
            }
        }
    }

    pub fn value(&self) -> u32 {
        match self.children {
            None => self.metas.iter().sum(),
            Some(ref children) => {
                let mut res = 0;
                let valid_chd_ref_range = 1..=(children.len());
                for meta_d in &self.metas {
                    let ind = (*meta_d) as usize;
                    if valid_chd_ref_range.contains(&ind) {
                        res += children[ind - 1].value();
                    }
                }

                res
            }
        }
    }
}

fn comp_meta_sum(root: &Node) -> u32 {
    let mut sum = root.metas.iter().sum();

    if let Some(ref children) = root.children {
        for c in children {
            sum += comp_meta_sum(c);
        }
    }

    sum
}
