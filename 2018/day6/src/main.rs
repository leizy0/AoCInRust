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

    // Compute the center point with the least total manhatan distance from all input points
    let center = comp_center(&input_list);
    println!("Center point of given input points is {:?}", center);

    // Iterate points have fixed manhatan distance from the center point, distance += 1 after each iteration
    // Count points with distance from center less than 10000
    // Loop is over when count in one iteration is zero
    const MAX_DIST: u32 = 10000;
    let mut cur_count = 0;
    let mut total_count = 0;
    let mut cur_center_dist = 0;

    loop {
        let p_itr = FixDistIterator::new(&center, cur_center_dist);
        let mut p_count = 0u32;
        let mut max_dist_sum = 0u32;
        for p in p_itr {
            p_count += 1;
            let dist_sum = p.mht_dist_sum(&input_list);
            if dist_sum > max_dist_sum {
                max_dist_sum = dist_sum;
            }

            if dist_sum < MAX_DIST {
                cur_count += 1;
            }
        }

        println!(
            "Loop points(manhatan distance {} from center): {}/{} is safe, with max dist sum({})",
            cur_center_dist, cur_count, p_count, max_dist_sum
        );

        if cur_count == 0 {
            break;
        }

        total_count += cur_count;
        cur_count = 0;
        cur_center_dist += 1;
    }

    println!("there are total {} points within safe region(sum of manhatan distance from all input points is less than {})", total_count, MAX_DIST);
}

#[derive(Debug, Copy, Clone)]
struct Point {
    px: i32,
    py: i32,
}

impl Point {
    pub fn new(desc: &str) -> Option<Point> {
        lazy_static! {
            static ref POINT_PATTERN: Regex = Regex::new(r"(\d+), (\d+)").unwrap();
        }

        match POINT_PATTERN.captures(desc) {
            Some(caps) => Some(Point {
                px: i32::from_str(caps.get(1).unwrap().as_str()).unwrap(),
                py: i32::from_str(caps.get(2).unwrap().as_str()).unwrap(),
            }),
            _ => None,
        }
    }

    pub fn x(&self) -> i32 {
        self.px
    }

    pub fn y(&self) -> i32 {
        self.py
    }

    pub fn mht_dist(&self, p: &Point) -> u32 {
        (i32::abs(self.px - p.px) + i32::abs(self.py - p.py)) as u32
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

    pub fn mht_dist_sum(&self, p_list: &[Point]) -> u32 {
        let mut sum = 0u32;
        for p in p_list {
            sum += self.mht_dist(p);
        }

        sum
    }
}

fn comp_center(p_list: &Vec<Point>) -> Point {
    let (x_list, y_list): (Vec<_>, Vec<_>) = p_list.iter().map(|p| (p.x(), p.y())).unzip();
    let center_x = comp_abs_center_n(x_list);
    let center_y = comp_abs_center_n(y_list);

    Point {
        px: center_x,
        py: center_y,
    }
}

fn comp_abs_center_n(mut n_list: Vec<i32>) -> i32 {
    let count = n_list.len();
    n_list.sort_unstable();
    match count % 2 {
        1 => n_list[count / 2],
        _ => {
            let less = n_list[count / 2];
            let more = n_list[count / 2 + 1];
            let less_dist = comp_abs_dist_sum(less, &n_list);
            let more_dist = comp_abs_dist_sum(more, &n_list);

            if less_dist < more_dist {
                less
            } else {
                more
            }
        }
    }
}

fn comp_abs_dist_sum(org: i32, n_list: &[i32]) -> u32 {
    let mut dist = 0u32;
    for n in n_list {
        dist += i32::abs(org - *n) as u32;
    }

    dist
}

struct FixDistIterator {
    dist: u32,
    origin: Point,
    offset_x: i32,
    offset_y_sign: i32,
}

impl FixDistIterator {
    pub fn new(org: &Point, dist: u32) -> FixDistIterator {
        FixDistIterator {
            dist: dist,
            origin: *org,
            offset_x: -(dist as i32),
            offset_y_sign: -1,
        }
    }

    pub fn is_over(&self) -> bool {
        self.offset_x > self.dist as i32
    }
}

impl Iterator for FixDistIterator {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_over() {
            return None;
        }

        let offset_y = self.offset_y_sign * (self.dist as i32 - self.offset_x.abs());
        let next_p = Point {
            px: self.origin.x() + self.offset_x,
            py: self.origin.y() + offset_y,
        };

        if offset_y == 0 || self.offset_y_sign > 0 {
            self.offset_x += 1;
        }

        if offset_y != 0 {
            self.offset_y_sign *= -1;
        }

        Some(next_p)
    }
}
