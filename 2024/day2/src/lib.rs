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
    InvalidChar(char),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidChar(c) => {
                write!(f, "Invalid character {} found in given level list.", c)
            }
        }
    }
}

impl error::Error for Error {}

#[derive(Debug, Parser)]
pub struct CLIArgs {
    pub input_path: PathBuf,
}

#[derive(Debug)]
pub struct Report {
    levels: Vec<usize>,
}

impl TryFrom<&str> for Report {
    type Error = Error;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        let mut levels = Vec::new();
        let mut cur_level = None;
        for c in value.chars() {
            if c.is_ascii_whitespace() {
                if let Some(level) = cur_level.take() {
                    levels.push(level);
                }

                continue;
            }

            let digit = c.to_digit(10).ok_or(Error::InvalidChar(c))?;
            cur_level = Some(cur_level.unwrap_or(0) * 10 + digit as usize);
        }

        if let Some(level) = cur_level.take() {
            levels.push(level);
        }

        Ok(Self { levels })
    }
}

impl Report {
    pub fn is_safe(&self) -> bool {
        let level_n = self.levels.len();
        if level_n <= 1 {
            return true;
        }

        let mut is_inc_op = None;
        for (last_l, cur_l) in self.levels[..(level_n - 1)]
            .iter()
            .zip(self.levels[1..].iter())
        {
            let cur_is_inc = last_l < cur_l;
            if *is_inc_op.get_or_insert(cur_is_inc) == cur_is_inc {
                let abs_diff = last_l.abs_diff(*cur_l);
                if abs_diff >= 1 && abs_diff <= 3 {
                    continue;
                }
            }

            return false;
        }

        true
    }
}

pub fn read_reps<P: AsRef<Path>>(path: P) -> Result<Vec<Report>> {
    let file = File::open(&path)
        .with_context(|| format!("Failed to open given file({}).", path.as_ref().display()))?;
    let reader = BufReader::new(file);

    reader
        .lines()
        .enumerate()
        .map(|(ind, l)| {
            l.with_context(|| {
                format!(
                    "Failed to read line {} from given file({})",
                    ind,
                    path.as_ref().display()
                )
            })
            .and_then(|s| {
                Report::try_from(s.as_str())
                    .with_context(|| format!("Failed to read levels from given string({}).", s))
            })
        })
        .collect()
}
