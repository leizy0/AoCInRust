use std::{
    collections::{HashSet, LinkedList},
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
    NoCommaInPositonText,
    InvalidCoordinateText(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NoCommaInPositonText => write!(
                f,
                "Expect a comma to separate coordinates of position in text."
            ),
            Error::InvalidCoordinateText(s) => {
                write!(f, "Invalid text({}) for coordinate of position.", s)
            }
        }
    }
}

impl error::Error for Error {}

#[derive(Debug, Parser)]
pub struct CLIArgs {
    pub input_path: PathBuf,
    pub map_size: usize,
    pub corrupt_size: Option<usize>,
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
    x: usize,
    y: usize,
}

impl TryFrom<&str> for Position {
    type Error = Error;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        let comma_pos = value.find(',').ok_or(Error::NoCommaInPositonText)?;
        let x_text = &value[..comma_pos];
        let x = x_text
            .parse::<usize>()
            .map_err(|_| Error::InvalidCoordinateText(x_text.to_string()))?;
        let y_text = &value[(comma_pos + 1)..];
        let y = y_text
            .parse::<usize>()
            .map_err(|_| Error::InvalidCoordinateText(y_text.to_string()))?;

        Ok(Position::new(x, y))
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl Position {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    pub fn neighbor(&self, dir: Direction) -> Option<Self> {
        match dir {
            Direction::Up if self.y > 0 => Some(Position::new(self.x, self.y - 1)),
            Direction::Right => Some(Position::new(self.x + 1, self.y)),
            Direction::Down => Some(Position::new(self.x, self.y + 1)),
            Direction::Left if self.x > 0 => Some(Position::new(self.x - 1, self.y)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tile {
    Fine,
    Corrupted,
}

#[derive(Debug, Clone)]
pub struct Map {
    tiles: Vec<Tile>,
    row_n: usize,
    col_n: usize,
}

impl Map {
    pub fn new(row_n: usize, col_n: usize) -> Self {
        Self {
            tiles: vec![Tile::Fine; row_n * col_n],
            row_n,
            col_n,
        }
    }

    pub fn new_square(side_len: usize) -> Self {
        Self::new(side_len, side_len)
    }

    pub fn corrupt(&mut self, corr_positions: &[Position]) {
        for pos in corr_positions {
            if let Some(tile_mut) = self.tile_mut(pos) {
                *tile_mut = Tile::Corrupted;
            }
        }
    }

    pub fn reset(&mut self) {
        for tile in &mut self.tiles {
            *tile = Tile::Fine;
        }
    }

    pub fn min_steps_n(&self, from: &Position, to: &Position) -> Option<usize> {
        if !self.is_inside(from)
            || !self.is_inside(to)
            || self.tile(from).is_some_and(|tile| *tile == Tile::Corrupted)
            || self.tile(from).is_some_and(|tile| *tile == Tile::Corrupted)
        {
            return None;
        }

        let mut search_positions = LinkedList::from([(0, from.clone())]);
        let mut searched_positions = HashSet::from([from.clone()]);
        while let Some((cur_steps_n, cur_pos)) = search_positions.pop_front() {
            if cur_pos == *to {
                return Some(cur_steps_n);
            }

            for next_pos in Direction::all_dirs()
                .iter()
                .flat_map(|dir| cur_pos.neighbor(*dir))
                .filter(|pos| {
                    self.tile(pos)
                        .map(|tile| *tile == Tile::Fine)
                        .unwrap_or(false)
                })
            {
                if searched_positions.insert(next_pos.clone()) {
                    search_positions.push_back((cur_steps_n + 1, next_pos));
                }
            }
        }

        None
    }

    fn tile_mut(&mut self, pos: &Position) -> Option<&mut Tile> {
        self.pos_to_ind(pos).and_then(|ind| self.tiles.get_mut(ind))
    }

    fn tile(&self, pos: &Position) -> Option<&Tile> {
        self.pos_to_ind(pos).and_then(|ind| self.tiles.get(ind))
    }

    fn pos_to_ind(&self, pos: &Position) -> Option<usize> {
        if self.is_inside(pos) {
            Some(pos.y * self.col_n + pos.x)
        } else {
            None
        }
    }

    fn is_inside(&self, pos: &Position) -> bool {
        pos.y < self.row_n && pos.x < self.col_n
    }
}

pub fn read_positions<P: AsRef<Path>>(path: P) -> Result<Vec<Position>> {
    let file = File::open(&path)
        .with_context(|| format!("Failed to open given file({}).", path.as_ref().display()))?;
    let reader = BufReader::new(file);
    reader
        .lines()
        .enumerate()
        .map(|(ind, line)| {
            line.with_context(|| {
                format!(
                    "Failed to read line {} from given file({}).",
                    ind + 1,
                    path.as_ref().display()
                )
            })
            .and_then(|s| {
                Position::try_from(s.as_str()).with_context(|| format!("Failed to parse position."))
            })
        })
        .collect()
}
