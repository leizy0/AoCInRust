use std::{
    collections::HashMap,
    error,
    fmt::Display,
    fs::File,
    io::{self, BufRead, BufReader},
    path::{Path, PathBuf},
};

use clap::Parser;
use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    InvalidOpText(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IOError(ioe) => write!(f, "I/O error: {}", ioe),
            Error::InvalidOpText(s) => write!(f, "Invalid operation text: {}", s),
        }
    }
}

impl error::Error for Error {}

#[derive(Debug, Parser)]
pub struct CLIArgs {
    pub input_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct Mask {
    and_mask: usize,
    or_mask: usize,
}

impl Mask {
    pub fn process(&self, v: usize) -> usize {
        v & self.and_mask | self.or_mask
    }
}

pub enum Operation {
    SetMask(Mask),
    SetMemWithMask(usize, usize),
}

impl TryFrom<&str> for Operation {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        type OpCtorFn = fn(&str) -> Option<Operation>;
        static OPERATION_CTORS: Lazy<Vec<OpCtorFn>> = Lazy::new(|| {
            vec![
                Operation::try_new_set_mask,
                Operation::try_new_set_mem_with_mask,
            ]
        });

        OPERATION_CTORS
            .iter()
            .filter_map(|f| f(value))
            .next()
            .ok_or(Error::InvalidOpText(value.to_string()))
    }
}

impl Operation {
    pub fn try_new_set_mask(text: &str) -> Option<Self> {
        static PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"mask = ([10Xx]{36})").unwrap());

        PATTERN.captures(text).map(|caps| {
            let mut and_mask = 0;
            let mut or_mask = 0;
            for c in caps[1].chars() {
                and_mask <<= 1;
                or_mask <<= 1;
                match c {
                    '1' => {
                        or_mask |= 1;
                    }
                    '0' => (),
                    'X' | 'x' => {
                        and_mask |= 1;
                    }
                    invalid_char => unreachable!(
                        "Invalid character({}) pass regular expression!",
                        invalid_char
                    ),
                }
            }

            Operation::SetMask(Mask { and_mask, or_mask })
        })
    }

    pub fn try_new_set_mem_with_mask(text: &str) -> Option<Operation> {
        static PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"mem\[(\d+)\] = (\d+)").unwrap());

        PATTERN.captures(text).map(|caps| {
            Operation::SetMemWithMask(
                caps[1].parse::<usize>().unwrap(),
                caps[2].parse::<usize>().unwrap(),
            )
        })
    }
}

pub struct SPCSimulator {
    mem: HashMap<usize, usize>,
    mask: Mask,
}

impl SPCSimulator {
    pub fn new() -> Self {
        Self {
            mem: HashMap::new(),
            mask: Mask {
                and_mask: 0,
                or_mask: 0,
            },
        }
    }

    pub fn execute(&mut self, op: &Operation) {
        match op {
            Operation::SetMask(new_mask) => self.mask = new_mask.clone(),
            Operation::SetMemWithMask(ind, value) => {
                self.mem.insert(*ind, self.mask.process(*value));
            }
        }
    }

    pub fn non_zero_mem(&self) -> Vec<(usize, usize)> {
        self.mem.iter().map(|(k, v)| (*k, *v)).collect()
    }
}

pub fn read_ops<P: AsRef<Path>>(path: P) -> Result<Vec<Operation>, Error> {
    let file = File::open(path).map_err(Error::IOError)?;
    let reader = BufReader::new(file);
    reader
        .lines()
        .map(|l| {
            l.map_err(Error::IOError)
                .and_then(|s| Operation::try_from(s.as_str()))
        })
        .collect()
}
