use std::{
    cell::RefCell,
    error,
    fmt::Display,
    fs::File,
    io::{stdout, BufRead, BufReader, Write},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use clap::Parser;
use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Debug)]
pub enum Error {
    InvalidRobotText(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidRobotText(s) => write!(f, "Invalid text({}) for robot.", s),
        }
    }
}

impl error::Error for Error {}

#[derive(Debug, Parser)]
pub struct Part1CLIArgs {
    pub map_width: usize,
    pub map_height: usize,
    pub input_path: PathBuf,
}

#[derive(Debug, Parser)]
pub struct Part2CLIArgs {
    pub map_width: usize,
    pub map_height: usize,
    pub input_path: PathBuf,
    pub move_start: usize,
    pub move_step: usize,
}

#[derive(Debug, Clone)]
pub struct Position {
    x: usize,
    y: usize,
}

impl Position {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    fn offset_wrap(&self, vec: &Vector, wrap_width: usize, wrap_height: usize) -> Self {
        fn add_wrap(left: usize, right: isize, wrap: usize) -> usize {
            let wrap = isize::try_from(wrap).unwrap();
            let mut sum = (isize::try_from(left).unwrap() + right) % wrap;
            if sum < 0 {
                sum += wrap;
            }

            usize::try_from(sum).unwrap()
        }

        Position::new(
            add_wrap(self.x, vec.x, wrap_width),
            add_wrap(self.y, vec.y, wrap_height),
        )
    }
}

#[derive(Debug, Clone)]
struct Vector {
    x: isize,
    y: isize,
}

impl Vector {
    pub fn new(x: isize, y: isize) -> Self {
        Self { x, y }
    }
}

#[derive(Debug)]
pub struct Map {
    width: usize,
    height: usize,
    marks: RefCell<Vec<bool>>,
}

impl Map {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            marks: RefCell::new(vec![false; width * height]),
        }
    }

    pub fn quad_ind(&self, pos: &Position) -> Option<usize> {
        if self.width % 2 == 0 || self.height % 2 == 0 {
            None
        } else {
            let spliter_x = self.width / 2;
            let spliter_y = self.height / 2;
            if pos.x == spliter_x || pos.y == spliter_y {
                None
            } else {
                let mut ind = if pos.x < spliter_x { 0 } else { 1 };
                if pos.y > spliter_y {
                    ind += 2;
                }

                Some(ind)
            }
        }
    }

    pub fn display(&self, robots: &[Robot]) -> Result<()> {
        self.clear_marks();
        for robot in robots {
            self.mark(robot.pos());
        }

        let mut lock = stdout().lock();
        for y in 0..self.height {
            for x in 0..self.width {
                write!(
                    lock,
                    "{}",
                    if self.is_marked(&Position::new(x, y)).unwrap_or(false) {
                        'R'
                    } else {
                        '.'
                    }
                )?;
            }
            writeln!(lock)?;
        }

        Ok(())
    }

    fn clear_marks(&self) {
        for mark in self.marks.borrow_mut().iter_mut() {
            *mark = false;
        }
    }

    fn mark(&self, pos: &Position) {
        if let Some(ind) = self.pos_to_ind(pos) {
            self.marks.borrow_mut()[ind] = true;
        }
    }

    fn is_marked(&self, pos: &Position) -> Option<bool> {
        self.pos_to_ind(pos).map(|ind| self.marks.borrow()[ind])
    }

    fn pos_to_ind(&self, pos: &Position) -> Option<usize> {
        if pos.x < self.width && pos.y < self.height {
            Some(pos.y * self.width + pos.x)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct Robot {
    pos: Position,
    velocity: Vector,
}

impl TryFrom<&str> for Robot {
    type Error = Error;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        static ROBOT_PATTERN: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"p=(\d+),(\d+) v=(-?\d+),(-?\d+)").unwrap());

        if let Some(caps) = ROBOT_PATTERN.captures(value) {
            let pos_x = caps[1].parse::<usize>().unwrap();
            let pos_y = caps[2].parse::<usize>().unwrap();
            let vel_x = caps[3].parse::<isize>().unwrap();
            let vel_y = caps[4].parse::<isize>().unwrap();
            Ok(Robot {
                pos: Position::new(pos_x, pos_y),
                velocity: Vector::new(vel_x, vel_y),
            })
        } else {
            Err(Error::InvalidRobotText(value.to_string()))
        }
    }
}

impl Robot {
    pub fn move_n_in(&mut self, count: usize, map: &Map) {
        for _ in 0..count {
            self.pos = self.pos.offset_wrap(&self.velocity, map.width, map.height);
        }
    }

    pub fn pos(&self) -> &Position {
        &self.pos
    }
}

pub fn read_robots<P: AsRef<Path>>(path: P) -> Result<Vec<Robot>> {
    let file = File::open(&path)
        .with_context(|| format!("Failed to open given file({}).", path.as_ref().display()))?;
    let reader = BufReader::new(file);
    reader
        .lines()
        .enumerate()
        .map(|(ind, line)| {
            line.with_context(|| {
                format!(
                    "Failed to read line {} in given file({}).",
                    ind + 1,
                    path.as_ref().display()
                )
            })
            .and_then(|s| {
                Robot::try_from(s.as_str()).with_context(|| {
                    format!(
                        "Failed to read robot from line {} in given file({}).",
                        ind + 1,
                        path.as_ref().display()
                    )
                })
            })
        })
        .collect()
}
