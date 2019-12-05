
#[macro_use]
extern crate lazy_static;
extern crate regex;

use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;

fn main() {
    let input_path = "input.txt";
    let input_file = File::open(input_path).expect(&format!("Failed to open input file({})", input_path));
    let input_list: Vec<Point> = BufReader::new(input_file).lines().map(|l| Point::new(&l.unwrap()).unwrap()).collect();

    let init_range = comp_points_range(&input_list);
    println!("There are {} points in input, with range({} x {})", input_list.len(), init_range.0, init_range.1);
}

struct Point {
    x: i32,
    y: i32,
    vx: i32,
    vy:i32,
}

impl Point {
    pub fn new(desc: &str) -> Option<Point> {
        lazy_static! {
            // position=< 50201,  30185> velocity=<-5, -3>
            static ref POINT_DESC_PATTERN: Regex = Regex::new(r"position=< *(-?\d+), *(-?\d+)> velocity=< *(-?\d+), *(-?\d+)>").unwrap();
        }

        POINT_DESC_PATTERN.captures(desc).map(|matches| {
            Point {
                x: matches.get(1).unwrap().as_str().parse().unwrap(),
                y: matches.get(2).unwrap().as_str().parse().unwrap(),
                vx: matches.get(3).unwrap().as_str().parse().unwrap(),
                vy: matches.get(4).unwrap().as_str().parse().unwrap(),
            }
        })
    }
}

fn comp_points_range(points: &[Point]) -> (u32, u32) {
    if points.is_empty() {
        return (0, 0);
    }

    let mut minx = points[0].x;
    let mut miny = points[0].y;
    let mut maxx = points[0].x;
    let mut maxy = points[0].y;
    for i in 1..points.len() {
        let cur_px = points[i].x;
        let cur_py = points[i].y;
        if cur_px < minx {
            minx = cur_px;
        } else if cur_px > maxx {
            maxx = cur_px;
        }

        if cur_py < miny {
            miny = cur_py;
        } else if cur_py > maxy {
            maxy = cur_py;
        }
    }

    assert!(maxx >= minx);
    assert!(maxy >= miny);

    ((maxx - minx + 1) as u32, (maxy - miny + 1) as u32)
}
