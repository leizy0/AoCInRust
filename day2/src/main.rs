use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() {
    let input_path = "./input.txt";
    let input =
        File::open(input_path).expect(&format!("Failed to open input file({})", input_path));
    let id_list: Vec<String> = BufReader::new(input).lines().map(|l| l.unwrap()).collect();

    let mut rep2_count = 0;
    let mut rep3_count = 0;
    for id in id_list {
        let counter = IdCounter::new(&id);
        rep2_count += if counter.has_rep_count(2) { 1 } else { 0 };
        rep3_count += if counter.has_rep_count(3) { 1 } else { 0 };
    }

    let checksum = rep2_count * rep3_count;
    println!("There are {} box with letter repeats exactly twice, {} box with letter repeats exactly three times, checksum is {}", rep2_count, rep3_count, checksum);
}

struct IdCounter {
    id: String,
    map: HashMap<char, u32>, // Key is char in id, value is repeat count
}

impl IdCounter {
    fn new(id: &str) -> IdCounter {
        let mut map = HashMap::new();

        // Compute repeat count of every letter in id, store it in map
        for c in id.chars() {
            let count = map.entry(c).or_insert(0);
            *count += 1;
        }

        IdCounter {
            id: id.to_string(),
            map,
        }
    }

    // Is there any chars in id have given repeat count?
    fn has_rep_count(&self, count: u32) -> bool {
        for (_k, v) in &self.map {
            if *v == count {
                return true;
            }
        }

        false
    }
}
