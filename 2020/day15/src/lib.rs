use std::{
    collections::HashMap,
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
    EmptyInupt,
    InvalidNumText(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IOError(ioe) => write!(f, "I/O error: {}", ioe),
            Error::EmptyInupt => write!(f, "Given empty input"),
            Error::InvalidNumText(s) => {
                write!(f, "Invalid number text({}), expect non-negative number", s)
            }
        }
    }
}

impl error::Error for Error {}

#[derive(Debug, Parser)]
pub struct CLIArgs {
    pub input_path: PathBuf,
}

pub struct MemGame {
    nums: Vec<usize>,
    recent_inds: HashMap<usize, usize>,
}

impl MemGame {
    pub fn new(start_nums: &[usize]) -> Self {
        let nums = Vec::from(start_nums);
        let recent_inds = start_nums
            .iter()
            .enumerate()
            .map(|(ind, n)| (*n, ind))
            .collect::<HashMap<usize, usize>>();
        Self { nums, recent_inds }
    }

    pub fn nth(&mut self, ind: usize) -> usize {
        let mut cur_ind = self.nums.len() - 1;
        while ind > cur_ind {
            let n = self.nums[cur_ind];
            let recent_ind = *self.recent_inds.entry(n).or_insert(cur_ind);
            let new_n = if cur_ind == recent_ind {
                // First time spoken.
                0
            } else {
                self.recent_inds.insert(n, cur_ind);
                cur_ind - recent_ind
            };
            self.nums.push(new_n);
            cur_ind += 1;
        }

        self.nums[ind]
    }
}

pub fn read_nums<P: AsRef<Path>>(path: P) -> Result<Vec<usize>, Error> {
    let file = File::open(path).map_err(Error::IOError)?;
    let reader = BufReader::new(file);
    reader
        .lines()
        .next()
        .ok_or(Error::EmptyInupt)
        .and_then(|l| {
            l.map_err(Error::IOError).and_then(|s| {
                s.split(',')
                    .map(|ns| {
                        ns.parse::<usize>()
                            .map_err(|_| Error::InvalidNumText(ns.to_string()))
                    })
                    .collect::<Result<Vec<_>, Error>>()
            })
        })
}
