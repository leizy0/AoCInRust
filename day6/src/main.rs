#[macro_use]
extern crate lazy_static;
extern crate regex; 

use std::io::{BufRead, BufReader};
use std::fs::File;
use regex::Regex;
use std::str::FromStr;

fn main() {
    let input_path = "./input.txt";
    let input_file = File::open(input_path).expect(&format!("Failed to open input file({})", input_path));
    let input_list: Vec<Point> = BufReader::new(input_file).lines().map(|s| Point::new(&s.unwrap()).unwrap()).collect();

    println!("There are {} point(s) in input", input_list.len());
}

struct Point {
    px: u32,
    py: u32
}

impl Point {
    pub fn new(desc: &str) -> Option<Point> {
        lazy_static! { static ref POINT_PATTERN: Regex = Regex::new(r"(\d+), (\d+)").unwrap(); }

        match POINT_PATTERN.captures(desc) {
            Some(caps) => Some(Point {
                px: u32::from_str(caps.get(1).unwrap().as_str()).unwrap(),
                py: u32::from_str(caps.get(2).unwrap().as_str()).unwrap()
            }),
            _ => None
        }
    }

    pub fn x(&self) -> u32 {
        self.px
    }

    fn y(&self) -> u32 {
        self.py
    }
}


