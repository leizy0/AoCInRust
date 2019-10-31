use std::fs::File;
use std::io::{BufRead, BufReader};
use std::iter::FromIterator;

fn main() {
    let input_path = "./input.txt";
    let input_file =
        File::open(input_path).expect(&format!("Failed to open input file({})", input_path));
    let input_list: Vec<String> = BufReader::new(input_file)
        .lines()
        .map(|l| l.unwrap())
        .collect();

    for (ind, input) in input_list.iter().enumerate() {
        let mut polymer = Polymer::new(input);
        let result = polymer.react();
        println!(
            "After reaction, polymer(#{}) has {} units remained({})",
            ind,
            result.chars().count(),
            result
        );
    }
}

const POLYMER_REACTED: char = ' ';

struct Polymer {
    unit_list: Vec<char>,
}

impl Polymer {
    pub fn new(org_link: &str) -> Polymer {
        Polymer {
            unit_list: org_link.chars().collect(),
        }
    }

    pub fn link(&self) -> String {
        String::from_iter(self.unit_list.iter())
    }

    pub fn react(&mut self) -> String {
        let mut cur_ind = 0;

        loop {
            match self.find_next_unit(cur_ind) {
                Some(next_ind) => {
                    if self.react_at(cur_ind, next_ind) {
                        match self.find_last_unit(cur_ind) {
                            Some(last_ind) => cur_ind = last_ind,
                            None => match self.find_next_unit(next_ind) {
                                Some(next_next_ind) => cur_ind = next_next_ind,
                                None => break,
                            },
                        };
                    } else {
                        cur_ind = next_ind;
                    }
                }
                None => break,
            }
        }

        self.unit_list = self
            .unit_list
            .iter()
            .filter(|&c| *c != POLYMER_REACTED)
            .copied()
            .collect();
        self.link()
    }

    fn react_at(&mut self, u0_pos: usize, u1_pos: usize) -> bool {
        let u0 = &self.unit_list[u0_pos];
        let u1 = &self.unit_list[u1_pos];

        let u0_low = u0.to_lowercase().to_string();
        let u0_up = u0.to_uppercase().to_string();
        let u1_str = u1.to_string();

        let can_react = if u0.is_uppercase() && u1.is_lowercase() {
            u0_low == u1_str
        } else if u0.is_lowercase() && u1.is_uppercase() {
            u0_up == u1_str
        } else {
            false
        };

        if can_react {
            self.unit_list[u0_pos] = POLYMER_REACTED;
            self.unit_list[u1_pos] = POLYMER_REACTED;
            true
        } else {
            false
        }
    }

    fn find_last_unit(&self, cur_pos: usize) -> Option<usize> {
        self.find_unit(cur_pos, -1)
    }

    fn find_next_unit(&self, cur_pos: usize) -> Option<usize> {
        self.find_unit(cur_pos, 1)
    }

    fn find_unit(&self, cur_pos: usize, step: isize) -> Option<usize> {
        let char_count = self.unit_list.len();

        let mut ind = cur_pos as isize;
        loop {
            ind += step;
            if ind < 0 || ind >= (char_count as isize) {
                break;
            } else if self.unit_list[ind as usize] != POLYMER_REACTED {
                return Some(ind as usize);
            }
        }

        None
    }
}
