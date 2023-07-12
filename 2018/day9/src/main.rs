#[macro_use]
extern crate lazy_static;
extern crate regex;

use regex::Regex;
use std::fmt::{Display, Formatter, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() {
    let input_path = "input.txt";
    let input_file =
        File::open(input_path).expect(&format!("Failed to open input file({})", input_path));
    let input_list: Vec<MarbleGame> = BufReader::new(input_file)
        .lines()
        .map(|l| MarbleGame::new(&l.unwrap()).unwrap())
        .collect();

    for (ind, game) in input_list.iter().enumerate() {
        let (win_player_id, win_score) = game.simulate();
        println!(
            "Game #{}: player #{} wins, the final score is {}",
            ind,
            win_player_id + 1,
            win_score
        );
    }
}

struct MarbleGame {
    player_n: u32,
    last_marble_ind: u32,
}

impl MarbleGame {
    pub fn new(desc: &str) -> Option<MarbleGame> {
        lazy_static! {
            // 419 players; last marble is worth 72164 points
            static ref MARBLE_GAME_DESC_PATTERN: Regex = Regex::new(r"(\d+) players; last marble is worth (\d+) points").unwrap();
        }

        match MARBLE_GAME_DESC_PATTERN.captures(desc) {
            Some(caps) => Some(MarbleGame {
                player_n: caps.get(1).unwrap().as_str().parse().unwrap(),
                last_marble_ind: caps.get(2).unwrap().as_str().parse().unwrap(),
            }),
            _ => None,
        }
    }

    pub fn simulate(&self) -> (u32, u32) {
        let mut score_list = vec![0u32; self.player_n as usize];
        let mut marble_list = MarbleList::new();
        let mut cur_player_ind = 0;

        marble_list.push(&0);
        const BONUS_BASE: u32 = 23;
        for marble_id in 1..=(self.last_marble_ind) {
            if marble_id % BONUS_BASE == 0 {
                // Compute score
                let remove_pos = marble_list.back(7).unwrap();
                let remove_score = marble_list.get(remove_pos);
                marble_list.remove(remove_pos);
                score_list[cur_player_ind] += remove_score + marble_id;
            } else {
                // Insert marble
                let insert_pos = marble_list.next(1).unwrap();
                marble_list.insert_after(insert_pos, &marble_id);
            }

            cur_player_ind = (cur_player_ind + 1) % (self.player_n as usize);
            // println!("Marble list: {}", marble_list);
        }

        let (max_player_ind, max_score) = score_list
            .iter()
            .enumerate()
            .max_by_key(|(_ind, &score)| score)
            .unwrap();
        (max_player_ind as u32, *max_score as u32)
    }
}

type Marble = u32;

#[derive(Copy, Clone, PartialEq, Debug)]
struct MarbleNode {
    value: Marble,
    next: usize,
    prev: usize,
}

impl MarbleNode {
    pub fn value(&self) -> Marble {
        self.value
    }
}

#[derive(PartialEq, Debug)]
struct MarbleList {
    head: Option<usize>,
    tail: Option<usize>,
    current: Option<usize>,
    nodes: Vec<MarbleNode>,
}

impl Display for MarbleList {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if let Some(mut pos) = self.head {
            let end_pos = pos;
            write!(f, "[")?;
            loop {
                write!(f, "{}", self.get(pos))?;
                pos = self.nodes[pos].next;
                if pos == end_pos {
                    write!(f, "]")?;
                    break;
                } else {
                    write!(f, " ")?;
                }
            }
        } else {
            write!(f, "[]")?;
        }

        Ok(())
    }
}

