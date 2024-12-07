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
    NoColonInEquation,
    InvalidResultText(String),
    InvalidOprandText(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NoColonInEquation => write!(
                f,
                "Can't find sperator(:) in equation text that separates the result and oprands."
            ),
            Error::InvalidResultText(s) => write!(f, "Invalid reuslt text({}).", s),
            Error::InvalidOprandText(s) => write!(f, "Invalid oprand text({}).", s),
        }
    }
}

impl error::Error for Error {}

#[derive(Debug, Parser)]
pub struct CLIArgs {
    pub input_path: PathBuf,
}

#[derive(Debug)]
pub struct Equation {
    result: usize,
    oprands: Vec<usize>,
}

impl TryFrom<&str> for Equation {
    type Error = Error;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        let colon_pos = value.find(':').ok_or(Error::NoColonInEquation)?;
        let result = value[..colon_pos]
            .parse::<usize>()
            .map_err(|_| Error::InvalidResultText(value[..colon_pos].to_string()))?;
        let oprands = value[(colon_pos + 1)..]
            .trim()
            .split_ascii_whitespace()
            .map(|s| {
                s.parse::<usize>()
                    .map_err(|_| Error::InvalidOprandText(s.to_string()))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { result, oprands })
    }
}

impl Equation {
    pub fn is_possible(&self) -> bool {
        let mut rev_oprands = self.oprands.clone();
        rev_oprands.reverse();
        Self::can_reach(self.result, &rev_oprands)
    }

    pub fn result(&self) -> usize {
        self.result
    }

    fn can_reach(target: usize, oprands: &[usize]) -> bool {
        if let Some(oprand) = oprands.first() {
            if target < *oprand {
                return false;
            }

            Self::can_reach(target - oprand, &oprands[1..])
                || (target % oprand == 0 && Self::can_reach(target / oprand, &oprands[1..]))
        } else {
            target == 0
        }
    }
}

pub fn read_equations<P: AsRef<Path>>(path: P) -> Result<Vec<Equation>> {
    let file = File::open(&path)
        .with_context(|| format!("Failed to open given file({}).", path.as_ref().display()))?;
    let reader = BufReader::new(file);
    reader
        .lines()
        .enumerate()
        .map(|(ind, l)| {
            l.with_context(|| {
                format!(
                    "Failed to read line {} in given file({}).",
                    ind + 1,
                    path.as_ref().display()
                )
            })
            .and_then(|s| {
                Equation::try_from(s.as_str())
                    .with_context(|| format!("Failed to parse equation from given string({}).", s))
            })
        })
        .collect()
}
