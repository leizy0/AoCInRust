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
enum Error {
    InconsistentRow(usize, usize),
    InvalidCharInSchemetic(char),
    KeyAndLockSchemetic(SchemeticBuilder),
    NeitherKeyNorLockSchemetic(SchemeticBuilder),
    BrokenKeySchemetic(SchemeticBuilder),
    BrokenLockSchemetic(SchemeticBuilder),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InconsistentRow(expect_col_n, this_col_n) => write!(f, "Expect {} col(s) in each row, but given {}.", expect_col_n, this_col_n),
            Error::InvalidCharInSchemetic(c) => write!(f, "Invalid character({}) found in given schemetic.", c),
            Error::KeyAndLockSchemetic(schemetic_builder) => write!(f, "Given {:?} should be a key or a lock, not both.", schemetic_builder),
            Error::NeitherKeyNorLockSchemetic(schemetic_builder) => write!(f, "Given {:?} can't be a key neither a lock, should be one of them.", schemetic_builder),
            Error::BrokenKeySchemetic(schemetic_builder) => write!(f, "Given {:?} is a broken key which has at least one inconsecutive column from bottom up.", schemetic_builder),
            Error::BrokenLockSchemetic(schemetic_builder) => write!(f, "Given {:?} is a broken lock which has at least one inconsecutive column from top down.", schemetic_builder),
        }
    }
}

impl error::Error for Error {}

#[derive(Debug, Parser)]
pub struct CLIArgs {
    pub input_path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemeticKind {
    Key,
    Lock,
}

#[derive(Debug, Clone)]
pub struct Schemetic {
    heights: Vec<usize>,
    row_n: usize,
    col_n: usize,
    kind: SchemeticKind,
}

impl Schemetic {
    pub fn kind(&self) -> SchemeticKind {
        self.kind
    }

    pub fn fit(&self, other: &Self) -> bool {
        if self.kind() == other.kind() || self.col_n != other.col_n {
            return false;
        }

        let max_height = self.row_n.min(other.row_n);
        (0..self.col_n).all(|col_ind| self.heights[col_ind] + other.heights[col_ind] <= max_height)
    }
}

#[derive(Debug)]
struct SchemeticBuilder {
    tiles: Vec<bool>,
    row_n: usize,
    col_n: Option<usize>,
}

impl SchemeticBuilder {
    pub fn new() -> Self {
        Self {
            tiles: Vec::new(),
            row_n: 0,
            col_n: None,
        }
    }

    pub fn add_row(&mut self, text: &str) -> Result<(), Error> {
        let this_col_n = text.chars().count();
        if *self.col_n.get_or_insert(this_col_n) != this_col_n {
            return Err(Error::InconsistentRow(self.col_n.unwrap(), this_col_n));
        }

        for c in text.chars() {
            self.tiles.push(match c {
                '.' => false,
                '#' => true,
                other => return Err(Error::InvalidCharInSchemetic(other)),
            });
        }
        self.row_n += 1;

        Ok(())
    }

    pub fn build(self) -> Result<Schemetic, Error> {
        let row_n = self.row_n;
        let col_n = self.col_n.unwrap_or(0);
        let is_lock = (0..col_n).all(|ind| self.tiles[ind]);
        let last_row_start_ind = (self.row_n - 1) * col_n;
        let is_key = (last_row_start_ind..(last_row_start_ind + col_n)).all(|ind| self.tiles[ind]);
        let (kind, heights) = if is_key && is_lock {
            return Err(Error::KeyAndLockSchemetic(self));
        } else if !is_key && !is_lock {
            return Err(Error::NeitherKeyNorLockSchemetic(self));
        } else if is_key {
            (
                SchemeticKind::Key,
                self.heights_on_rows((0..self.row_n).rev())
                    .ok_or(Error::BrokenKeySchemetic(self))?,
            )
        } else {
            (
                SchemeticKind::Lock,
                self.heights_on_rows(0..self.row_n)
                    .ok_or(Error::BrokenLockSchemetic(self))?,
            )
        };

        Ok(Schemetic {
            heights,
            row_n,
            col_n,
            kind,
        })
    }

    fn heights_on_rows(&self, row_iter: impl Iterator<Item = usize> + Clone) -> Option<Vec<usize>> {
        let col_n = self.col_n.unwrap_or(0);
        let mut heights = vec![0; col_n];
        for col_ind in 0..col_n {
            let mut height_ended = false;
            for row_ind in row_iter.clone() {
                let tile_ind = row_ind * col_n + col_ind;
                if !height_ended {
                    if self.tiles[tile_ind] {
                        heights[col_ind] += 1;
                    } else {
                        height_ended = true;
                    }
                } else {
                    if self.tiles[tile_ind] {
                        return None;
                    }
                }
            }
        }

        Some(heights)
    }
}

pub fn read_key_lock<P: AsRef<Path>>(path: P) -> Result<(Vec<Schemetic>, Vec<Schemetic>)> {
    let file = File::open(&path)
        .with_context(|| format!("Failed to open given file({}).", path.as_ref().display()))?;
    let reader = BufReader::new(file);
    let mut cur_builder: Option<SchemeticBuilder> = None;
    let mut keys = Vec::new();
    let mut locks = Vec::new();
    for (ind, line) in reader.lines().enumerate() {
        let line = line.with_context(|| {
            format!(
                "Failed to read line {} in given file({}).",
                ind + 1,
                path.as_ref().display()
            )
        })?;
        if line.is_empty() {
            if let Some(builder) = cur_builder.take() {
                let schemetic = builder.build()?;
                match schemetic.kind() {
                    SchemeticKind::Key => keys.push(schemetic),
                    SchemeticKind::Lock => locks.push(schemetic),
                }
            }

            continue;
        }

        cur_builder
            .get_or_insert_with(|| SchemeticBuilder::new())
            .add_row(line.as_str())?;
    }

    if let Some(builder) = cur_builder.take() {
        let schemetic = builder.build()?;
        match schemetic.kind() {
            SchemeticKind::Key => keys.push(schemetic),
            SchemeticKind::Lock => locks.push(schemetic),
        }
    }

    Ok((keys, locks))
}