impl MarbleList {
    pub fn new() -> MarbleList {
        MarbleList {
            head: None,
            tail: None,
            current: None,
            nodes: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        if self.head.is_none() {
            assert!(self.tail.is_none());
            assert!(self.current.is_none());

            true
        } else {
            assert!(self.tail.is_some());
            assert!(self.current.is_some());

            false
        }
    }

    pub fn get(&self, pos: usize) -> Marble {
        self.nodes[pos].value()
    }

    pub fn push(&mut self, value: &Marble) {
        let mut new_node = MarbleNode {
            value: *value,
            next: 0,
            prev: 0,
        };

        if self.is_empty() {
            self.nodes.push(new_node);
            self.head = Some(0);
            self.tail = Some(0);
            self.current = Some(0);
        } else {
            let tail_pos = self.tail.unwrap();
            new_node.next = self.head.unwrap();
            new_node.prev = self.tail.unwrap();

            let new_node_ind = self.nodes.len();
            self.nodes[tail_pos].next = new_node_ind;
            let head_pos = self.head.unwrap();
            self.nodes[head_pos].prev = new_node_ind;

            self.nodes.push(new_node);
            self.tail = Some(new_node_ind);
            self.current = Some(new_node_ind);
        }
    }

    pub fn insert_after(&mut self, pos: usize, value: &Marble) {
        assert!(!self.is_empty());

        let new_node_ind = self.nodes.len();
        let anchor_next_pos = self.nodes[pos].next;

        self.nodes.push(MarbleNode {
            value: *value,
            next: anchor_next_pos,
            prev: pos,
        });

        self.nodes[pos].next = new_node_ind;
        self.nodes[anchor_next_pos].prev = new_node_ind;

        if pos == self.tail.unwrap() {
            self.tail = Some(new_node_ind);
        }

        self.current = Some(new_node_ind);
    }

    pub fn next(&mut self, step: u32) -> Option<usize> {
        self.get_node(step as i32)
    }

    pub fn back(&mut self, step: u32) -> Option<usize> {
        self.get_node(-(step as i32))
    }

    pub fn get_node(&mut self, step: i32) -> Option<usize> {
        if self.is_empty() {
            return None;
        }

        let is_forward = step > 0;
        let step_abs = step.abs() as u32;
        let mut pos = self.current.unwrap();
        for _ in 0..step_abs {
            pos = if is_forward {
                self.nodes[pos].next
            } else {
                self.nodes[pos].prev
            }
        }

        Some(pos)
    }

    pub fn remove(&mut self, pos: usize) {
        assert!(!self.is_empty());

        if self.nodes.len() == 1 {
            self.nodes.pop();
            self.head = None;
            self.tail = None;
            self.current = None;
        } else {
            // Repair next and previous node of removed node
            let prev_pos = self.nodes[pos].prev;
            let next_pos = self.nodes[pos].next;
            self.nodes[prev_pos].next = next_pos;
            self.nodes[next_pos].prev = prev_pos;

            // Swap removed node and node at vector end,
            // and pop the end out
            let end_pos = self.nodes.len() - 1;
            self.nodes[pos] = self.nodes[end_pos];
            self.nodes.pop();

            // Repair next and previous node of end node
            let end_node_next = self.nodes[pos].next;
            let end_node_prev = self.nodes[pos].prev;
            self.nodes[end_node_prev].next = pos;
            self.nodes[end_node_next].prev = pos;

            // Repair head, tail and current pointer
            let head_pos = self.head.unwrap();
            let tail_pos = self.tail.unwrap();
            if pos == head_pos {
                self.head = Some(next_pos);
            } else if end_pos == head_pos {
                self.head = Some(pos);
            } else if pos == tail_pos {
                self.tail = Some(prev_pos);
            } else if end_pos == tail_pos {
                self.tail = Some(pos);
            }

            if next_pos == end_pos {
                self.current = Some(pos);
            } else {
                self.current = Some(next_pos);
            }
        }
    }
}

#[test]
fn test_push() {
    let mut list = MarbleList::new();
    list.push(&3);
    list.push(&5);
    list.push(&7);

    assert_eq!(
        list,
        MarbleList {
            head: Some(0),
            tail: Some(2),
            current: Some(2),
            nodes: vec![
                MarbleNode {
                    value: 3,
                    next: 1,
                    prev: 2,
                },
                MarbleNode {
                    value: 5,
                    next: 2,
                    prev: 0
                },
                MarbleNode {
                    value: 7,
                    next: 0,
                    prev: 1,
                },
            ],
        }
    );
}

#[test]
fn test_insert() {
    let mut list = MarbleList::new();
    list.push(&3);
    list.push(&5);
    list.push(&7);

    let pos = list.back(1).unwrap();
    assert_eq!(pos, 1);

    list.insert_after(pos, &6);

    assert_eq!(
        list,
        MarbleList {
            head: Some(0),
            tail: Some(2),
            current: Some(3),
            nodes: vec![
                MarbleNode {
                    value: 3,
                    next: 1,
                    prev: 2,
                },
                MarbleNode {
                    value: 5,
                    next: 3,
                    prev: 0
                },
                MarbleNode {
                    value: 7,
                    next: 0,
                    prev: 3,
                },
                MarbleNode {
                    value: 6,
                    next: 2,
                    prev: 1,
                },
            ],
        }
    );
}

#[test]
fn test_remove() {
    let mut list = MarbleList::new();
    list.push(&3);
    list.push(&5);
    list.push(&7);

    let pos = list.back(1).unwrap();
    assert_eq!(pos, 1);

    list.remove(pos);
    assert_eq!(
        list,
        MarbleList {
            head: Some(0),
            tail: Some(1),
            current: Some(1),
            nodes: vec![
                MarbleNode {
                    value: 3,
                    next: 1,
                    prev: 1,
                },
                MarbleNode {
                    value: 7,
                    next: 0,
                    prev: 0,
                }
            ],
        }
    );
}
