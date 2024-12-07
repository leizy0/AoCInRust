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

#[derive(Debug, Clone, Copy)]
pub enum Operator {
    Plus,
    Multiply,
    Concatenation,
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

type EquationTestFn = fn(usize, usize) -> Option<usize>;

impl Equation {
    pub fn is_possible(&self, test_operators: &[Operator]) -> bool {
        let mut rev_oprands = self.oprands.clone();
        rev_oprands.reverse();

        let test_fns = test_operators
            .iter()
            .map(|op| match op {
                Operator::Plus => Self::new_target_if_plus,
                Operator::Multiply => Self::new_target_if_multiply,
                Operator::Concatenation => Self::new_target_if_concat,
            })
            .collect::<Vec<_>>();

        Self::can_reach(self.result, &rev_oprands, &test_fns)
    }

    pub fn result(&self) -> usize {
        self.result
    }

    fn can_reach(target: usize, oprands: &[usize], test_fns: &[EquationTestFn]) -> bool {
        if oprands.len() == 1 {
            return target == oprands[0];
        }

        if let Some(oprand) = oprands.first() {
            if target < *oprand {
                return false;
            }

            test_fns.iter().any(|f| {
                f(target, *oprand)
                    .map(|new_target| Self::can_reach(new_target, &oprands[1..], test_fns))
                    .unwrap_or(false)
            })
        } else {
            target == 0
        }
    }

    fn new_target_if_plus(target: usize, oprand0: usize) -> Option<usize> {
        Some(target - oprand0)
    }

    fn new_target_if_multiply(target: usize, oprand0: usize) -> Option<usize> {
        if target % oprand0 == 0 {
            Some(target / oprand0)
        } else {
            None
        }
    }

    fn new_target_if_concat(target: usize, oprand0: usize) -> Option<usize> {
        let mut min_10_power = 10;
        while min_10_power <= oprand0 {
            min_10_power *= 10;
        }

        if target % min_10_power == oprand0 {
            Some(target / min_10_power)
        } else {
            None
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
