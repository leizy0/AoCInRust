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
    InconsistentRow(usize, usize),
    MultipleRobots(Position, Position),
    InvalidCharforMap(char),
    NoRobotInMap,
    InvalidCharforDirection(char),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InconsistentRow(expect_col_n, this_col_n) => write!(
                f,
                "Expect {} columns in each row, given {}.",
                expect_col_n, this_col_n
            ),
            Error::MultipleRobots(last_position, this_position) => write!(
                f,
                "Given two robots in map({}, {}), expect only one.",
                last_position, this_position
            ),
            Error::InvalidCharforMap(c) => write!(f, "Invalid character({}) for map.", c),
            Error::NoRobotInMap => write!(f, "No robot found in given map, expect one."),
            Error::InvalidCharforDirection(c) => {
                write!(f, "Invalid character({}) for direction.", c)
            }
        }
    }
}

impl error::Error for Error {}

#[derive(Debug, Parser)]
pub struct CLIArgs {
    pub input_path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    Wall,
    Robot,
    Box,
    Floor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Position {
    pub r: usize,
    pub c: usize,
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

    pub fn neighbor(&self, dir: Direction) -> Option<Self> {
        match dir {
            Direction::Up if self.r > 0 => Some(Position::new(self.r - 1, self.c)),
            Direction::Right => Some(Position::new(self.r, self.c + 1)),
            Direction::Down => Some(Position::new(self.r + 1, self.c)),
            Direction::Left if self.c > 0 => Some(Position::new(self.r, self.c - 1)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl TryFrom<char> for Direction {
    type Error = Error;

    fn try_from(value: char) -> std::result::Result<Self, Self::Error> {
        match value {
            '^' => Ok(Direction::Up),
            '>' => Ok(Direction::Right),
            'v' => Ok(Direction::Down),
            '<' => Ok(Direction::Left),
            other => Err(Error::InvalidCharforDirection(other)),
        }
    }
}

#[derive(Debug)]
pub struct Map {
    tiles: Vec<Tile>,
    robot_pos: Position,
    row_n: usize,
    col_n: usize,
}

impl Display for Map {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for r in 0..self.row_n {
            for c in 0..self.col_n {
                let pos = Position::new(r, c);
                let tile_char = match self.tile(&pos).unwrap() {
                    Tile::Wall => '#',
                    Tile::Robot => '@',
                    Tile::Box => 'O',
                    Tile::Floor => '.',
                };
                write!(f, "{}", tile_char)?;
            }
            writeln!(f, "")?;
        }

        Ok(())
    }
}

impl Map {
    pub fn simulate(&mut self, dirs: &[Direction]) {
        for dir in dirs {
            self.try_move(&self.robot_pos.clone(), *dir);
        }
    }

    pub fn position_iter(&self, tile: Tile) -> impl Iterator<Item = Position> + use<'_> {
        self.tiles
            .iter()
            .enumerate()
            .filter(move |(_, this_tile)| **this_tile == tile)
            .map(|(ind, _)| self.ind_to_pos(ind).unwrap())
    }

    fn try_move(&mut self, pos: &Position, dir: Direction) -> bool {
        if let Some(cur_tile) = self.tile(pos).cloned() {
            match cur_tile {
                Tile::Robot | Tile::Box => {
                    if let Some(next_position) = pos.neighbor(dir) {
                        if let Some(next_tile) = self.tile(&next_position).cloned() {
                            match next_tile {
                                Tile::Box | Tile::Floor => {
                                    if self.try_move(&next_position, dir) {
                                        *self.tile_mut(&next_position).unwrap() = cur_tile;
                                        *self.tile_mut(&pos).unwrap() = Tile::Floor;
                                        if cur_tile == Tile::Robot {
                                            self.robot_pos = next_position;
                                        }
                                        return true;
                                    }
                                }
                                _ => (),
                            }
                        }
                    }
                }
                Tile::Floor => return true,
                _ => (),
            }
        }

        false
    }

    fn tile(&self, pos: &Position) -> Option<&Tile> {
        self.pos_to_ind(pos).and_then(|ind| self.tiles.get(ind))
    }

    fn tile_mut(&mut self, pos: &Position) -> Option<&mut Tile> {
        self.pos_to_ind(pos).and_then(|ind| self.tiles.get_mut(ind))
    }

    fn pos_to_ind(&self, pos: &Position) -> Option<usize> {
        if pos.r < self.row_n && pos.c < self.col_n {
            Some(pos.r * self.col_n + pos.c)
        } else {
            None
        }
    }

    fn ind_to_pos(&self, ind: usize) -> Option<Position> {
        if ind < self.tiles.len() {
            Some(Position::new(ind / self.col_n, ind % self.col_n))
        } else {
            None
        }
    }
}

pub struct MapBuilder {
    tiles: Vec<Tile>,
    robot_pos: Option<Position>,
    row_n: usize,
    col_n: Option<usize>,
}

impl MapBuilder {
    pub fn new() -> Self {
        Self {
            tiles: Vec::new(),
            robot_pos: None,
            row_n: 0,
            col_n: None,
        }
    }

    pub fn add_row(&mut self, text: &str) -> Result<(), Error> {
        let this_col_n = text.chars().count();
        if *self.col_n.get_or_insert(this_col_n) != this_col_n {
            return Err(Error::InconsistentRow(self.col_n.unwrap(), this_col_n));
        }

        for (ind, c) in text.chars().enumerate() {
            self.tiles.push(match c {
                '#' => Tile::Wall,
                '.' => Tile::Floor,
                '@' => {
                    let this_robot_pos = Position::new(self.row_n, ind);
                    if *self.robot_pos.get_or_insert(this_robot_pos.clone()) != this_robot_pos {
                        return Err(Error::MultipleRobots(
                            self.robot_pos.as_ref().unwrap().clone(),
                            this_robot_pos,
                        ));
                    }

                    Tile::Robot
                }
                'O' => Tile::Box,
                other => return Err(Error::InvalidCharforMap(other)),
            });
        }
        self.row_n += 1;

        Ok(())
    }

    pub fn build(self) -> Result<Map, Error> {
        if let Some(robot_pos) = self.robot_pos {
            Ok(Map {
                tiles: self.tiles,
                robot_pos,
                row_n: self.row_n,
                col_n: self.col_n.unwrap_or(0),
            })
        } else {
            Err(Error::NoRobotInMap)
        }
    }
}

pub fn read_game<P: AsRef<Path>>(path: P) -> Result<(Map, Vec<Direction>)> {
    let file = File::open(&path)
        .with_context(|| format!("Failed to open given file({}).", path.as_ref().display()))?;
    let reader = BufReader::new(file);
    let mut builder = MapBuilder::new();
    let mut enum_lines = reader.lines().enumerate();
    while let Some((ind, line)) = enum_lines.next() {
        let line = line.with_context(|| {
            format!(
                "Failed to read line {} in given file({}).",
                ind + 1,
                path.as_ref().display()
            )
        })?;
        if line.is_empty() {
            break;
        }

        builder.add_row(line.as_str())?;
    }

    let mut move_dirs = Vec::new();
    while let Some((ind, line)) = enum_lines.next() {
        let line = line.with_context(|| {
            format!(
                "Failed to read line {} in given file({}).",
                ind + 1,
                path.as_ref().display()
            )
        })?;
        for c in line.chars() {
            move_dirs.push(Direction::try_from(c)?);
        }
    }

    Ok((builder.build()?, move_dirs))
}
