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
        let (len, _) = Self::safe_len(&self.levels, None);
        len == self.levels.len()
    }

    pub fn is_tolerantly_safe(&self) -> bool {
        fn is_safe_if_remove(levels: &[usize], ind: usize, last_is_inc_req: Option<bool>) -> bool {
            if ind == levels.len() - 1 {
                return true;
            }

            let is_inc_req = if ind == 0 {
                None
            } else {
                let last_is_inc_req = if ind == 1 { None } else { last_is_inc_req };

                let (is_safe, is_inc) =
                    Report::is_safe_pair(levels[ind - 1], levels[ind + 1], last_is_inc_req);
                if !is_safe {
                    return false;
                }

                Some(is_inc)
            };
            let left_levels = &levels[(ind + 1)..];
            let (len, _) = Report::safe_len(left_levels, is_inc_req);

            len == left_levels.len()
        }

        let (len, last_is_inc_op) = Self::safe_len(&self.levels, None);
        if len == self.levels.len() {
            return true;
        }

        // Remove the first level.
        if len == 2 && is_safe_if_remove(&self.levels, 0, last_is_inc_op) {
            return true;
        }

        // Remove the left level.
        is_safe_if_remove(&self.levels, len - 1, last_is_inc_op) ||
        // Remove the right level.
            is_safe_if_remove(&self.levels, len, last_is_inc_op)
    }

    fn safe_len(levels: &[usize], is_inc_req: Option<bool>) -> (usize, Option<bool>) {
        let level_n = levels.len();
        if level_n <= 1 {
            return (level_n, None);
        }

        let mut is_inc_op = is_inc_req;
        let mut pair_n = 0;
        for (last_l, cur_l) in levels[..(level_n - 1)].iter().zip(levels[1..].iter()) {
            let (is_safe, cur_is_inc) = Self::is_safe_pair(*last_l, *cur_l, is_inc_op);
            is_inc_op.get_or_insert(cur_is_inc);
            if !is_safe {
                break;
            }

            pair_n += 1;
        }

        (pair_n + 1, is_inc_op)
    }

    fn is_safe_pair(l_level: usize, r_level: usize, is_inc_req: Option<bool>) -> (bool, bool) {
        let cur_is_inc = l_level < r_level;
        if is_inc_req.is_some_and(|is_inc| is_inc != cur_is_inc) {
            return (false, cur_is_inc);
        }

        let abs_diff = l_level.abs_diff(r_level);
        (abs_diff >= 1 && abs_diff <= 3, cur_is_inc)
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
