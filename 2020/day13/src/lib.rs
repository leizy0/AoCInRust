use std::{
    error,
    fmt::Display,
    fs::File,
    io::{self, BufRead, BufReader},
    path::{Path, PathBuf},
};

use clap::Parser;

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    EmptyInput,
    InvalidDepartTimeText(String),
    NoSchedulesInInput,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IOError(ioe) => write!(f, "I/O error: {}", ioe),
            Error::EmptyInput => write!(f, "Given empty input"),
            Error::InvalidDepartTimeText(s) => write!(f, "Invalid text for depart time: {}", s),
            Error::NoSchedulesInInput => write!(f, "No information about schedules in given input"),
        }
    }
}

impl error::Error for Error {}

#[derive(Debug, Parser)]
pub struct CLIArgs {
    pub input_path: PathBuf,
}

pub fn read_notes<P: AsRef<Path>>(path: P) -> Result<(usize, Vec<(usize, usize)>), Error> {
    let file = File::open(path).map_err(Error::IOError)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let depart_time = lines.next().ok_or(Error::EmptyInput).and_then(|l| {
        l.map_err(Error::IOError).and_then(|s| {
            s.parse::<usize>()
                .map_err(|_| Error::InvalidDepartTimeText(s.clone()))
        })
    })?;
    let bus_schedules = lines
        .next()
        .ok_or(Error::NoSchedulesInInput)
        .and_then(|l| {
            l.map_err(Error::IOError).map(|s| {
                s.split(',')
                    .enumerate()
                    .filter_map(|(interval, cycle_str)| cycle_str.parse::<usize>().ok().map(|cycle| (cycle, interval)))
                    .collect::<Vec<_>>()
            })
        })?;
    Ok((depart_time, bus_schedules))
}

pub fn lcm(m: usize, n: usize) -> usize {
    if m == 0 || n == 0 {
        0
    } else {
        m / gcd(m, n) * n
    }
}

fn gcd(mut m: usize, mut n: usize) -> usize {
    while n != 0 {
        let rem = m % n;
        m = n;
        n = rem;
    }

    m
}

#[test]
fn test_gcd() {
    assert_eq!(gcd(0, 5), 5);
    assert_eq!(gcd(5, 0), 5);
    assert_eq!(gcd(32, 48), 16);
    assert_eq!(gcd(5, 7), 1);
}

#[test]
fn test_lcm() {
    assert_eq!(lcm(0, 5), 0);
    assert_eq!(lcm(5, 0), 0);
    assert_eq!(lcm(32, 48), 96);
    assert_eq!(lcm(5, 7), 35);
}
