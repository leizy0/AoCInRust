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
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InconsistentRow(expect_col_n, this_col_n) => write!(
                f,
                "Expect {} columns in each row, given {}.",
                expect_col_n, this_col_n
            ),
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

    pub fn neighbor(&self, dir: Direction) -> Option<Position> {
        match dir {
            Direction::Up if self.r > 0 => Some(Position::new(self.r - 1, self.c)),
            Direction::Right => Some(Position::new(self.r, self.c + 1)),
            Direction::Down => Some(Position::new(self.r + 1, self.c)),
            Direction::Left if self.c > 0 => Some(Position::new(self.r, self.c - 1)),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct Region {
    tile_positions: Vec<Position>,
    perimeter: usize,
}

impl Region {
    pub fn area(&self) -> usize {
        self.tile_positions.len()
    }

    pub fn perimeter(&self) -> usize {
        self.perimeter
    }
}

#[derive(Debug)]
pub struct Map {
    tiles: Vec<char>,
    row_n: usize,
    col_n: usize,
}

impl Map {
    pub fn all_regions(&self) -> Vec<Region> {
        let mut regions = Vec::new();
        let mut region_marks = vec![false; self.tiles.len()];
        for r in 0..self.row_n {
            for c in 0..self.col_n {
                let pos = Position::new(r, c);
                let ind = self.pos_to_ind(&pos).unwrap();
                if !region_marks[ind] {
                    if let Some(region) = self.search_region(&pos, &mut region_marks) {
                        regions.push(region);
                    }
                }
            }
        }

        regions
    }

    fn search_region(&self, start_pos: &Position, region_marks: &mut [bool]) -> Option<Region> {
        if !self
            .pos_to_ind(start_pos)
            .is_some_and(|ind| !region_marks[ind])
        {
            return None;
        }

        let Some(region_char) = self.tile(start_pos).copied() else {
            return None;
        };

        let mut next_positions = LinkedList::from_iter(iter::once(start_pos.clone()));
        let mut searched_positions = HashSet::new();
        let mut perimeter = 0;
        let mut tile_positions = Vec::new();
        while let Some(cur_pos) = next_positions.pop_front() {
            if !searched_positions.insert(cur_pos.clone()) {
                continue;
            }

            tile_positions.push(cur_pos.clone());
            region_marks[self.pos_to_ind(&cur_pos).unwrap()] = true;
            for dir in Direction::all_dirs().iter().copied() {
                if let Some(neighbor) = cur_pos.neighbor(dir) {
                    if self.tile(&neighbor).is_some_and(|c| *c == region_char) {
                        if !searched_positions.contains(&neighbor) {
                            next_positions.push_back(neighbor);
                        }
                        continue;
                    }
                }

                perimeter += 1;
            }
        }

        Some(Region {
            tile_positions,
            perimeter,
        })
    }

    fn pos_to_ind(&self, pos: &Position) -> Option<usize> {
        if pos.r < self.row_n && pos.c < self.col_n {
            Some(pos.r * self.col_n + pos.c)
        } else {
            None
        }
    }

    fn tile(&self, pos: &Position) -> Option<&char> {
        self.pos_to_ind(pos).and_then(|ind| self.tiles.get(ind))
    }
}

#[derive(Debug)]
struct MapBuilder {
    tiles: Vec<char>,
    row_n: usize,
    col_n: Option<usize>,
}

impl MapBuilder {
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

        self.tiles.extend(text.chars());
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

pub fn read_map<P: AsRef<Path>>(path: P) -> Result<Map> {
    let file = File::open(&path)
        .with_context(|| format!("Failed to open given file({}).", path.as_ref().display()))?;
    let reader = BufReader::new(file);
    let mut builder = MapBuilder::new();
    for (ind, line) in reader.lines().enumerate() {
        let line = line.with_context(|| {
            format!(
                "Failed to read line {} in given file({}).",
                ind + 1,
                path.as_ref().display()
            )
        })?;
        builder.add_row(line.as_str())?;
    }

    Ok(builder.build())
}
