use std::{
    error,
    fmt::Display,
    fs::File,
    io::{self, BufRead, BufReader},
    ops::Range,
    path::Path,
};

use clap::Parser;
use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    InvalidPassword(String),
    InvalidConstraint(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IOError(ioe) => write!(f, "I/O error: {}", ioe),
            Error::InvalidPassword(s) => write!(f, "Invalid password: {}", s),
            Error::InvalidConstraint(s) => write!(f, "Invalid constraint: {}", s),
        }
    }
}

impl error::Error for Error {}

#[derive(Debug, Parser)]
pub struct CliArgs {
    pub input_path: String,
}

struct Constraint {
    c: char,
    range: Range<usize>,
}

impl TryFrom<&str> for Constraint {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        static PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\d+)-(\d+) (.)").unwrap());
        if let Some(caps) = PATTERN.captures(value) {
            let low_bound = caps[1].parse::<usize>().unwrap();
            let high_bound = caps[2].parse::<usize>().unwrap();
            let c = caps[3].chars().next().unwrap();

            Ok(Constraint {
                c,
                range: low_bound..(high_bound + 1),
            })
        } else {
            Err(Error::InvalidConstraint(value.to_string()))
        }
    }
}

impl Constraint {
    pub fn check(&self, s: &str) -> bool {
        self.range
            .contains(&s.chars().filter(|c| *c == self.c).count())
    }
}

pub struct Password {
    constraint: Constraint,
    text: String,
}

impl TryFrom<&str> for Password {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        const SEPARATOR: &str = ": ";
        let sep_ind = value
            .find(SEPARATOR)
            .ok_or(Error::InvalidPassword(value.to_string()))?;
        Ok(Password {
            constraint: Constraint::try_from(&value[..sep_ind])?,
            text: value[(sep_ind + SEPARATOR.len())..].to_string(),
        })
    }
}

impl Password {
    pub fn is_valid(&self) -> bool {
        self.constraint.check(&self.text)
    }
}

pub fn read_pws<P: AsRef<Path>>(path: P) -> Result<Vec<Password>, Error> {
    let file = File::open(path).map_err(Error::IOError)?;
    let reader = BufReader::new(file);
    reader
        .lines()
        .map(|l| {
            l.map_err(Error::IOError)
                .and_then(|s| Password::try_from(s.as_str()))
        })
        .collect::<Result<Vec<_>, Error>>()
}
