use std::fs::File;
use std::io::{BufRead, BufReader};
use std::iter::FromIterator;

fn main() {
    let input_path = "./input.txt";
    let input = File::open(input_path).expect(
        &format!("Failed to open input file({})", input_path));
    let id_list: Vec<String> = BufReader::new(input).lines().map(|l| l.unwrap()).collect();

    let id_count = id_list.len();

    // Loop on all pairs of ids
    for i in 0..id_count {
        for j in (i + 1)..id_count {
            let id1 = &id_list[i];
            let id2 = &id_list[j];

            let char_list1: Vec<char> = id1.chars().collect();
            let char_list2: Vec<char> = id2.chars().collect();
            if char_list1.len() != char_list2.len() {
                println!("Two ids({}, {}) have different char count, ignore", id1, id2);
                continue;
            }

            let com_chars = list_com_elems(&char_list1, &char_list2);
            if com_chars.len() == (char_list1.len() - 1) {
                println!("Common sequence({}) with only one char different found, between {} and {}",
                    String::from_iter(com_chars.iter()), id1, id2);
                return;
            }
        }
    }
    
    println!("Can't find any id pair has exactly one char different");
    std::process::exit(1);
}

// List all same elements at same position between given vectors
// if two vector have different length, then return empty result
fn list_com_elems<T: std::cmp::PartialEq + Copy>(v1: &Vec<T>, v2: &Vec<T>) -> Vec<T> {
    let mut com_elems = Vec::new();
    if v1.len() != v2.len() {
        return com_elems;
    }

    for i in 0..(v1.len()) {
        if v1[i] == v2[i] {
            com_elems.push(v1[i]);
        }
    }

    com_elems
}