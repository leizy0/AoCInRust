use std::{
    error,
    fmt::Display,
    fs::File,
    io::{self, BufRead, BufReader},
    ops::AddAssign,
    path::{Path, PathBuf},
};

use clap::Parser;

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    InconsistentRow(usize, usize), // (column number of the current row, column number of earlier row)
    InvalidTileCharacter(char),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IOError(ioe) => write!(f, "I/O error: {}", ioe),
            Error::InconsistentRow(this_col_n, expect_col_n) => write!(
                f,
                "Inconsistent column number({}) found, expect {} as earlier rows did.",
                this_col_n, expect_col_n
            ),
            Error::InvalidTileCharacter(c) => {
                write!(f, "Found invalid character({}) for tile in map.", c)
            }
        }
    }
}

impl error::Error for Error {}

#[derive(Debug, Parser)]
pub struct CliArgs {
    pub input_path: PathBuf,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TileType {
    Empty,
    Tree,
}

// Assume only has direction right down.
#[derive(Debug, Clone)]
pub struct Direction {
    delta_r: usize,
    delta_c: usize,
}

impl Direction {
    pub fn new(delta_r: usize, delta_c: usize) -> Self {
        Self { delta_r, delta_c }
    }
}

#[derive(Debug, Clone)]
pub struct Position {
    r: usize,
    c: usize,
}

impl AddAssign<&Direction> for Position {
    fn add_assign(&mut self, rhs: &Direction) {
        self.r += rhs.delta_r;
        self.c += rhs.delta_c;
    }
}

impl AddAssign<Direction> for Position {
    fn add_assign(&mut self, rhs: Direction) {
        *self += &rhs;
    }
}

impl Position {
    pub fn new(r: usize, c: usize) -> Self {
        Self { r, c }
    }
}

pub struct Map {
    tiles: Vec<TileType>,
    row_n: usize,
    col_n: usize,
}

impl Map {
    pub fn tile(&self, pos: &Position) -> Option<&TileType> {
        self.pos_to_ind(pos).and_then(|ind| self.tiles.get(ind))
    }

    fn pos_to_ind(&self, pos: &Position) -> Option<usize> {
        if pos.r >= self.row_n {
            None
        } else {
            Some(pos.r * self.col_n + pos.c % self.col_n)
        }
    }
}

struct MapBuilder {
    tiles: Vec<TileType>,
    col_n: Option<usize>,
    row_n: usize,
}

impl MapBuilder {
    pub fn new() -> Self {
        Self {
            tiles: Vec::new(),
            col_n: None,
            row_n: 0,
        }
    }

    pub fn push_row(&mut self, row_str: &str) -> Result<(), Error> {
        let this_col_n = row_str.len(); // Assume only ASCII characters.
        if *self.col_n.get_or_insert(this_col_n) != this_col_n {
            return Err(Error::InconsistentRow(this_col_n, self.col_n.unwrap()));
        }

        for c in row_str.chars() {
            let tt = match c {
                '.' => TileType::Empty,
                '#' => TileType::Tree,
                other => return Err(Error::InvalidTileCharacter(other)),
            };
            self.tiles.push(tt);
        }
        self.row_n += 1;

        Ok(())
    }

    pub fn build(self) -> Map {
        Map {
            tiles: self.tiles,
            row_n: self.row_n,
            col_n: self.col_n.unwrap_or(0),
        }
    }
}

pub fn read_map<P: AsRef<Path>>(path: P) -> Result<Map, Error> {
    let file = File::open(path).map_err(Error::IOError)?;
    let reader = BufReader::new(file);
    let mut builder = MapBuilder::new();
    for line in reader.lines() {
        builder.push_row(&line.map_err(Error::IOError)?)?;
    }

    Ok(builder.build())
}
