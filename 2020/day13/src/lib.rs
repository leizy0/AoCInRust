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

pub fn read_notes<P: AsRef<Path>>(path: P) -> Result<(usize, Vec<usize>), Error> {
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
                    .filter_map(|sch_s| sch_s.parse::<usize>().ok())
                    .collect::<Vec<_>>()
            })
        })?;
    Ok((depart_time, bus_schedules))
}
