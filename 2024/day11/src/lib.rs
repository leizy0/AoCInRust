use std::{
    collections::HashMap,
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
    EmptyFile,
    InvlaidStoneText(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::EmptyFile => write!(
                f,
                "Can't read stones from empty file, expect one line in it."
            ),
            Error::InvlaidStoneText(s) => write!(f, "Invalid text({}) for stone.", s),
        }
    }
}

impl error::Error for Error {}

#[derive(Debug, Parser)]
pub struct CLIArgs {
    pub input_path: PathBuf,
}

#[derive(Debug)]
struct Stone {
    n: usize,
    next_ind: Option<usize>,
}

impl Stone {
    pub fn new(n: usize, next_ind: Option<usize>) -> Self {
        Self { n, next_ind }
    }
}

#[derive(Debug)]
pub struct StoneLine {
    stones: Vec<Stone>,
    head_ind: Option<usize>,
}

impl Display for StoneLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        let mut cur_ind_op = if let Some(first_ind) = self.head_ind {
            let first_stone = &self.stones[first_ind];
            write!(f, "{}", first_stone.n)?;
            first_stone.next_ind
        } else {
            None
        };

        while let Some(cur_ind) = cur_ind_op {
            let stone = &self.stones[cur_ind];
            write!(f, ", {}", stone.n)?;
            cur_ind_op = stone.next_ind;
        }

        write!(f, "]")
    }
}

impl TryFrom<&str> for StoneLine {
    type Error = Error;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        let mut stones = value
            .split_ascii_whitespace()
            .enumerate()
            .map(|(ind, s)| {
                s.parse::<usize>()
                    .map_err(|_| Error::InvlaidStoneText(s.to_string()))
                    .map(|n| Stone::new(n, Some(ind + 1)))
            })
            .collect::<Result<Vec<_>, _>>()?;
        if let Some(last) = stones.last_mut() {
            last.next_ind = None;
        }

        let head_ind = if stones.is_empty() { None } else { Some(0) };

        Ok(Self { stones, head_ind })
    }
}

impl StoneLine {
    pub fn blink(&mut self) {
        let mut cur_ind_op = self.head_ind;
        while let Some(cur_ind) = cur_ind_op {
            let cur_n = self.stones[cur_ind].n;
            if let Some((left, right)) = Self::split_digits(cur_n) {
                cur_ind_op = self.stones[cur_ind].next_ind;
                let right_stone = Stone::new(right, cur_ind_op);
                {
                    let right_ind = self.count();
                    let cur_stone = &mut self.stones[cur_ind];
                    cur_stone.n = left;
                    cur_stone.next_ind = Some(right_ind);
                }
                self.stones.push(right_stone);
                continue;
            }

            if cur_n == 0 {
                self.stones[cur_ind].n = 1;
            } else {
                self.stones[cur_ind].n = cur_n * 2024;
            }

            cur_ind_op = self.stones[cur_ind].next_ind;
        }
    }

    pub fn stone_n_after_blink(&self, blink_n: usize) -> usize {
        let mut cur_ind_op = self.head_ind;
        let mut count = 0;
        let mut blink_history = HashMap::<(usize, usize), usize>::new();
        while let Some(cur_ind) = cur_ind_op {
            count +=
                Self::this_stone_n_after_blink(self.stones[cur_ind].n, blink_n, &mut blink_history);
            cur_ind_op = self.stones[cur_ind].next_ind;
        }

        count
    }

    pub fn count(&self) -> usize {
        self.stones.len()
    }

    fn split_digits(n: usize) -> Option<(usize, usize)> {
        let mut digits_n = 1u32;
        let mut least_large_10_power = 10;
        while least_large_10_power <= n {
            least_large_10_power *= 10;
            digits_n += 1;
        }

        if digits_n % 2 == 0 {
            let split_factor = 10usize.pow(digits_n / 2);
            Some((n / split_factor, n % split_factor))
        } else {
            None
        }
    }

    fn this_stone_n_after_blink(
        n: usize,
        blink_n: usize,
        blink_history: &mut HashMap<(usize, usize), usize>,
    ) -> usize {
        if blink_n == 0 {
            return 1;
        } else if let Some(blink_result) = blink_history.get(&(n, blink_n)) {
            return *blink_result;
        }

        let this_blink_result = if n == 0 {
            Self::this_stone_n_after_blink(1, blink_n - 1, blink_history)
        } else if let Some((left, right)) = Self::split_digits(n) {
            Self::this_stone_n_after_blink(left, blink_n - 1, blink_history)
                + Self::this_stone_n_after_blink(right, blink_n - 1, blink_history)
        } else {
            Self::this_stone_n_after_blink(n * 2024, blink_n - 1, blink_history)
        };
        blink_history
            .entry((n, blink_n))
            .or_insert(this_blink_result);

        this_blink_result
    }
}

pub fn read_stones<P: AsRef<Path>>(path: P) -> Result<StoneLine> {
    let file = File::open(&path)
        .with_context(|| format!("Failed to open given file({}).", path.as_ref().display()))?;
    let reader = BufReader::new(file);
    reader
        .lines()
        .next()
        .ok_or(Error::EmptyFile)?
        .with_context(|| {
            format!(
                "Failed to read the first line of given file({}).",
                path.as_ref().display()
            )
        })
        .and_then(|s| {
            StoneLine::try_from(s.as_str())
                .with_context(|| format!("Failed to parse stones from given text({}).", s))
        })
}
