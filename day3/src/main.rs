use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;

fn main() {
    let input_path = "./input.txt";
    let input_file = File::open(input_path).expect(&format!("Failed to open file {}", input_path));
    let rect_list: Vec<Rect> = BufReader::new(input_file)
        .lines()
        .map(|l| Rect::new(&l.unwrap()))
        .collect();

    let rect_count = rect_list.len();
    for i in 0..rect_count {
        let mut has_overlapped = false;
        for j in 0..rect_count {
            if i != j && is_overlap(&rect_list[i], &rect_list[j]) {
                has_overlapped = true;
                break;
            }
        }

        if !has_overlapped {
            println!(
                "Rectangle(#{}) doesn't overlap with others",
                rect_list[i].id()
            );
            return;
        }
    }

    println!("No non-overlapped rectangle found");
    std::process::exit(1);
}

pub struct Rect {
    n: u32,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

impl Rect {
    pub fn new(desc: &str) -> Rect {
        // #1375 @ 516,787: 23x11
        let pattern = Regex::new(r"#(\d+) @ (\d+),(\d+): (\d+)x(\d+)").unwrap();
        let cap = pattern.captures(desc).unwrap();
        Rect {
            n: u32::from_str(cap.get(1).unwrap().as_str()).unwrap(),
            x: u32::from_str(cap.get(2).unwrap().as_str()).unwrap(),
            y: u32::from_str(cap.get(3).unwrap().as_str()).unwrap(),
            w: u32::from_str(cap.get(4).unwrap().as_str()).unwrap(),
            h: u32::from_str(cap.get(5).unwrap().as_str()).unwrap(),
        }
    }

    pub fn top_left(&self) -> (u32, u32) {
        (self.x, self.y)
    }

    pub fn bottom_right(&self) -> (u32, u32) {
        (self.x + self.w, self.y + self.h)
    }

    pub fn id(&self) -> u32 {
        self.n
    }

    pub fn width(&self) -> u32 {
        self.w
    }

    pub fn height(&self) -> u32 {
        self.h
    }
}

fn is_overlap(rect1: &Rect, rect2: &Rect) -> bool {
    !is_not_overlap(rect1, rect2)
}

fn is_not_overlap(rect1: &Rect, rect2: &Rect) -> bool {
    let (tl_x1, tl_y1) = rect1.top_left();
    let (br_x1, br_y1) = rect1.bottom_right();
    let (tl_x2, tl_y2) = rect2.top_left();
    let (br_x2, br_y2) = rect2.bottom_right();

    br_x1 <= tl_x2 ||
    br_y1 <= tl_y2 ||
    br_x2 <= tl_x1 ||
    br_y2 <= tl_y1
}
