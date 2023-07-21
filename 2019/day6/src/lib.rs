use std::fmt::Display;

use orbit::Object;

pub mod orbit;

#[derive(Debug)]
pub enum Error {
    IOError(std::io::Error),
    InvalidOrbitSpec(String),
    RewriteOrbitLink(Object, Object, Object),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IOError(ioe) => write!(f, "I/O Error: {}", ioe),
            Error::InvalidOrbitSpec(sp) => write!(f, "Invalid orbit specification({})", sp),
            Error::RewriteOrbitLink(orbitor, orbited, new_orbited) => write!(
                f,
                "Can't rewrite orbited object(from {} to {}) of object({})",
                orbited, new_orbited, orbitor
            ),
        }
    }
}
