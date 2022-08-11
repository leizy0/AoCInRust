use std::{path::Path, fs::File, io::{BufReader, BufRead}, fmt::Display};

use once_cell::sync::Lazy;
use regex::Regex;

use crate::Error;

struct Position {
    x: isize,
    y: isize,
    z: isize,
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

impl Position {
    pub fn new(x: isize, y: isize, z: isize) -> Self {
        Position { x, y, z }
    }

    pub fn mht_dist(&self, other: &Self) -> usize {
        (self.x - other.x).unsigned_abs()
        + (self.y - other.y).unsigned_abs()
        + (self.z - other.z).unsigned_abs()
    }
}


pub struct Nanobot {
    pos: Position,
    signal_radius: usize,
}

impl TryFrom<&str> for Nanobot {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        static NANOBOT_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"pos=<(-?\d+),(-?\d+),(-?\d+)>,\s+r=(\d+)").unwrap());
        let caps = NANOBOT_PATTERN.captures(value).ok_or(Error::NotMatchNanobotPattern(value.to_string()))?;
        Ok(Nanobot {
            pos: Position::new(
                caps[1].parse::<isize>().unwrap(),
                caps[2].parse::<isize>().unwrap(),
                caps[3].parse::<isize>().unwrap()
            ),
            signal_radius: caps[4].parse::<usize>().unwrap(),
        })
    }
}

impl Display for Nanobot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(pos: {}, signal radius: {})", self.pos, self.signal_radius)
    }
}

impl Nanobot {
    pub fn signal_rad(&self) -> usize {
        self.signal_radius
    }

    pub fn is_in_range(&self, other: &Self) -> bool {
        self.pos.mht_dist(&other.pos) <= self.signal_radius
    }
}

pub fn load_bots<P>(input_path: P) -> Result<Vec<Nanobot>, Error> where P: AsRef<Path> {
    let input_file = File::open(input_path).map_err(Error::IOError)?;
    let reader = BufReader::new(input_file);
    reader.lines()
        .map(|lr| lr.map_err(Error::IOError).and_then(|l| Nanobot::try_from(l.as_str())))
        .collect::<Result<Vec<_>, Error>>()
}