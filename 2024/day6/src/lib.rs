use std::{
    collections::HashSet,
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
    MultipleGuards(Guard, Guard),
    InvalidChar(char),
    NoGuard,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InconsistentRow(expect_col_n, real_col_n) => write!(
                f,
                "Expect {} columns in this row, given {}.",
                expect_col_n, real_col_n
            ),
            Error::MultipleGuards(guard0, guard1) => write!(
                f,
                "Found multiple guards({}, {}) in given laboratory, expect one only.",
                guard0, guard1
            ),
            Error::InvalidChar(c) => {
                write!(f, "Invalid character({}) in text of laboratory layout.", c)
            }
            Error::NoGuard => write!(f, "There's no guard in given laboratory, but expect one."),
        }
    }
}

impl error::Error for Error {}

#[derive(Debug, Parser)]
pub struct CLIArgs {
    pub input_path: PathBuf,
}

#[derive(Debug, Clone, Copy)]
enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::Up => write!(f, "^"),
            Direction::Right => write!(f, ">"),
            Direction::Down => write!(f, "v"),
            Direction::Left => write!(f, "<"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Position {
    r: usize,
    c: usize,
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.r, self.c)
    }
}

impl Position {
    pub fn new(r: usize, c: usize) -> Self {
        Self { r, c }
    }

    pub fn along(&self, dir: Direction) -> Option<Position> {
        match dir {
            Direction::Up if self.r > 0 => Some(Self::new(self.r - 1, self.c)),
            Direction::Right => Some(Self::new(self.r, self.c + 1)),
            Direction::Down => Some(Self::new(self.r + 1, self.c)),
            Direction::Left if self.c > 0 => Some(Self::new(self.r, self.c - 1)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
struct Guard {
    pos: Position,
    dir: Direction,
}

impl Display for Guard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.dir, self.pos())
    }
}

impl Guard {
    pub fn new(pos: &Position, dir: Direction) -> Self {
        Self {
            pos: pos.clone(),
            dir,
        }
    }

    pub fn pos(&self) -> &Position {
        &self.pos
    }

    pub fn ahead_pos(&self) -> Option<Position> {
        self.pos.along(self.dir)
    }

    pub fn go_ahead(&mut self) -> bool {
        if let Some(ahead_pos) = self.ahead_pos() {
            self.pos = ahead_pos;
            true
        } else {
            false
        }
    }

    pub fn turn_right(&mut self) {
        self.dir = match self.dir {
            Direction::Up => Direction::Right,
            Direction::Right => Direction::Down,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up,
        }
    }
}

pub struct Laboratory {
    tiles: Vec<bool>, // Is occupied?
    row_n: usize,
    col_n: usize,
    guard: Guard,
}

impl Laboratory {
    pub fn sim_patrol(&self) -> usize {
        let mut cur_guard = self.guard.clone();
        let mut moved_tiles = HashSet::new();
        while self.is_inside(cur_guard.pos()) {
            moved_tiles.insert(cur_guard.pos().clone());
            if let Some(next_pos) = cur_guard.ahead_pos() {
                if self
                    .is_occupied(&next_pos)
                    .is_some_and(|is_occupied| *is_occupied)
                {
                    cur_guard.turn_right();
                    continue;
                }

                cur_guard.go_ahead();
            } else {
                break;
            }
        }

        moved_tiles.len()
    }

    fn is_inside(&self, pos: &Position) -> bool {
        pos.r < self.row_n && pos.c < self.col_n
    }

    fn is_occupied(&self, pos: &Position) -> Option<&bool> {
        if pos.r >= self.row_n || pos.c >= self.col_n {
            None
        } else {
            self.tiles.get(pos.r * self.col_n + pos.c)
        }
    }
}

struct LaboratoryBuilder {
    tiles: Vec<bool>,
    row_n: usize,
    col_n: Option<usize>,
    guard: Option<Guard>,
}

impl LaboratoryBuilder {
    pub fn new() -> Self {
        Self {
            tiles: Vec::new(),
            row_n: 0,
            col_n: None,
            guard: None,
        }
    }

    pub fn add_row(&mut self, row_text: &str) -> Result<(), Error> {
        let this_col_n = row_text.chars().count();
        if *self.col_n.get_or_insert(this_col_n) != this_col_n {
            return Err(Error::InconsistentRow(self.col_n.unwrap(), this_col_n));
        }

        for (ind, c) in row_text.chars().enumerate() {
            match c {
                '.' => self.tiles.push(false),
                '#' => self.tiles.push(true),
                '^' => {
                    let guard = Guard::new(&Position::new(self.row_n, ind), Direction::Up);
                    if self.guard.is_some() {
                        return Err(Error::MultipleGuards(self.guard.take().unwrap(), guard));
                    }

                    self.guard = Some(guard);
                    self.tiles.push(false);
                }
                other => return Err(Error::InvalidChar(other)),
            }
        }
        self.row_n += 1;

        Ok(())
    }

    pub fn build(self) -> Result<Laboratory, Error> {
        if self.guard.is_none() {
            return Err(Error::NoGuard);
        }

        Ok(Laboratory {
            tiles: self.tiles,
            row_n: self.row_n,
            col_n: self.col_n.unwrap_or(0),
            guard: self.guard.unwrap(),
        })
    }
}

pub fn read_lab<P: AsRef<Path>>(path: P) -> Result<Laboratory> {
    let file = File::open(&path)
        .with_context(|| format!("Failed to open given file({}).", path.as_ref().display()))?;
    let reader = BufReader::new(file);
    let mut builder = LaboratoryBuilder::new();
    for (ind, line) in reader.lines().enumerate() {
        let line = line.with_context(|| {
            format!(
                "Failed to read line {} from given file({}).",
                ind + 1,
                path.as_ref().display()
            )
        })?;
        builder.add_row(line.as_str())?;
    }

    Ok(builder.build()?)
}
