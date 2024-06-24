use std::{
    error,
    fmt::Display,
    fs::File,
    io::{self, BufRead, BufReader},
    path::{Path, PathBuf},
};

use clap::Parser;
use int_enum::IntEnum;
use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    InvalidInstText(String),
    InvalidTrunDegree(isize),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IOError(ioe) => write!(f, "I/O error: {}", ioe),
            Error::InvalidInstText(s) => write!(f, "Invalid instruction text({})", s),
            Error::InvalidTrunDegree(deg) => write!(f, "Invalid turning degree({})", deg),
        }
    }
}

impl error::Error for Error {}

#[derive(Debug, Parser)]
pub struct CLIArgs {
    pub input_path: PathBuf,
}

#[derive(Debug)]
pub enum Instruction {
    North(isize),
    South(isize),
    West(isize),
    East(isize),
    Left(isize),
    Right(isize),
    Forward(isize),
}

impl TryFrom<&str> for Instruction {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        type InstCtorFn = fn(&str) -> Option<Instruction>;
        static CTORS: Lazy<Vec<InstCtorFn>> = Lazy::new(|| {
            vec![
                Instruction::try_new_north,
                Instruction::try_new_south,
                Instruction::try_new_west,
                Instruction::try_new_east,
                Instruction::try_new_left,
                Instruction::try_new_right,
                Instruction::try_new_forward,
            ]
        });

        CTORS
            .iter()
            .filter_map(|f| f(value))
            .next()
            .ok_or(Error::InvalidInstText(value.to_string()))
    }
}

impl Instruction {
    pub fn try_new_north(s: &str) -> Option<Instruction> {
        static PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"N([+-]?\d+)").unwrap());

        PATTERN
            .captures(s)
            .and_then(|caps| caps[1].parse::<isize>().ok().map(|i| Instruction::North(i)))
    }

    pub fn try_new_south(s: &str) -> Option<Instruction> {
        static PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"S([+-]?\d+)").unwrap());

        PATTERN
            .captures(s)
            .and_then(|caps| caps[1].parse::<isize>().ok().map(|i| Instruction::South(i)))
    }

    pub fn try_new_west(s: &str) -> Option<Instruction> {
        static PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"W([+-]?\d+)").unwrap());

        PATTERN
            .captures(s)
            .and_then(|caps| caps[1].parse::<isize>().ok().map(|i| Instruction::West(i)))
    }

    pub fn try_new_east(s: &str) -> Option<Instruction> {
        static PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"E([+-]?\d+)").unwrap());

        PATTERN
            .captures(s)
            .and_then(|caps| caps[1].parse::<isize>().ok().map(|i| Instruction::East(i)))
    }

    pub fn try_new_left(s: &str) -> Option<Instruction> {
        static PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"L([+-]?\d+)").unwrap());

        PATTERN
            .captures(s)
            .and_then(|caps| caps[1].parse::<isize>().ok().map(|i| Instruction::Left(i)))
    }

    pub fn try_new_right(s: &str) -> Option<Instruction> {
        static PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"R([+-]?\d+)").unwrap());

        PATTERN
            .captures(s)
            .and_then(|caps| caps[1].parse::<isize>().ok().map(|i| Instruction::Right(i)))
    }

    pub fn try_new_forward(s: &str) -> Option<Instruction> {
        static PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"F([+-]?\d+)").unwrap());

        PATTERN.captures(s).and_then(|caps| {
            caps[1]
                .parse::<isize>()
                .ok()
                .map(|i| Instruction::Forward(i))
        })
    }
}

#[derive(Debug, Clone)]
pub struct Position {
    x: isize,
    y: isize,
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl Position {
    pub fn new(x: isize, y: isize) -> Self {
        Self { x, y }
    }

    pub fn m_dist(&self, other: &Position) -> usize {
        let x_abs_diff = usize::try_from(self.x.abs_diff(other.x)).unwrap();
        let y_abs_diff = usize::try_from(self.y.abs_diff(other.y)).unwrap();
        x_abs_diff + y_abs_diff
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, IntEnum)]
enum Direction {
    North = 0,
    East = 1,
    South = 2,
    West = 3,
}

pub struct Ship {
    pos: Position,
    dir: Direction,
}

impl Ship {
    pub fn new() -> Self {
        Self {
            pos: Position::new(0, 0),
            dir: Direction::East,
        }
    }

    pub fn pos(&self) -> Position {
        self.pos.clone()
    }

    pub fn handle(&mut self, inst: &Instruction) -> Result<(), Error> {
        match inst {
            Instruction::North(dist) => self.pos.y -= dist,
            Instruction::South(dist) => self.pos.y += dist,
            Instruction::West(dist) => self.pos.x -= dist,
            Instruction::East(dist) => self.pos.x += dist,
            Instruction::Left(deg) => self.turn_clockwise(-deg)?,
            Instruction::Right(deg) => self.turn_clockwise(*deg)?,
            Instruction::Forward(dist) => self.forward(*dist),
        }

        Ok(())
    }

    fn turn_clockwise(&mut self, deg: isize) -> Result<(), Error> {
        if deg % 90 != 0 {
            return Err(Error::InvalidTrunDegree(deg));
        }

        self.dir =
            Direction::try_from((((u8::from(self.dir) as isize + deg / 90) % 4 + 4) % 4) as u8)
                .unwrap();
        Ok(())
    }

    fn forward(&mut self, dist: isize) {
        match self.dir {
            Direction::North => self.pos.y -= dist,
            Direction::East => self.pos.x += dist,
            Direction::South => self.pos.y += dist,
            Direction::West => self.pos.x -= dist,
        }
    }
}

pub fn read_insts<P: AsRef<Path>>(path: P) -> Result<Vec<Instruction>, Error> {
    let file = File::open(path).map_err(Error::IOError)?;
    let reader = BufReader::new(file);
    reader
        .lines()
        .map(|l| {
            l.map_err(Error::IOError)
                .and_then(|s| Instruction::try_from(s.as_str()))
        })
        .collect()
}
