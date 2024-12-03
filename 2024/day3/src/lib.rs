use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use clap::Parser;
use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Debug, Parser)]
pub struct CLIArgs {
    pub input_path: PathBuf,
}

pub struct Instruction {
    text: String,
}

impl Instruction {
    pub fn new(s: &str) -> Self {
        Self {
            text: s.to_string(),
        }
    }

    pub fn mul_sum(&self) -> usize {
        static MUL_INST_PATTERN: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"mul\((\d+),(\d+)\)").unwrap());

        let mut sum = 0;
        for caps in MUL_INST_PATTERN.captures_iter(&self.text) {
            debug_assert!(caps.len() == 3);
            let l_factor = caps[1].parse::<usize>().unwrap();
            let r_factor = caps[2].parse::<usize>().unwrap();
            sum += l_factor * r_factor;
        }

        sum
    }
}

pub fn read_insts<P: AsRef<Path>>(path: P) -> Result<Vec<Instruction>> {
    let file = File::open(&path)
        .with_context(|| format!("Failed to open given file({}).", path.as_ref().display()))?;
    let reader = BufReader::new(file);
    reader
        .lines()
        .enumerate()
        .map(|(ind, l)| {
            l.with_context(|| {
                format!(
                    "Failed to read line {} of given file({}).",
                    ind + 1,
                    path.as_ref().display()
                )
            })
            .map(|s| Instruction::new(s.as_str()))
        })
        .collect()
}
