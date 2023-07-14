use std::fmt::Display;

pub mod int_code;

#[derive(Debug)]
pub enum Error {
    IOError(std::io::Error),
    EmptyError,
    ParseIntError(String),
    IndexError(usize),
    InvalidOpCode(u32),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IOError(ioe) => write!(f, "I/O Error: {}", ioe),
            Error::EmptyError => write!(f, "Get empty code in file"),
            Error::ParseIntError(s) => write!(f, "Failed to parse integer from string({})", s),
            Error::IndexError(i) => write!(f, "Invalid index({}) found in execution", i),
            Error::InvalidOpCode(c) => {
                write!(f, "Invalid operation code({}) found in execution", c)
            }
        }
    }
}
