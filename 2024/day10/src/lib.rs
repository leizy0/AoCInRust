use std::{
    collections::{HashSet, LinkedList},
    error,
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader},
    iter,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use clap::Parser;

#[derive(Debug)]
pub enum Error {
    InconsistentRow(usize, usize),
    InvalidChar(char),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InconsistentRow(expect_col_n, this_col_n) => write!(
                f,
                "Expect {} character(s) in one row, given {}.",
                expect_col_n, this_col_n
            ),
            Error::InvalidChar(c) => write!(f, "Invalid character({}) in map text.", c),
        }
    }
}

impl error::Error for Error {}

#[derive(Debug, Parser)]
pub struct CLIArgs {
    pub input_path: PathBuf,
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl Direction {
    pub fn all_dirs() -> &'static [Direction] {
        static ALL_DIRECTIONS: [Direction; 4] = [
            Direction::Up,
            Direction::Right,
            Direction::Down,
            Direction::Left,
        ];

        &ALL_DIRECTIONS
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Position {
    r: usize,
    c: usize,
}

impl Position {
    pub fn new(r: usize, c: usize) -> Self {
        Self { r, c }
    }

    pub fn neightbor(&self, dir: Direction) -> Option<Position> {
        match dir {
            Direction::Up if self.r > 0 => Some(Position::new(self.r - 1, self.c)),
            Direction::Right => Some(Position::new(self.r, self.c + 1)),
            Direction::Down => Some(Position::new(self.r + 1, self.c)),
            Direction::Left if self.c > 0 => Some(Position::new(self.r, self.c - 1)),
            _ => None,
        }
    }
}

pub struct Map {
    heights: Vec<usize>,
    row_n: usize,
    col_n: usize,
}

impl Map {
    pub fn trailheads(&self) -> Vec<Position> {
        (0..self.row_n)
            .flat_map(|r| {
                (0..self.col_n).filter_map(move |c| {
                    let pos = Position::new(r, c);
                    if self.height(&pos).is_some_and(|height| *height == 0) {
                        Some(pos)
                    } else {
                        None
                    }
                })
            })
            .collect()
    }

    pub fn score_from(&self, pos: &Position) -> usize {
        if !self.height(pos).is_some_and(|h| *h == 0) {
            return 0;
        }

        let mut end_positions = HashSet::new();
        let mut searched_positions = HashSet::new();
        let mut search_positions = LinkedList::from_iter(iter::once(pos.clone()));
        while let Some(cur_pos) = search_positions.pop_front() {
            if searched_positions.insert(cur_pos.clone()) {
                if let Some(cur_height) = self.height(&cur_pos) {
                    if *cur_height == 9 {
                        end_positions.insert(cur_pos.clone());
                        continue;
                    }

                    let search_height = cur_height + 1;
                    search_positions.extend(Direction::all_dirs().iter().filter_map(|dir| {
                        cur_pos.neightbor(*dir).filter(|pos| {
                            self.height(&pos)
                                .map(|height| *height == search_height)
                                .unwrap_or(false)
                        })
                    }));
                }
            }
        }

        end_positions.len()
    }

    fn height(&self, pos: &Position) -> Option<&usize> {
        if pos.r < self.row_n && pos.c < self.col_n {
            self.heights.get(pos.r * self.row_n + pos.c)
        } else {
            None
        }
    }
}

struct MapBuilder {
    heights: Vec<usize>,
    row_n: usize,
    col_n: Option<usize>,
}

impl MapBuilder {
    pub fn new() -> Self {
        Self {
            heights: Vec::new(),
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
            let height = c.to_digit(10).ok_or(Error::InvalidChar(c))?;
            self.heights.push(usize::try_from(height).unwrap());
        }
        self.row_n += 1;

        Ok(())
    }

    pub fn build(self) -> Map {
        Map {
            heights: self.heights,
            row_n: self.row_n,
            col_n: self.col_n.unwrap_or(0),
        }
    }
}

pub fn read_map<P: AsRef<Path>>(path: P) -> Result<Map> {
    let file = File::open(&path)
        .with_context(|| format!("Failed to open given file({}).", path.as_ref().display()))?;
    let reader = BufReader::new(file);
    let mut builder = MapBuilder::new();
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
