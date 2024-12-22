use std::{
    collections::{HashSet, LinkedList},
    error,
    fmt::{Debug, Display},
    fs::File,
    io::{BufRead, BufReader},
    iter,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use clap::Parser;

#[derive(Debug)]
pub enum Error {
    InvalidKey(char),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidKey(key) => write!(f, "Invalid key({}).", key),
        }
    }
}

impl error::Error for Error {}

#[derive(Debug, Parser)]
pub struct CLIArgs {
    pub input_path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    pub fn reverse(&self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Right => Direction::Left,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
        }
    }

    pub fn key(&self) -> char {
        match self {
            Direction::Up => '^',
            Direction::Right => '>',
            Direction::Down => 'v',
            Direction::Left => '<',
        }
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

    pub fn neighbor(&self, dir: Direction) -> Option<Self> {
        match dir {
            Direction::Up if self.r > 0 => Some(Self::new(self.r - 1, self.c)),
            Direction::Right => Some(Self::new(self.r, self.c + 1)),
            Direction::Down => Some(Self::new(self.r + 1, self.c)),
            Direction::Left if self.c > 0 => Some(Self::new(self.r, self.c - 1)),
            _ => None,
        }
    }
}

pub trait UI: Debug {
    fn start_pos(&self) -> Position;
    fn seek_key_steps_n(&self, start_pos: &Position, key: char) -> Option<(usize, Position)>;
    fn seek_key(&self, start_pos: &Position, key: char) -> Option<(Vec<Vec<Direction>>, Position)>;
    fn input(&self, code: &str) -> Result<Vec<Vec<char>>, Error>;
}

#[derive(Debug)]
pub struct Keypad {
    keys: Vec<Option<char>>,
    keys_set: HashSet<char>,
    row_n: usize,
    col_n: usize,
    start_pos: Position,
}

impl UI for Keypad {
    fn start_pos(&self) -> Position {
        self.start_pos.clone()
    }

    fn seek_key_steps_n(&self, start_pos: &Position, key: char) -> Option<(usize, Position)> {
        if !self.is_inside(start_pos) {
            return None;
        }

        let mut search_states = LinkedList::from([(start_pos.clone(), 0)]);
        let mut searched_positions = HashSet::from([(start_pos.clone())]);
        while let Some((cur_pos, cur_steps_n)) = search_states.pop_front() {
            if *self.key(&cur_pos).unwrap() == key {
                return Some((cur_steps_n, cur_pos));
            }

            for next_pos in Direction::all_dirs().iter().flat_map(|dir| {
                cur_pos
                    .neighbor(*dir)
                    .filter(|next_pos| self.is_inside(next_pos))
            }) {
                if searched_positions.insert(next_pos.clone()) {
                    search_states.push_back((next_pos, cur_steps_n + 1));
                }
            }
        }

        None
    }

    fn seek_key(&self, start_pos: &Position, key: char) -> Option<(Vec<Vec<Direction>>, Position)> {
        if !self.is_inside(start_pos) {
            return None;
        }

        let mut search_states = LinkedList::from([(start_pos.clone(), Vec::new())]);
        let mut searched_positions = HashSet::new();
        let mut min_paths = Vec::new();
        let mut min_path_steps_n = None;
        let mut key_pos = None;
        while let Some((cur_pos, cur_path)) = search_states.pop_front() {
            if min_path_steps_n.is_some_and(|min_steps_n| cur_path.len() > min_steps_n) {
                break;
            }

            searched_positions.insert(cur_pos.clone());
            if *self.key(&cur_pos).unwrap() == key {
                min_path_steps_n.get_or_insert(cur_path.len());
                key_pos.get_or_insert(cur_pos);
                min_paths.push(cur_path);
                continue;
            }

            for (next_pos, dir) in Direction::all_dirs().iter().flat_map(|dir| {
                cur_pos
                    .neighbor(*dir)
                    .filter(|next_pos| self.is_inside(next_pos))
                    .map(|next_pos| (next_pos, *dir))
            }) {
                if !searched_positions.contains(&next_pos) {
                    let mut next_path = cur_path.clone();
                    next_path.push(dir);
                    search_states.push_back((next_pos, next_path));
                }
            }
        }

        key_pos.map(|pos| (min_paths, pos))
    }

    fn input(&self, code: &str) -> Result<Vec<Vec<char>>, Error> {
        code.chars()
            .map(|c| {
                if self.keys_set.contains(&c) {
                    Ok(c)
                } else {
                    Err(Error::InvalidKey(c))
                }
            })
            .collect::<Result<Vec<_>, _>>()
            .map(|v| Vec::from([v]))
    }
}

impl Keypad {
    pub fn new_numeric() -> Self {
        let keys = Vec::from([
            Some('7'),
            Some('8'),
            Some('9'),
            Some('4'),
            Some('5'),
            Some('6'),
            Some('1'),
            Some('2'),
            Some('3'),
            None,
            Some('0'),
            Some('A'),
        ]);
        let keys_set = Self::keys_set_from_keys(&keys);

        Self {
            keys,
            keys_set,
            row_n: 4,
            col_n: 3,
            start_pos: Position::new(3, 2),
        }
    }

