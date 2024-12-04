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
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InconsistentRow(expect_n, real_n) => write!(
                f,
                "Expect {} characters per row, given {}.",
                expect_n, real_n
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
enum Direction {
    Left,
    UpLeft,
    Up,
    UpRight,
    Right,
    DownRight,
    Down,
    DownLeft,
}

impl Direction {
    pub fn all_dirs() -> &'static [Direction] {
        static ALL_DIRS: [Direction; 8] = [
            Direction::Left,
            Direction::UpLeft,
            Direction::Up,
            Direction::UpRight,
            Direction::Right,
            Direction::DownRight,
            Direction::Down,
            Direction::DownLeft,
        ];

        &ALL_DIRS
    }
}

#[derive(Debug, Clone)]
struct Position {
    r: usize,
    c: usize,
}

impl Position {
    pub fn new(r: usize, c: usize) -> Self {
        Self { r, c }
    }

    pub fn along(&self, dir: Direction) -> Option<Self> {
        match dir {
            Direction::Left if self.c > 0 => Some(Position::new(self.r, self.c - 1)),
            Direction::UpLeft if self.r > 0 && self.c > 0 => {
                Some(Position::new(self.r - 1, self.c - 1))
            }
            Direction::Up if self.r > 0 => Some(Position::new(self.r - 1, self.c)),
            Direction::UpRight if self.r > 0 => Some(Position::new(self.r - 1, self.c + 1)),
            Direction::Right => Some(Position::new(self.r, self.c + 1)),
            Direction::DownRight => Some(Position::new(self.r + 1, self.c + 1)),
            Direction::Down => Some(Position::new(self.r + 1, self.c)),
            Direction::DownLeft if self.c > 0 => Some(Position::new(self.r + 1, self.c - 1)),
            _ => None,
        }
    }
}

pub struct LetterMatrix {
    letters: Vec<char>,
    row_n: usize,
    col_n: usize,
}

impl LetterMatrix {
    pub fn search(&self, word: &str) -> usize {
        if word.is_empty() {
            return 0;
        }

        let first_char = word.chars().next().unwrap();
        let mut count = 0;
        for r in 0..self.row_n {
            for c in 0..self.col_n {
                let start_pos = Position::new(r, c);
                if self
                    .letter(&start_pos)
                    .map(|l| *l != first_char)
                    .unwrap_or(true)
                {
                    continue;
                }

                count += Direction::all_dirs()
                    .iter()
                    .filter(|dir| {
                        let mut pos_op = Some(start_pos.clone());
                        word.chars().all(|c| {
                            if let Some(pos) = pos_op.take() {
                                let is_match = self.letter(&pos).map(|l| *l == c).unwrap_or(false);
                                pos_op = pos.along(**dir);
                                is_match
                            } else {
                                false
                            }
                        })
                    })
                    .count();
            }
        }

        count
    }

    fn letter(&self, pos: &Position) -> Option<&char> {
        if pos.r >= self.row_n || pos.c >= self.col_n {
            None
        } else {
            self.letters.get(pos.r * self.col_n + pos.c)
        }
    }
}

struct LetterMatrixBuilder {
    letters: Vec<char>,
    row_n: usize,
    col_n: Option<usize>,
}

impl LetterMatrixBuilder {
    pub fn new() -> Self {
        Self {
            letters: Vec::new(),
            row_n: 0,
            col_n: None,
        }
    }

    pub fn add_row(&mut self, row_str: &str) -> Result<(), Error> {
        let char_n = row_str.chars().count();
        if *self.col_n.get_or_insert(char_n) != char_n {
            return Err(Error::InconsistentRow(self.col_n.unwrap(), char_n));
        }

        self.letters.extend(row_str.chars());
        self.row_n += 1;

        Ok(())
    }

    pub fn build(self) -> LetterMatrix {
        LetterMatrix {
            letters: self.letters,
            row_n: self.row_n,
            col_n: self.col_n.unwrap_or(0),
        }
    }
}

pub fn read_letter_mat<P: AsRef<Path>>(path: P) -> Result<LetterMatrix> {
    let file = File::open(&path)
        .with_context(|| format!("Failed to open given file({}).", path.as_ref().display()))?;
    let reader = BufReader::new(file);
    let mut builder = LetterMatrixBuilder::new();
    for (ind, line) in reader.lines().enumerate() {
        let line = line.with_context(|| {
            format!(
                "Failed to read line {} from given file({}).",
                ind + 1,
                path.as_ref().display()
            )
        })?;
        builder.add_row(&line).with_context(|| {
            format!("Failed to add one row(line {}) to letter matrix.", ind + 1)
        })?;
    }

    Ok(builder.build())
}
