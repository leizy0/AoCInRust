use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use once_cell::sync::Lazy;
use regex::Regex;

use crate::Error;

#[derive(Debug)]
pub struct Point {
    x: isize,
    y: isize,
    z: isize,
    w: isize,
}

impl TryFrom<&str> for Point {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        static POINT_PATTERN: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"(-?\d+),(-?\d+),(-?\d+),(-?\d+)").unwrap());
        let caps = POINT_PATTERN
            .captures(value)
            .ok_or(Error::NotMatchPointPattern(value.to_string()))?;
        Ok(Point {
            x: caps[1].parse::<isize>().unwrap(),
            y: caps[2].parse::<isize>().unwrap(),
            z: caps[3].parse::<isize>().unwrap(),
            w: caps[4].parse::<isize>().unwrap(),
        })
    }
}

impl Point {
    pub fn mht_dist(&self, other: &Point) -> usize {
        self.x.abs_diff(other.x)
            + self.y.abs_diff(other.y)
            + self.z.abs_diff(other.z)
            + self.w.abs_diff(other.w)
    }
}

pub fn load_points<P>(input_path: P) -> Result<Vec<Point>, Error>
where
    P: AsRef<Path>,
{
    let input_file = File::open(input_path).map_err(Error::IOError)?;
    let reader = BufReader::new(input_file);
    reader
        .lines()
        .map(|lr| {
            lr.map_err(Error::IOError)
                .and_then(|l| Point::try_from(l.as_str()))
        })
        .collect::<Result<Vec<_>, Error>>()
}
