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
enum Error {
    NoPatterns,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NoPatterns => write!(
                f,
                "Expect one line for patterns at the start of given file."
            ),
        }
    }
}

impl error::Error for Error {}

#[derive(Debug, Parser)]
pub struct CLIArgs {
    pub input_path: PathBuf,
}

#[derive(Debug)]
pub struct Design {
    text: String,
}

impl Display for Design {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.text.fmt(f)
    }
}

impl Design {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
        }
    }

    pub fn is_possible_with(&self, patterns: &[String]) -> bool {
        self.possible_ways_n(patterns) > 0
    }

    pub fn possible_ways_n(&self, patterns: &[String]) -> usize {
        let mut checked_design_ways = vec![None; self.text.len()];
        self.possible_ways_n_recur(patterns, 0, &mut checked_design_ways)
    }

    fn possible_ways_n_recur(
        &self,
        patterns: &[String],
        ind: usize,
        checked_design_ways: &mut [Option<usize>],
    ) -> usize {
        let text_len = self.text.len();
        if ind >= text_len {
            return 1;
        }

        if let Some(checked_ways_n) = checked_design_ways[ind] {
            return checked_ways_n;
        }

        let mut ways_n = 0;
        for pattern in patterns.iter().filter(|p| ind + p.len() <= text_len) {
            let pat_len = pattern.len();
            if &self.text[ind..(ind + pat_len)] == pattern {
                ways_n += self.possible_ways_n_recur(patterns, ind + pat_len, checked_design_ways);
            }
        }
        debug_assert!(checked_design_ways[ind].is_none());
        checked_design_ways[ind].get_or_insert(ways_n);

        ways_n
    }
}

pub fn read_pattern_design<P: AsRef<Path>>(path: P) -> Result<(Vec<String>, Vec<Design>)> {
    let file = File::open(&path)
        .with_context(|| format!("Failed to open given file({}).", path.as_ref().display()))?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    let patterns = lines
        .next()
        .ok_or(Error::NoPatterns)?
        .with_context(|| {
            format!(
                "Failed to read line 1 of given file({}).",
                path.as_ref().display()
            )
        })?
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();
    let designs = lines
        .enumerate()
        .filter(|(_, line)| line.as_ref().is_ok_and(|s| !s.is_empty()) || line.is_err())
        .map(|(ind, line)| {
            line.with_context(|| {
                format!(
                    "Failed to read line {} of given file({}).",
                    ind + 2,
                    path.as_ref().display()
                )
            })
            .map(|s| Design::new(s.as_str()))
        })
        .collect::<Result<Vec<_>>>()?;

    Ok((patterns, designs))
}
