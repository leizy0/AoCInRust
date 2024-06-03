use std::{
    collections::HashMap,
    fmt::Display,
    fs::File,
    io::{self, BufRead, BufReader},
    path::{Path, PathBuf},
};

use clap::Parser;

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    InvalidQuestionChar(char),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IOError(ioe) => write!(f, "I/O error: {}", ioe),
            Error::InvalidQuestionChar(c) => write!(
                f,
                "Invalid character for question code: {}, expect lowercase of ascii letters",
                c
            ),
        }
    }
}

#[derive(Debug, Parser)]
pub struct CliArgs {
    pub input_path: PathBuf,
}

pub struct GroupAnwser {
    que_app_counts: HashMap<char, usize>,
    mem_count: usize,
}

impl GroupAnwser {
    pub fn new() -> Self {
        Self {
            que_app_counts: HashMap::new(),
            mem_count: 0,
        }
    }

    pub fn any_app_n(&self) -> usize {
        self.que_app_counts.len()
    }

    pub fn all_app_n(&self) -> usize {
        self.que_app_counts
            .iter()
            .filter(|(_, n)| **n == self.mem_count)
            .count()
    }

    pub fn is_empty(&self) -> bool {
        self.que_app_counts.is_empty()
    }

    fn add_answer(&mut self, text: &str) -> Result<(), Error> {
        for q in text.chars() {
            if q.is_ascii_alphabetic() && q.is_ascii_lowercase() {
                *self.que_app_counts.entry(q).or_insert(0) += 1;
            } else {
                return Err(Error::InvalidQuestionChar(q));
            }
        }

        self.mem_count += 1;
        Ok(())
    }
}

pub fn read_ga<P: AsRef<Path>>(path: P) -> Result<Vec<GroupAnwser>, Error> {
    let file = File::open(path).map_err(Error::IOError)?;
    let reader = BufReader::new(file);
    let mut cur_ga: Option<GroupAnwser> = None;
    let mut grp_answers = Vec::new();
    for line in reader.lines() {
        let s = line.map_err(Error::IOError)?;
        if s.is_empty() {
            if let Some(ga) = cur_ga.take() {
                if !ga.is_empty() {
                    grp_answers.push(ga);
                }
            }
        } else {
            cur_ga
                .get_or_insert_with(GroupAnwser::new)
                .add_answer(s.as_str())?;
        }
    }

    // Never forget the last one!
    if let Some(ga) = cur_ga.take() {
        if !ga.is_empty() {
            grp_answers.push(ga);
        }
    }

    Ok(grp_answers)
}
