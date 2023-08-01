use std::fmt::Display;

pub mod asteroid;

#[derive(Debug)]
pub enum Error {
    IOError(std::io::Error),
    InvalidCharacterInMap(usize, usize, char),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IOError(ioe) => write!(f, "I/O error: {}", ioe),
            Error::InvalidCharacterInMap(r_ind, c_ind, c) => write!(
                f,
                "Invalid character({}) found in given map, at position(x = {}, y = {})",
                c, c_ind, r_ind
            ),
        }
    }
}
