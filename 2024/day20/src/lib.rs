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
    InconsistentRow(usize, usize),
    MultipleStartPosition(Position, Position),
    MultipleEndPosition(Position, Position),
    InvalidCharForMap(char),
    NoStartPosition,
    NoEndPosition,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InconsistentRow(expect_col_n, this_col_n) => write!(
                f,
                "Expect {} column(s) in each row, given {}.",
                expect_col_n, this_col_n
            ),
            Error::MultipleStartPosition(last_pos, pos) => write!(
                f,
                "Expect only one start position, given two({}, {}).",
                last_pos, pos
            ),
            Error::MultipleEndPosition(last_pos, pos) => write!(
                f,
                "Expect only one end position, given two({}, {}).",
                last_pos, pos
            ),
            Error::InvalidCharForMap(c) => write!(f, "Invalid character({}) for map.", c),
            Error::NoStartPosition => write!(f, "No start position in map."),
            Error::NoEndPosition => write!(f, "No end position in map."),
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
            Direction::North if self.r > 0 => Some(Self::new(self.r - 1, self.c)),
            Direction::East => Some(Self::new(self.r, self.c + 1)),
            Direction::South => Some(Self::new(self.r + 1, self.c)),
            Direction::West if self.c > 0 => Some(Self::new(self.r, self.c - 1)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    Wall,
    Track,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    pub fn all_dirs() -> &'static [Direction] {
        static ALL_DIRCTIONS: [Direction; 4] = [
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ];

        &ALL_DIRCTIONS
    }

    pub fn turn_clockwise(&self) -> Self {
        match self {
            Direction::North => Direction::East,
            Direction::East => Direction::South,
            Direction::South => Direction::West,
            Direction::West => Direction::North,
        }
    }

    pub fn turn_counterclockwise(&self) -> Self {
        match self {
            Direction::North => Direction::West,
            Direction::East => Direction::North,
            Direction::South => Direction::East,
            Direction::West => Direction::South,
        }
    }

    pub fn reverse(&self) -> Self {
        match self {
            Direction::North => Direction::South,
            Direction::East => Direction::West,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Cheat {
    start_pos: Position,
    end_pos: Position,
}

impl Cheat {
    pub fn new(start_pos: &Position, end_pos: &Position) -> Self {
        Self {
            start_pos: start_pos.clone(),
            end_pos: end_pos.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Program {
    pos: Position,
    cheat: Option<Cheat>,
}

impl Program {
    pub fn new(pos: &Position) -> Self {
        Self {
            pos: pos.clone(),
            cheat: None,
        }
    }

    pub fn done_cheat(&self) -> Option<&Cheat> {
        self.cheat.as_ref()
    }

    pub fn clone_and_go(&self, map: &Map, dir: Direction) -> Option<Self> {
        self.pos
            .neighbor(dir)
            .filter(|pos| map.tile(pos).map_or(false, |tile| *tile == Tile::Track))
            .map(|next_position| Self {
                pos: next_position,
                cheat: self.cheat.clone(),
            })
    }

    pub fn clone_and_cheat(&self, map: &Map, dir: Direction) -> Option<Self> {
        if self.cheat.is_some() {
            return None;
        }

        if let Some(start_pos) = self
            .pos
            .neighbor(dir)
            .filter(|pos| map.tile(pos).map_or(false, |tile| *tile == Tile::Wall))
        {
            if let Some(end_pos) = start_pos
                .neighbor(dir)
                .filter(|pos| map.tile(pos).map_or(false, |tile| *tile == Tile::Track))
            {
                return Some(Self {
                    pos: end_pos.clone(),
                    cheat: Some(Cheat::new(&start_pos, &end_pos)),
                });
            }
        }

        None
    }

    pub fn pos(&self) -> &Position {
        &self.pos
    }
}

#[derive(Debug)]
pub struct Map {
    tiles: Vec<Tile>,
    row_n: usize,
    col_n: usize,
    start_pos: Position,
    end_pos: Position,
}

impl Map {
    pub fn fastest_steps_n(&self) -> Option<usize> {
        let mut search_states = LinkedList::from([(self.start_pos.clone(), 0)]);
        let mut searched_positions = HashSet::from([self.start_pos.clone()]);
        while let Some((cur_pos, cur_steps_n)) = search_states.pop_front() {
            if cur_pos == self.end_pos {
                return Some(cur_steps_n);
            }

            for next_pos in Direction::all_dirs().iter().filter_map(|dir| {
                cur_pos.neighbor(*dir).filter(|pos| {
                    self.tile(&pos)
                        .map(|tile| *tile == Tile::Track)
                        .unwrap_or(false)
                })
            }) {
                if searched_positions.insert(next_pos.clone()) {
                    search_states.push_back((next_pos, cur_steps_n + 1));
                }
            }
        }

        None
    }

    pub fn cheats_steps_n(&self, threshold_steps_n: usize) -> usize {
        let mut cheats_n = 0;
        let mut search_states = LinkedList::from([(Program::new(&self.start_pos), 0)]);
        let mut searched_programs = HashSet::from([Program::new(&self.start_pos)]);
        while let Some((cur_program, cur_steps_n)) = search_states.pop_front() {
            if cur_steps_n > threshold_steps_n {
                continue;
            }

            if *cur_program.pos() == self.end_pos {
                if let Some(_cheat) = cur_program.done_cheat() {
                    cheats_n += 1;
                }
            }

            for next_program in Direction::all_dirs()
                .iter()
                .filter_map(|dir| cur_program.clone_and_go(self, *dir))
            {
                if searched_programs.insert(next_program.clone()) {
                    search_states.push_back((next_program, cur_steps_n + 1));
                }
            }

            if cur_program.done_cheat().is_none() {
                for next_cheated_program in Direction::all_dirs()
                    .iter()
                    .filter_map(|dir| cur_program.clone_and_cheat(self, *dir))
                {
                    if searched_programs.insert(next_cheated_program.clone()) {
                        search_states.push_back((next_cheated_program, cur_steps_n + 2));
                    }
                }
            }
        }

        cheats_n
    }

    pub fn tile(&self, pos: &Position) -> Option<&Tile> {
        if pos.r < self.row_n && pos.c < self.col_n {
            self.tiles.get(pos.r * self.col_n + pos.c)
        } else {
            None
        }
    }
}

#[derive(Debug)]
struct MapBuilder {
    tiles: Vec<Tile>,
    row_n: usize,
    col_n: Option<usize>,
    start_pos: Option<Position>,
    end_pos: Option<Position>,
}

impl MapBuilder {
    pub fn new() -> Self {
        Self {
            tiles: Vec::new(),
            row_n: 0,
            col_n: None,
            start_pos: None,
            end_pos: None,
        }
    }

    pub fn add_row(&mut self, text: &str) -> Result<(), Error> {
        let this_col_n = text.chars().count();
        if *self.col_n.get_or_insert(this_col_n) != this_col_n {
            return Err(Error::InconsistentRow(self.col_n.unwrap(), this_col_n));
        }

        for (ind, c) in text.chars().enumerate() {
            let pos = Position::new(self.row_n, ind);
            self.tiles.push(match c {
                'S' => {
                    if let Some(last_pos) = self.start_pos.as_ref().take() {
                        return Err(Error::MultipleStartPosition(last_pos.clone(), pos));
                    }

                    self.start_pos = Some(pos);
                    Tile::Track
                }
                'E' => {
                    if let Some(last_pos) = self.end_pos.as_ref().take() {
                        return Err(Error::MultipleEndPosition(last_pos.clone(), pos));
                    }

                    self.end_pos = Some(pos);
                    Tile::Track
                }
                '#' => Tile::Wall,
                '.' => Tile::Track,
                other => return Err(Error::InvalidCharForMap(other)),
            });
        }
        self.row_n += 1;

        Ok(())
    }

    pub fn build(self) -> Result<Map, Error> {
        let Some(start_pos) = self.start_pos else {
            return Err(Error::NoStartPosition);
        };
        let Some(end_pos) = self.end_pos else {
            return Err(Error::NoEndPosition);
        };

        Ok(Map {
            tiles: self.tiles,
            row_n: self.row_n,
            col_n: self.col_n.unwrap_or(0),
            start_pos,
            end_pos,
        })
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
        builder.add_row(line.as_str())?
    }

    Ok(builder.build()?)
}
