use std::io::{BufRead, BufReader};
use std::fs::File;
use regex::Regex;
use std::str::FromStr;

fn main() {
    let input_path = "./input.txt";
    let input_file = File::open(input_path).expect(&format!("Failed to open file {}", input_path));
    let rect_list: Vec<Rect> = BufReader::new(input_file).lines().map(|l| Rect::new(&l.unwrap())).collect();

    let mut fabric = Fabric::new(1000, 1000);
    for rect in &rect_list {
        fabric.cut(rect);
    }

    println!("Total {} square inches overlapped", fabric.overlap_area());
}

pub struct Rect {
    x: u32,
    y: u32,
    w: u32,
    h: u32
}

impl Rect {
    pub fn new(desc: &str) -> Rect {
        // #1375 @ 516,787: 23x11
        let pattern = Regex::new(r"#\d+ @ (\d+),(\d+): (\d+)x(\d+)").unwrap();
        let cap = pattern.captures(desc).unwrap();
        Rect {
            x: u32::from_str(cap.get(1).unwrap().as_str()).unwrap(),
            y: u32::from_str(cap.get(2).unwrap().as_str()).unwrap(),
            w: u32::from_str(cap.get(3).unwrap().as_str()).unwrap(),
            h: u32::from_str(cap.get(4).unwrap().as_str()).unwrap()
        }
    }

    pub fn top_left(&self) -> (u32, u32) {
        (self.x, self.y)
    }

    pub fn bottom_right(&self) -> (u32, u32) {
        (self.x + self.w, self.y + self.h)
    }

    pub fn width(&self) -> u32 {
        self.w
    }

    pub fn height(&self) -> u32 {
        self.h
    }
}

#[derive(Copy, Clone)]
enum UnitStatus {
    Empty,
    Cut,
    Count
}

pub struct Fabric {
    w: u32,
    h: u32,
    canvas: Vec<UnitStatus>,
    overlap_count: u32
}

impl Fabric {
    pub fn new(width: u32, height: u32) -> Fabric {
        let canv = vec![UnitStatus::Empty; (width * height) as usize];

        Fabric {
            w: width,
            h: height,
            canvas: canv,
            overlap_count: 0
        }
    }

    pub fn width(&self) -> u32 {
        self.w
    }

    pub fn height(&self) -> u32 {
        self.h
    }

    pub fn overlap_area(&self) -> u32 {
        self.overlap_count
    }

    pub fn cut(&mut self, rect: &Rect) {
        let (tl_x, tl_y) = rect.top_left();
        let (br_x, br_y) = rect.bottom_right();
        for y in tl_y..br_y {
            for x in tl_x..br_x {
                let status = &mut self.canvas[(y * self.w + x) as usize];
                match *status {
                    UnitStatus::Empty => *status = UnitStatus::Cut,
                    UnitStatus::Cut => {
                        self.overlap_count += 1;
                        *status = UnitStatus::Count;
                    },
                    _ => ()
                };
            }
        }
    }
}