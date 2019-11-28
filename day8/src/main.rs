use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() {
    let input_path = "input.txt";
    let input_file = File::open(input_path).expect(&format!("Failed to read input file({})", input_path));
    let input_list: Vec<u32> = BufReader::new(input_file).lines().flat_map(|l| l.unwrap().split_ascii_whitespace().map(|s| s.parse::<u32>().unwrap()).collect::<Vec<u32>>()).collect();

    println!("There are {} numbers in input", input_list.len());
}
