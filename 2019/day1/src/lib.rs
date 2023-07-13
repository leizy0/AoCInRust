use std::fmt::Display;

pub mod mass;

#[derive(Debug)]
pub enum Error {
    IOError(std::io::Error),
    ParseIntError(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IOError(ioe) => write!(f, "I/O Error: {}", ioe),
            Self::ParseIntError(s) => write!(f, "Failed to parset integer fro string: {}", s),
        }
    }
}
