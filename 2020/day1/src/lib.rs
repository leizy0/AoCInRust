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
    InvalidInputStr(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IOError(ioe) => write!(f, "I/O error: {}", ioe),
            Error::InvalidInputStr(s) => write!(
                f,
                "Invalid string({}) found in input, expect unsigned integers.",
                s
            ),
        }
    }
}

impl error::Error for Error {}

#[derive(Debug, Parser)]
pub struct CliArgs {
    pub input_path: PathBuf,
}

pub fn read_ints<P: AsRef<Path>>(path: P) -> Result<Vec<usize>, Error> {
    let file = File::open(path).map_err(Error::IOError)?;
    let reader = BufReader::new(file);
    reader
        .lines()
        .map(|l| {
            l.map_err(Error::IOError).and_then(|s| {
                s.parse::<usize>()
                    .map_err(|_| Error::InvalidInputStr(s.clone()))
            })
        })
        .collect::<Result<Vec<_>, Error>>()
}

pub fn find_ints_of_sum(sorted_ints: &[usize], sum: usize, ints_n: usize) -> Option<Vec<usize>> {
    let mut res = vec![0; ints_n];
    if find_ints_of_sum_recur(sorted_ints, sum, &mut res, 0) {
        Some(res)
    } else {
        None
    }
}

fn find_ints_of_sum_recur(
    sorted_ints: &[usize],
    sum: usize,
    ns: &mut [usize],
    n_ind: usize,
) -> bool {
    if sorted_ints.is_empty() || ns.is_empty() {
        return false;
    }

    if n_ind + 1 == ns.len() {
        // Find the final number.
        if let Ok(_) = sorted_ints.binary_search(&sum) {
            ns[n_ind] = sum;
            return true;
        } else {
            return false;
        }
    }

    let limit = sum / (ns.len() - n_ind);
    for (ind, n) in sorted_ints.iter().enumerate() {
        if *n > limit {
            // Left numbers are greater than the average of the current sum, so there's no chance to find other numbers in given ascending integers.
            break;
        }

        ns[n_ind] = *n;
        if find_ints_of_sum_recur(&sorted_ints[(ind + 1)..], sum - n, ns, n_ind + 1) {
            // Found left numbers.
            return true;
        }
    }

    return false;
}
