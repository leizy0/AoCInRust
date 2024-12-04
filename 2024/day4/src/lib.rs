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
pub struct Part1CLIArgs {
    pub input_path: PathBuf,
}

#[derive(Debug, Parser)]
pub struct Part2CLIArgs {
    pub input_path: PathBuf,
    pub patterns_path: PathBuf,
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

    pub fn offset(&self, offset: &Position) -> Self {
        Position::new(self.r + offset.r, self.c + offset.c)
    }
}

pub struct LetterMatrix {
    letters: Vec<char>,
    row_n: usize,
    col_n: usize,
}

impl LetterMatrix {
    pub fn search_word(&self, word: &str) -> usize {
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

    pub fn search_pats(&self, pats: &[Pattern]) -> usize {
        let mut count = 0;
        for r in 0..self.row_n {
            for c in 0..self.col_n {
                count += pats
                    .iter()
                    .filter(|pat| pat.is_match(self, &Position::new(r, c)))
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

pub struct Pattern {
    units: Vec<Option<char>>,
    row_n: usize,
    col_n: usize,
}

impl From<LetterMatrix> for Pattern {
    fn from(value: LetterMatrix) -> Self {
        let units = value
            .letters
            .iter()
            .map(|c| Some(*c).filter(|c| *c != '.'))
            .collect::<Vec<_>>();
        Self {
            units,
            row_n: value.row_n,
            col_n: value.col_n,
        }
    }
}

impl Pattern {
    fn is_match(&self, mat: &LetterMatrix, pos: &Position) -> bool {
        for r in 0..self.row_n {
            for c in 0..self.col_n {
                let pat_pos = Position::new(r, c);
                let mat_pos = pat_pos.offset(&pos);

                if self
                    .unit(&pat_pos)
                    .map(|c| mat.letter(&mat_pos).map(|l| c != l).unwrap_or(true))
                    .unwrap_or(false)
                {
                    return false;
                }
            }
        }

        true
    }

    fn unit(&self, pos: &Position) -> Option<&char> {
        if pos.r >= self.row_n || pos.c >= self.col_n {
            None
        } else {
            self.units
                .get(pos.r * self.col_n + pos.c)
                .and_then(|op| op.as_ref())
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

pub fn read_patterns<P: AsRef<Path>>(path: P) -> Result<Vec<Pattern>> {
    let file = File::open(&path)
        .with_context(|| format!("Failed to open given file({}).", path.as_ref().display()))?;
    let reader = BufReader::new(file);

    let mut patterns = Vec::new();
    let mut builder_op: Option<LetterMatrixBuilder> = None;

    for (ind, line) in reader.lines().enumerate() {
        let line = line.with_context(|| {
            format!(
                "Failed to read line {} from given file({}).",
                ind + 1,
                path.as_ref().display()
            )
        })?;

        if line.is_empty() {
            if let Some(builder) = builder_op.take() {
                patterns.push(builder.build().into());
            }
        } else {
            builder_op
                .get_or_insert(LetterMatrixBuilder::new())
                .add_row(&line)
                .with_context(|| {
                    format!("Failed to add one row(line {}) to letter matrix.", ind + 1)
                })?;
        }
    }

    if let Some(builder) = builder_op.take() {
        patterns.push(builder.build().into());
    }

    Ok(patterns)
}
