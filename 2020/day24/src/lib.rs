use std::{
    error,
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use clap::Parser;

#[derive(Debug)]
pub enum Error {
    InvalidDirText(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidDirText(s) => write!(f, "Invalid string for direction: {}", s),
        }
    }
}

impl error::Error for Error {}

#[derive(Debug, Parser)]
pub struct CLIArgs {
    pub input_path: PathBuf,
}

// i + j + k = 0.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HexPos {
    i: isize, // + for northwest, - for southeast.
    j: isize, // + for east, - for west.
    k: isize, // + for southwest, - for northeast.
}

impl TryFrom<&str> for HexPos {
    type Error = Error;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        let mut pos = HexPos::new();
        let mut chars = value.chars();
        while let Some(c) = chars.next() {
            match c {
                // East.
                'e' => pos.j += 1,
                'n' => {
                    if let Some(next_c) = chars.next() {
                        match next_c {
                            // Northeast.
                            'e' => pos.k -= 1,
                            // Northwest.
                            'w' => pos.i += 1,
                            other => {
                                return Err(Error::InvalidDirText(String::from_iter(&[c, other])))
                            }
                        }
                    } else {
                        return Err(Error::InvalidDirText(c.to_string()));
                    }
                }
                's' => {
                    if let Some(next_c) = chars.next() {
                        match next_c {
                            // Southeast.
                            'e' => pos.i -= 1,
                            // Southwest.
                            'w' => pos.k += 1,
                            other => {
                                return Err(Error::InvalidDirText(String::from_iter(&[c, other])))
                            }
                        }
                    } else {
                        return Err(Error::InvalidDirText(c.to_string()));
                    }
                }
                // West.
                'w' => pos.j -= 1,
                other => return Err(Error::InvalidDirText(other.to_string())),
            }
        }
        pos.canonicalize();

        Ok(pos)
    }
}

impl HexPos {
    pub fn new() -> Self {
        Self { i: 0, j: 0, k: 0 }
    }

    pub fn canonicalize(&mut self) {
        let min = self.i.min(self.j.min(self.k));
        self.i -= min;
        self.j -= min;
        self.k -= min;
    }
}

pub fn read_hex_poss<P: AsRef<Path>>(path: P) -> Result<Vec<HexPos>> {
    let file = File::open(path.as_ref()).with_context(|| {
        format!(
            "Failed to open given input file: {}.",
            path.as_ref().display()
        )
    })?;
    let reader = BufReader::new(file);
    reader
        .lines()
        .enumerate()
        .map(|(ind, l)| {
            l.with_context(|| format!("Failed to read line {}.", ind + 1))
                .and_then(|s| HexPos::try_from(s.as_str()).map_err(|e| e.into()))
        })
        .collect()
}
