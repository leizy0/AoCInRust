use std::{
    collections::{HashMap, HashSet, LinkedList},
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
    cheat_start_time: Option<usize>,
    cheat_end_time: Option<usize>,
    cheat_start_pos: Option<Position>,
}

impl Program {
    pub fn new(pos: &Position) -> Self {
        Self {
            pos: pos.clone(),
            cheat: None,
            cheat_start_time: None,
            cheat_end_time: None,
            cheat_start_pos: None,
        }
    }

    pub fn cheat_start_time(&self) -> Option<usize> {
        self.cheat_start_time
    }

    pub fn done_cheat(&self) -> Option<(&Cheat, usize, usize)> {
        self.cheat.as_ref().map(|cheat| {
            (
                cheat,
                self.cheat_start_time.unwrap(),
                self.cheat_end_time.unwrap(),
            )
        })
    }

    pub fn clone_and_start_cheating(&self, cur_time: usize) -> Option<Self> {
        if self.cheat_start_time.is_none() {
            debug_assert!(self.cheat.is_none() && self.cheat_start_pos.is_none());
            Some(Self {
                cheat_start_time: Some(cur_time),
                cheat_start_pos: Some(self.pos.clone()),
                ..self.clone()
            })
        } else {
            None
        }
    }

    pub fn clone_and_end_cheating(&self, cur_time: usize, map: &Map) -> Option<Self> {
        if self.cheat_start_time.is_some() {
            if let Some(cheat_start_pos) = &self.cheat_start_pos {
                if &self.pos != cheat_start_pos
                    && self.cheat.is_none()
                    && map
                        .tile(&self.pos)
                        .map_or(false, |tile| *tile == Tile::Track)
                {
                    debug_assert!(self.cheat_end_time.is_none());
                    return Some(Self {
                        cheat: Some(Cheat::new(cheat_start_pos, &self.pos)),
                        cheat_end_time: Some(cur_time),
                        ..self.clone()
                    });
                }
            }
        }

        None
    }

    pub fn clone_and_go(&self, map: &Map, dir: Direction) -> Option<Self> {
        self.pos
            .neighbor(dir)
            .and_then(|pos| map.tile(&pos).map(|tile| (pos, tile)))
            .filter(|(_, tile)| self.is_cheating() || **tile == Tile::Track)
            .map(|(next_position, _)| Self {
                pos: next_position.clone(),
                ..self.clone()
            })
    }

    pub fn is_cheating(&self) -> bool {
        self.cheat_start_time.is_some() && self.cheat_end_time.is_none()
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
        self.min_time_from_end().get(&self.start_pos).copied()
    }

    pub fn saving_cheats_n(&self, cheat_duration: usize) -> HashMap<usize, usize> {
        let mut cheats_steps_n = HashMap::new();
        let mut search_states = LinkedList::from([(Program::new(&self.start_pos), 0)]);
        let mut searched_programs = HashSet::from([Program::new(&self.start_pos)]);
        let min_time_from_end = self.min_time_from_end();
        let Some(no_cheat_pico_sec_n) = min_time_from_end.get(&self.start_pos).copied() else {
            return HashMap::new();
        };
        while let Some((cur_program, cur_pico_sec_n)) = search_states.pop_front() {
            if cur_pico_sec_n > no_cheat_pico_sec_n {
                continue;
            }

            if *cur_program.pos() == self.end_pos {
                if let Some((cheat, _, _)) = cur_program.done_cheat() {
                    // Complete the map.
                    cheats_steps_n
                        .entry(cheat.clone())
                        .or_insert(cur_pico_sec_n);
                }
            }

            if let Some(cheat_start_time) = cur_program.cheat_start_time() {
                if cur_program.is_cheating() && cheat_start_time + cheat_duration < cur_pico_sec_n {
                    // Cheat timeout.
                    continue;
                }
            }

            if let Some(cheating_program) = cur_program.clone_and_start_cheating(cur_pico_sec_n) {
                if searched_programs.insert(cheating_program.clone()) {
                    // Start cheat.
                    search_states.push_front((cheating_program, cur_pico_sec_n));
                }
            }

            if let Some(cheated_program) = cur_program.clone_and_end_cheating(cur_pico_sec_n, self)
            {
                // End cheat.
                let (cur_cheat, _, _) = cheated_program.done_cheat().unwrap();
                let left_pico_sec_n = min_time_from_end.get(&cur_cheat.end_pos).unwrap();
                let total_pico_sec_n = cur_pico_sec_n + left_pico_sec_n;
                cheats_steps_n
                    .entry(cur_cheat.clone())
                    .or_insert(total_pico_sec_n);
            }

            for next_program in Direction::all_dirs()
                .iter()
                .filter_map(|dir| cur_program.clone_and_go(self, *dir))
            {
                // Move.
                if searched_programs.insert(next_program.clone()) {
                    search_states.push_back((next_program, cur_pico_sec_n + 1));
                }
            }
        }

        let mut savings_map = HashMap::new();
        for saving_pico_sec_n in cheats_steps_n
            .iter()
            .filter_map(|(_, pico_sec_n)| no_cheat_pico_sec_n.checked_sub(*pico_sec_n))
        {
            *savings_map.entry(saving_pico_sec_n).or_insert(0) += 1;
        }

        savings_map
    }

    pub fn min_time_from_end(&self) -> HashMap<Position, usize> {
        let mut search_states = LinkedList::from([(self.end_pos.clone(), 0)]);
        let mut min_time_from_end = HashMap::from([(self.end_pos.clone(), 0)]);
        while let Some((cur_pos, cur_steps_n)) = search_states.pop_front() {
            for next_pos in Direction::all_dirs().iter().filter_map(|dir| {
                cur_pos.neighbor(*dir).filter(|pos| {
                    self.tile(&pos)
                        .map(|tile| *tile == Tile::Track)
                        .unwrap_or(false)
                })
            }) {
                let next_steps_n = cur_steps_n + 1;
                if !min_time_from_end.contains_key(&next_pos) {
                    min_time_from_end
                        .entry(next_pos.clone())
                        .or_insert(next_steps_n);
                    search_states.push_back((next_pos, next_steps_n));
                }
            }
        }

        min_time_from_end
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
