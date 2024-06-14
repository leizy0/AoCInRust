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
    InvalidNumberText(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IOError(ioe) => write!(f, "I/O error: {}", ioe),
            Error::InvalidNumberText(s) => {
                write!(f, "Invalid text: {}, expect a non-negative number", s)
            }
        }
    }
}

impl error::Error for Error {}

#[derive(Debug, Parser)]
pub struct CliArgs {
    pub input_path: PathBuf,
}

pub fn read_num<P: AsRef<Path>>(path: P) -> Result<Vec<usize>, Error> {
    let file = File::open(path).map_err(Error::IOError)?;
    let reader = BufReader::new(file);
    reader
        .lines()
        .map(|l| {
            l.map_err(Error::IOError).and_then(|s| {
                s.parse::<usize>()
                    .map_err(|_| Error::InvalidNumberText(s.clone()))
            })
        })
        .collect::<Result<Vec<_>, Error>>()
}
