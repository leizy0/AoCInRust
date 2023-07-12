use std::{fmt::Display, io};

pub mod map;
pub mod sim;

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    InvalidInputChar(char),
    InconsistentInputRowSize { old_size: usize, new_size: usize },
    InvalidMapIndex(usize),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IOError(ioe) => write!(f, "I/O Error: {}", ioe),
            Error::InvalidInputChar(c) => write!(f, "Found invalid input char({})", c),
            Error::InconsistentInputRowSize { old_size, new_size } => write!(
                f,
                "Row size in input is inconsistent, it's {} before, now it's {}",
                old_size, new_size
            ),
            Error::InvalidMapIndex(ind) => write!(f, "Invalid map index({})", ind),
        }
    }
}
