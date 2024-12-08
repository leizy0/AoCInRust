use std::{
    collections::{HashMap, HashSet},
    error,
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader},
    ops::{Add, Neg, Sub},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use clap::Parser;

#[derive(Debug)]
enum Error {
    InconsistentRow(usize, usize),
    InvalidCharInSignalMap(char),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InconsistentRow(expect_col_n, this_col_n) => write!(
                f,
                "Expect {} columns in one row, given {}.",
                expect_col_n, this_col_n
            ),
            Error::InvalidCharInSignalMap(c) => write!(
                f,
                "Invalid character({}) in signal map, expect letters or digits.",
                c
            ),
        }
    }
}

impl error::Error for Error {}

#[derive(Debug, Parser)]
pub struct CLIArgs {
    pub input_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Position {
    r: usize,
    c: usize,
}

impl Sub for Position {
    type Output = Vector;

    fn sub(self, rhs: Self) -> Self::Output {
        &self - &rhs
    }
}

impl Sub<Position> for &Position {
    type Output = Vector;

    fn sub(self, rhs: Position) -> Self::Output {
        self - &rhs
    }
}

impl Sub<&Position> for Position {
    type Output = Vector;

    fn sub(self, rhs: &Position) -> Self::Output {
        &self - rhs
    }
}

impl Sub for &Position {
    type Output = Vector;

    fn sub(self, rhs: Self) -> Self::Output {
        Vector::new(
            isize::try_from(self.r).unwrap() - isize::try_from(rhs.r).unwrap(),
            isize::try_from(self.c).unwrap() - isize::try_from(rhs.c).unwrap(),
        )
    }
}

impl Add<&Vector> for &Position {
    type Output = Option<Position>;

    fn add(self, rhs: &Vector) -> Self::Output {
        let r = isize::try_from(self.r).unwrap() + rhs.r;
        let c = isize::try_from(self.c).unwrap() + rhs.c;

        if r >= 0 && c >= 0 {
            Some(Position::new(
                usize::try_from(r).unwrap(),
                usize::try_from(c).unwrap(),
            ))
        } else {
            None
        }
    }
}

impl Add<Vector> for &Position {
    type Output = Option<Position>;

    fn add(self, rhs: Vector) -> Self::Output {
        self + &rhs
    }
}

impl Add<&Vector> for Position {
    type Output = Option<Position>;

    fn add(self, rhs: &Vector) -> Self::Output {
        &self + rhs
    }
}

impl Add<Vector> for Position {
    type Output = Option<Position>;

    fn add(self, rhs: Vector) -> Self::Output {
        &self + &rhs
    }
}

impl Sub<&Vector> for &Position {
    type Output = Option<Position>;

    fn sub(self, rhs: &Vector) -> Self::Output {
        self + (-rhs)
    }
}

impl Sub<Vector> for &Position {
    type Output = Option<Position>;

    fn sub(self, rhs: Vector) -> Self::Output {
        self - &rhs
    }
}

impl Sub<&Vector> for Position {
    type Output = Option<Position>;

    fn sub(self, rhs: &Vector) -> Self::Output {
        &self - rhs
    }
}

impl Sub<Vector> for Position {
    type Output = Option<Position>;

    fn sub(self, rhs: Vector) -> Self::Output {
        &self - &rhs
    }
}

impl Position {
    pub fn new(r: usize, c: usize) -> Self {
        Self { r, c }
    }
}

#[derive(Debug, Clone)]
pub struct Vector {
    r: isize,
    c: isize,
}

impl Neg for Vector {
    type Output = Vector;

    fn neg(self) -> Self::Output {
        -&self
    }
}

impl Neg for &Vector {
    type Output = Vector;

    fn neg(self) -> Self::Output {
        Vector::new(-self.r, -self.c)
    }
}

impl Vector {
    pub fn new(r: isize, c: isize) -> Self {
        Self { r, c }
    }
}

pub struct SignalMap {
    signals: HashMap<char, Vec<Position>>,
    row_n: usize,
    col_n: usize,
}

impl SignalMap {
    pub fn antinode_positions(&self) -> HashSet<Position> {
        let mut res_positions = HashSet::new();
        for (_, positions) in &self.signals {
            let pos_n = positions.len();
            for pos0_ind in 0..pos_n {
                for pos1_ind in (pos0_ind + 1)..pos_n {
                    let pos0 = &positions[pos0_ind];
                    let pos1 = &positions[pos1_ind];
                    let offset = pos1 - pos0;
                    res_positions.extend(
                        [pos0 - &offset, pos1 + &offset]
                            .iter()
                            .flatten()
                            .filter(|p| self.is_inside(p))
                            .cloned(),
                    );
                }
            }
        }

        res_positions
    }

    fn is_inside(&self, pos: &Position) -> bool {
        pos.r < self.row_n && pos.c < self.col_n
    }
}

struct SignalMapBuilder {
    signals: HashMap<char, Vec<Position>>,
    row_n: usize,
    col_n: Option<usize>,
}

impl SignalMapBuilder {
    pub fn new() -> Self {
        Self {
            signals: HashMap::new(),
            row_n: 0,
            col_n: None,
        }
    }

    pub fn add_row(&mut self, text: &str) -> Result<(), Error> {
        let this_col_n = text.chars().count();
        if *self.col_n.get_or_insert(this_col_n) != this_col_n {
            return Err(Error::InconsistentRow(self.col_n.unwrap(), this_col_n));
        }

        for (col_ind, c) in text.chars().enumerate().filter(|(_, c)| *c != '.') {
            if !c.is_ascii_alphanumeric() {
                return Err(Error::InvalidCharInSignalMap(c));
            }

            self.signals
                .entry(c)
                .or_insert_with(|| Vec::new())
                .push(Position::new(self.row_n, col_ind));
        }
        self.row_n += 1;

        Ok(())
    }

    pub fn build(self) -> SignalMap {
        SignalMap {
            signals: self.signals,
            row_n: self.row_n,
            col_n: self.col_n.unwrap_or(0),
        }
    }
}

pub fn read_signal_map<P: AsRef<Path>>(path: P) -> Result<SignalMap> {
    let file = File::open(&path)
        .with_context(|| format!("Failed to open given file({}).", path.as_ref().display()))?;
    let reader = BufReader::new(file);
    let mut builder = SignalMapBuilder::new();

    for (ind, line) in reader.lines().enumerate() {
        let line = line.with_context(|| {
            format!(
                "Failed to read line {} of given file({}).",
                ind + 1,
                path.as_ref().display()
            )
        })?;
        builder.add_row(line.as_str())?;
    }

    Ok(builder.build())
}
