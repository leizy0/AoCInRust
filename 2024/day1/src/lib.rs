use std::{
    error,
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use anyhow::{Context, Error};
use clap::Parser;

#[derive(Debug)]
enum Day1Error {
    InvalidChar(char),
    NoneDigits(String, usize),
}

impl Display for Day1Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Day1Error::InvalidChar(c) => write!(f, "Invalid character({}) in location ID list.", c),
            Day1Error::NoneDigits(s, start_ind) => write!(
                f,
                "Given string({}) start from {}th character doesn't have any digits.",
                s, start_ind
            ),
        }
    }
}

impl error::Error for Day1Error {}

#[derive(Debug, Parser)]
pub struct CLIArgs {
    pub input_path: PathBuf,
}

pub fn read_lists<P: AsRef<Path>>(path: P) -> Result<(Vec<usize>, Vec<usize>), Error> {
    let file = File::open(&path)
        .with_context(|| format!("Failed to open given file({})", path.as_ref().display()))?;
    let reader = BufReader::new(file);
    let mut list0 = Vec::new();
    let mut list1 = Vec::new();
    for (ind, line) in reader.lines().enumerate() {
        let s = line.with_context(|| {
            format!(
                "Failed to read line #{} of given file({}).",
                ind + 1,
                path.as_ref().display()
            )
        })?;
        let (id0, id0_len) = read_num(&s, 0)
            .with_context(|| format!("Failed to read the first location ID in string({}).", s))?;
        let (id1, _) = read_num(&s, id0_len)
            .with_context(|| format!("Failed to read the second location ID in string({}).", s))?;
        list0.push(id0);
        list1.push(id1);
    }

    Ok((list0, list1))
}

fn read_num(s: &str, start_ind: usize) -> Result<(usize, usize), Day1Error> {
    let mut n_op = None;
    let mut len = 0;
    for c in s.chars().skip(start_ind) {
        if c.is_ascii_whitespace() {
            if n_op.is_some() {
                break;
            }
        } else {
            let digit = c.to_digit(10).ok_or(Day1Error::InvalidChar(c))?;
            n_op = Some(n_op.unwrap_or(0) * 10 + digit as usize);
        }

        len += 1;
    }

    n_op.ok_or(Day1Error::NoneDigits(s.to_string(), start_ind))
        .map(|n| (n, len))
}
