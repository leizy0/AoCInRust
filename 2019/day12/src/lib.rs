use std::fmt::Display;

pub mod n_body;

pub enum Error {
    IOError(std::io::Error),
    InvalidBodyDescription(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IOError(ioe) => write!(f, "I/O Error: {}", ioe),
            Error::InvalidBodyDescription(s) => write!(f, "Invalid body Description({})", s),
        }
    }
}
