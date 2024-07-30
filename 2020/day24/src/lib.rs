use std::{
    collections::HashSet,
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
    InvalidDirText(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidDirText(s) => write!(f, "Invalid string for direction: {}", s),
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
    Northwest,
    Northeast,
    East,
    Southeast,
    Southwest,
    West,
}

impl Direction {
    fn all_dirs() -> &'static [Direction] {
        static ALL_DIRECTIONS: [Direction; 6] = [
            Direction::Northwest,
            Direction::Northeast,
            Direction::East,
            Direction::Southeast,
            Direction::Southwest,
            Direction::West,
        ];

        &ALL_DIRECTIONS
    }
}

// i + j + k = 0.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HexPos {
    i: isize, // + for northwest, - for southeast.
    j: isize, // + for east, - for west.
    k: isize, // + for southwest, - for northeast.
}

impl TryFrom<&str> for HexPos {
    type Error = Error;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        let mut pos = HexPos::new();
        let mut chars = value.chars();
        while let Some(c) = chars.next() {
            match c {
                // East.
                'e' => pos.j += 1,
                'n' => {
                    if let Some(next_c) = chars.next() {
                        match next_c {
                            // Northeast.
                            'e' => pos.k -= 1,
                            // Northwest.
                            'w' => pos.i += 1,
                            other => {
                                return Err(Error::InvalidDirText(String::from_iter(&[c, other])))
                            }
                        }
                    } else {
                        return Err(Error::InvalidDirText(c.to_string()));
                    }
                }
                's' => {
                    if let Some(next_c) = chars.next() {
                        match next_c {
                            // Southeast.
                            'e' => pos.i -= 1,
                            // Southwest.
                            'w' => pos.k += 1,
                            other => {
                                return Err(Error::InvalidDirText(String::from_iter(&[c, other])))
                            }
                        }
                    } else {
                        return Err(Error::InvalidDirText(c.to_string()));
                    }
                }
                // West.
                'w' => pos.j -= 1,
                other => return Err(Error::InvalidDirText(other.to_string())),
            }
        }
        pos.canonicalize();

        Ok(pos)
    }
}

impl HexPos {
    pub fn new() -> Self {
        Self { i: 0, j: 0, k: 0 }
    }

    pub fn canonicalize(&mut self) {
        let min = self.i.min(self.j.min(self.k));
        self.i -= min;
        self.j -= min;
        self.k -= min;
    }

    fn neighbors(&self) -> impl Iterator<Item = Self> + '_ {
        Direction::all_dirs().iter().map(|dir| self.along(*dir))
    }

    fn along(&self, dir: Direction) -> Self {
        let mut cpos = self.clone();
        match dir {
            Direction::Northwest => cpos.i += 1,
            Direction::Northeast => cpos.k -= 1,
            Direction::East => cpos.j += 1,
            Direction::Southeast => cpos.i -= 1,
            Direction::Southwest => cpos.k += 1,
            Direction::West => cpos.j -= 1,
        }
        cpos.canonicalize();

        cpos
    }
}

pub struct HexPlane {
    sets: [HashSet<HexPos>; 2],
    cur_set_ind: usize,
}

impl HexPlane {
    pub fn new() -> Self {
        Self {
            sets: [HashSet::new(), HashSet::new()],
            cur_set_ind: 0,
        }
    }

    pub fn flip_pos(&mut self, pos: &HexPos) {
        let cur_set = self.cur_set_mut();
        let _ = cur_set.insert(pos.clone()) || cur_set.remove(pos);
    }

    pub fn flip(&mut self) {
        let (r_set, w_set) = self.rw_sets();
        let is_black = |pos: &HexPos| r_set.contains(pos);
        w_set.clear();
        let mut checked_pos = HashSet::new();
        for pos in r_set
            .iter()
            .flat_map(|pos| pos.neighbors().chain(iter::once(pos.clone())))
            .filter(|pos| checked_pos.insert(pos.clone()))
        {
            let black_neighbors_n = pos.neighbors().filter(|pos| is_black(pos)).count();
            if is_black(&pos) && (black_neighbors_n == 1 || black_neighbors_n == 2) {
                // Retain black.
                w_set.insert(pos);
            } else if !is_black(&pos) && black_neighbors_n == 2 {
                // Flip to black.
                w_set.insert(pos);
            }
        }

        self.swap_sets();
    }

    pub fn black_n(&self) -> usize {
        self.cur_set().len()
    }

    fn cur_set(&self) -> &HashSet<HexPos> {
        &self.sets[self.cur_set_ind]
    }

    fn cur_set_mut(&mut self) -> &mut HashSet<HexPos> {
        &mut self.sets[self.cur_set_ind]
    }

    fn rw_sets(&mut self) -> (&HashSet<HexPos>, &mut HashSet<HexPos>) {
        let (left_sets, right_sets) = self.sets.split_at_mut(1);
        if self.cur_set_ind == 0 {
            (&left_sets[0], &mut right_sets[0])
        } else {
            (&right_sets[0], &mut left_sets[0])
        }
    }

    fn swap_sets(&mut self) {
        self.cur_set_ind = 1 - self.cur_set_ind;
    }
}

pub fn read_hex_poss<P: AsRef<Path>>(path: P) -> Result<Vec<HexPos>> {
    let file = File::open(path.as_ref()).with_context(|| {
        format!(
            "Failed to open given input file: {}.",
            path.as_ref().display()
        )
    })?;
    let reader = BufReader::new(file);
    reader
        .lines()
        .enumerate()
        .map(|(ind, l)| {
            l.with_context(|| format!("Failed to read line {}.", ind + 1))
                .and_then(|s| HexPos::try_from(s.as_str()).map_err(|e| e.into()))
        })
        .collect()
}
