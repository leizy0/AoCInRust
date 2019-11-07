#[macro_use]
extern crate lazy_static;
extern crate regex;

use regex::Regex;
use std::cmp;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::iter::Iterator;
use std::str::FromStr;

fn main() {
    let input_path = "./input.txt";
    let input_file =
        File::open(input_path).expect(&format!("Failed to open input file({})", input_path));
    let input_list: Vec<Point> = BufReader::new(input_file)
        .lines()
        .map(|s| Point::new(&s.unwrap()).unwrap())
        .collect();

    // Construct grid
    let (maxw, maxh) = input_list.iter().fold((0, 0), |acc, p| {
        (cmp::max(acc.0, p.px), cmp::max(acc.1, p.py))
    });
    println!("Consturct grid({} x {})", maxw, maxh);
    let mut grid = vec![vec![None as Option<u32>; maxw as usize]; maxh as usize];

    // Iterate grid, compute closet point index for every point in grid, and accumulate count of same index
    let mut index_count_map = HashMap::new();
    for y in 0..maxh {
        for x in 0..maxw {
            let cur_p = Point { px: x, py: y };
            let closest = cur_p.closest_index(&input_list);
            match closest {
                Some(i) => {
                    let entry = index_count_map.entry(i).or_insert(0);
                    *entry += 1;
                }
                _ => (),
            }

            grid[y as usize][x as usize] = closest;
        }
    }

    // Find index of point with infinite closest point count, which has closest point at border
    let mut inf_ind_set = HashSet::new();
    // Iterate along up and down border
    for x in 0..maxw {
        match grid[0][x as usize] {
            Some(i) => inf_ind_set.insert(i),
            _ => false,
        };

        match grid[maxh as usize - 1][x as usize] {
            Some(i) => inf_ind_set.insert(i),
            _ => false,
        };
    }

    // Iterate along left and right border
    for y in 0..maxh {
        match grid[y as usize][0] {
            Some(i) => inf_ind_set.insert(i),
            _ => false,
        };

        match grid[y as usize][maxw as usize - 1] {
            Some(i) => inf_ind_set.insert(i),
            _ => false,
        };
    }

    // Find point with the most finite closest point count, Comput and print final result
    let (max_closest_ind, max_closest_count) = index_count_map
        .iter()
        .filter(|(k, _)| !inf_ind_set.contains(k))
        .max_by_key(|(_, v)| *v)
        .unwrap();
    println!(
        "The {:?}(#{}) has the most points({}) that's closest with it",
        input_list[*max_closest_ind as usize], max_closest_ind, max_closest_count
    );
}

#[derive(Debug)]
struct Point {
    px: u32,
    py: u32,
}

impl Point {
    pub fn new(desc: &str) -> Option<Point> {
        lazy_static! {
            static ref POINT_PATTERN: Regex = Regex::new(r"(\d+), (\d+)").unwrap();
        }

        match POINT_PATTERN.captures(desc) {
            Some(caps) => Some(Point {
                px: u32::from_str(caps.get(1).unwrap().as_str()).unwrap(),
                py: u32::from_str(caps.get(2).unwrap().as_str()).unwrap(),
            }),
            _ => None,
        }
    }

    pub fn x(&self) -> u32 {
        self.px
    }

    pub fn y(&self) -> u32 {
        self.py
    }

    pub fn mht_dist(&self, p: &Point) -> u32 {
        (i32::abs(self.px as i32 - p.px as i32) + i32::abs(self.py as i32 - p.py as i32)) as u32
    }

    pub fn closest_index(&self, p_list: &[Point]) -> Option<u32> {
        if p_list.len() == 0 {
            return None;
        } else if p_list.len() == 1 {
            return Some(0);
        }

        let mut dist_list: Vec<(u32, u32)> = p_list
            .iter()
            .enumerate()
            .map(|(i, p)| (i as u32, self.mht_dist(p)))
            .collect();
        dist_list.sort_unstable_by_key(|(_, d)| *d);

        if dist_list[0].1 == dist_list[1].1 {
            None
        } else {
            Some(dist_list[0].0)
        }
    }
}
