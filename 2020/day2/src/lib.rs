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
    ZeroIndexInCon2(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IOError(ioe) => write!(f, "I/O error: {}", ioe),
            Error::InvalidPassword(s) => write!(f, "Invalid password: {}", s),
            Error::InvalidConstraint(s) => write!(f, "Invalid constraint: {}", s),
            Error::ZeroIndexInCon2(s) => {
                write!(
                    f,
                    "Found zero index while parsing text({}) for Constraint2.",
                    s
                )
            }
        }
    }
}

impl error::Error for Error {}

#[derive(Debug, Parser)]
pub struct CliArgs {
    pub input_path: String,
}

pub trait Constraint: for<'a> TryFrom<&'a str, Error = Error> {
    fn check(&self, s: &str) -> bool;
}

pub struct Constraint1 {
    c: char,
    range: Range<usize>,
}

impl TryFrom<&str> for Constraint1 {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        static PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\d+)-(\d+) (.)").unwrap());
        if let Some(caps) = PATTERN.captures(value) {
            let low_bound = caps[1].parse::<usize>().unwrap();
            let high_bound = caps[2].parse::<usize>().unwrap();
            let c = caps[3].chars().next().unwrap();

            Ok(Constraint1 {
                c,
                range: low_bound..(high_bound + 1),
            })
        } else {
            Err(Error::InvalidConstraint(value.to_string()))
        }
    }
}

impl Constraint for Constraint1 {
    fn check(&self, s: &str) -> bool {
        self.range
            .contains(&s.chars().filter(|c| *c == self.c).count())
    }
}

pub struct Constraint2 {
    c: char,
    ind0: usize,
    ind1: usize,
}

impl TryFrom<&str> for Constraint2 {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        static PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\d+)-(\d+) (.)").unwrap());
        if let Some(caps) = PATTERN.captures(value) {
            let ind0 = caps[1]
                .parse::<usize>()
                .unwrap()
                .checked_sub(1)
                .ok_or(Error::ZeroIndexInCon2(value.to_string()))?;
            let ind1 = caps[2]
                .parse::<usize>()
                .unwrap()
                .checked_sub(1)
                .ok_or(Error::ZeroIndexInCon2(value.to_string()))?;
            let c = caps[3].chars().next().unwrap();

            Ok(Constraint2 { c, ind0, ind1 })
        } else {
            Err(Error::InvalidConstraint(value.to_string()))
        }
    }
}

impl Constraint for Constraint2 {
    fn check(&self, s: &str) -> bool {
        fn is_at(target_c: char, s: &str, ind: usize) -> bool {
            s.chars().nth(ind).map(|c| c == target_c).unwrap_or(false)
        }

        let is_at0 = is_at(self.c, s, self.ind0);
        let is_at1 = is_at(self.c, s, self.ind1);
        is_at0 ^ is_at1
    }
}

pub struct Password<C: Constraint> {
    constraint: C,
    text: String,
}

impl<C: Constraint> TryFrom<&str> for Password<C> {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        const SEPARATOR: &str = ": ";
        let sep_ind = value
            .find(SEPARATOR)
            .ok_or(Error::InvalidPassword(value.to_string()))?;
        Ok(Password {
            constraint: C::try_from(&value[..sep_ind])?,
            text: value[(sep_ind + SEPARATOR.len())..].to_string(),
        })
    }
}

impl<C: Constraint> Password<C> {
    pub fn is_valid(&self) -> bool {
        self.constraint.check(&self.text)
    }
}

pub fn read_pws<C: Constraint, P: AsRef<Path>>(path: P) -> Result<Vec<Password<C>>, Error> {
    let file = File::open(path).map_err(Error::IOError)?;
    let reader = BufReader::new(file);
    reader
        .lines()
        .map(|l| {
            l.map_err(Error::IOError)
                .and_then(|s| Password::<C>::try_from(s.as_str()))
        })
        .collect::<Result<Vec<_>, Error>>()
}