    pub fn new_directional() -> Self {
        use Direction::{Down, Left, Right, Up};
        let keys = Vec::from([
            None,
            Some(Up.key()),
            Some('A'),
            Some(Left.key()),
            Some(Down.key()),
            Some(Right.key()),
        ]);
        let keys_set = Self::keys_set_from_keys(&keys);

        Self {
            keys,
            keys_set,
            row_n: 2,
            col_n: 3,
            start_pos: Position::new(0, 2),
        }
    }

    fn is_inside(&self, pos: &Position) -> bool {
        self.key(pos).is_some()
    }

    fn key(&self, pos: &Position) -> Option<&char> {
        self.pos_to_ind(pos)
            .and_then(|ind| self.keys.get(ind).and_then(|key_op| key_op.as_ref()))
    }

    fn pos_to_ind(&self, pos: &Position) -> Option<usize> {
        if pos.r < self.row_n && pos.c < self.col_n {
            Some(pos.r * self.col_n + pos.c)
        } else {
            None
        }
    }

    fn keys_set_from_keys(keys: &Vec<Option<char>>) -> HashSet<char> {
        keys.iter().flat_map(|key_op| *key_op).collect()
    }
}

#[derive(Debug)]
pub struct Robot {
    ui: Keypad,
    target_ui: Box<dyn UI>,
}

impl UI for Robot {
    fn start_pos(&self) -> Position {
        self.ui.start_pos()
    }

    fn seek_key_steps_n(&self, start_pos: &Position, key: char) -> Option<(usize, Position)> {
        self.ui.seek_key_steps_n(start_pos, key)
    }

    fn seek_key(&self, start_pos: &Position, key: char) -> Option<(Vec<Vec<Direction>>, Position)> {
        self.ui.seek_key(start_pos, key)
    }

    fn input(&self, code: &str) -> Result<Vec<Vec<char>>, Error> {
        let target_codes = self.target_ui.input(code)?;
        let mut code_min_keys_n = vec![0; target_codes.len()];
        for (code_ind, code) in target_codes.iter().enumerate() {
            let mut cur_pos = self.target_ui.start_pos();
            for key in code {
                let (min_steps_n, end_pos) = self
                    .target_ui
                    .seek_key_steps_n(&cur_pos, *key)
                    .ok_or(Error::InvalidKey(*key))?;
                code_min_keys_n[code_ind] += min_steps_n + 1;
                cur_pos = end_pos;
            }
        }

        let Some(min_keys_n) = code_min_keys_n.iter().min().copied() else {
            return Ok(Vec::new());
        };

        let min_keys_codes_n = code_min_keys_n
            .iter()
            .filter(|this_min_keys_n| **this_min_keys_n == min_keys_n)
            .count();
        let mut all_min_keys = vec![Vec::from([Vec::new()]); min_keys_codes_n];
        for (all_min_keys_ind, target_code_ind) in code_min_keys_n
            .iter()
            .enumerate()
            .filter(|(_, this_min_keys_n)| **this_min_keys_n == min_keys_n)
            .map(|(code_ind, _)| code_ind)
            .enumerate()
        {
            let mut cur_pos = self.target_ui.start_pos();
            for key in &target_codes[target_code_ind] {
                let (key_paths, seek_end_pos) = self
                    .target_ui
                    .seek_key(&cur_pos, *key)
                    .ok_or(Error::InvalidKey(*key))?;

                let mut cur_code_all_min_keys = Vec::new();
                for min_keys in &all_min_keys[all_min_keys_ind] {
                    for key_path in &key_paths {
                        let mut cur_min_keys = min_keys.clone();
                        cur_min_keys
                            .extend(key_path.iter().map(|dir| dir.key()).chain(iter::once('A')));

                        cur_code_all_min_keys.push(cur_min_keys);
                    }
                }
                all_min_keys[all_min_keys_ind] = cur_code_all_min_keys;
                cur_pos = seek_end_pos;
            }
        }

        Ok(all_min_keys
            .into_iter()
            .flat_map(|min_keys_for_one_code| min_keys_for_one_code.into_iter())
            .collect())
    }
}

impl Robot {
    pub fn new(target_ui: impl UI + 'static) -> Self {
        Self {
            ui: Keypad::new_directional(),
            target_ui: Box::new(target_ui),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DoorCode {
    text: String,
}

impl DoorCode {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn number(&self) -> usize {
        let number_end_ind = self
            .text
            .chars()
            .filter(|c| !c.is_ascii_digit())
            .next()
            .and_then(|first_non_digit| self.text.find(first_non_digit))
            .unwrap_or(self.text.len());
        self.text[..number_end_ind].parse::<usize>().unwrap_or(0)
    }
}

pub fn read_door_codes<P: AsRef<Path>>(path: P) -> Result<Vec<DoorCode>> {
    let file = File::open(&path)
        .with_context(|| format!("Failed to open given file({}).", path.as_ref().display()))?;
    let reader = BufReader::new(file);
    reader
        .lines()
        .enumerate()
        .map(|(ind, line)| {
            line.with_context(|| {
                format!(
                    "Failed to read line {} in given file({}).",
                    ind + 1,
                    path.as_ref().display()
                )
            })
            .map(|s| DoorCode::new(s.as_str()))
        })
        .collect()
}
