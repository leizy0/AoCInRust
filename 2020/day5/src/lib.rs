use std::{
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
    InvalidBoardPassStr(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IOError(ioe) => write!(f, "I/O error: {}", ioe),
            Error::InvalidBoardPassStr(s) => write!(f, "Invalid board pass string({})", s),
        }
    }
}

impl error::Error for Error {}

#[derive(Debug, Parser)]
pub struct CliArgs {
    pub input_path: PathBuf,
}

pub struct BoardPass {
    row_ind: usize,
    col_ind: usize,
}

impl TryFrom<&str> for BoardPass {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        static PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"([F|B]{7})([L|R]{3})").unwrap());

        fn bin_from_str(s: &str, c0: char, c1: char) -> Option<usize> {
            let mut n = 0;
            for c in s.chars() {
                n <<= 1;
                let digit = if c == c0 {
                    0
                } else if c == c1 {
                    1
                } else {
                    return None;
                };

                n |= digit;
            }

            Some(n)
        }

        if let Some(caps) = PATTERN.captures(value) {
            let row_ind = bin_from_str(&caps[1], 'F', 'B').unwrap();
            let col_ind = bin_from_str(&caps[2], 'L', 'R').unwrap();
            Ok(BoardPass { row_ind, col_ind })
        } else {
            Err(Error::InvalidBoardPassStr(value.to_string()))
        }
    }
}

impl BoardPass {
    pub fn id(&self) -> usize {
        self.row_ind * 8 + self.col_ind
    }
}

pub fn read_pass<P: AsRef<Path>>(path: P) -> Result<Vec<BoardPass>, Error> {
    let file = File::open(path).map_err(Error::IOError)?;
    let reader = BufReader::new(file);
    reader
        .lines()
        .map(|l| {
            l.map_err(Error::IOError)
                .and_then(|s| BoardPass::try_from(s.as_str()))
        })
        .collect::<Result<Vec<_>, Error>>()
}
