
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate image;

use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;
use image::{GrayImage, Luma};

fn main() {
    let input_path = "input.txt";
    let input_file = File::open(input_path).expect(&format!("Failed to open input file({})", input_path));
    let input_list: Vec<Point> = BufReader::new(input_file).lines().map(|l| Point::new(&l.unwrap()).unwrap()).collect();

    let mut simulator = StarMoveSimulator::new(input_list);

    const START_TICK: u32 = 9982;
    const END_TICK: u32 = 10046u32;
    simulator.sim_tick(START_TICK);
    for i in START_TICK..=END_TICK {
        let range = simulator.range();
        let mut image = GrayImage::from_pixel(range.0, range.1, Luma([0]));
        let (bias_x, bias_y) = simulator.bound_min().unwrap();

        for star in simulator.star_iter() {
            let indx = (star.x - bias_x) as u32;
            let indy = (star.y - bias_y) as u32;
            *image.get_pixel_mut(indx, indy) = Luma([255u8]);
        }

        let file_path = format!("images/pic_{}.png", i);
        let message = format!("Failed to save image({})", file_path);
        image.save(file_path).expect(&message);
        simulator.sim_tick(1);
    }
}

#[derive(Copy, Clone)]
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

struct StarMoveSimulator {
    stars: Vec<Point>,
}

impl StarMoveSimulator {
    pub fn new(point_list: Vec<Point>) -> StarMoveSimulator {
        StarMoveSimulator {
            stars: point_list,
        }
    }

    pub fn sim_tick(&mut self, tick_n: u32) {
        for star in &mut self.stars {
            star.x += star.vx * (tick_n as i32);
            star.y += star.vy * (tick_n as i32);
        }
    }

    pub fn range(&self) -> (u32, u32) {
        comp_points_range(&self.stars)
    }

    pub fn bound_min(&self) -> Option<(i32, i32)> {
        if self.stars.is_empty() {
            return None
        }

        let mut min_x = self.stars[0].x;
        let mut min_y = self.stars[0].y;
        for star in &self.stars {
            if star.x < min_x {
                min_x = star.x;
            }

            if star.y < min_y {
                min_y = star.y;
            }
        }

        Some((min_x, min_y))
    }

    pub fn star_iter(&self) -> &[Point] {
        &self.stars
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
