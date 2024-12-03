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
        self.mul_sum_enable(false, true).0
    }

    pub fn mul_sum_enable(&self, enable_do: bool, init_do_mul: bool) -> (usize, bool) {
        static MUL_INST_PATTERN: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"mul\((\d+),(\d+)\)|do\(\)|don't\(\)").unwrap());

        let mut sum = 0;
        let mut do_mul = init_do_mul;
        for caps in MUL_INST_PATTERN.captures_iter(&self.text) {
            if caps.get(1).is_none() {
                if enable_do {
                    if &caps[0] == "do()" {
                        do_mul = true;
                    } else if &caps[0] == "don't()" {
                        do_mul = false;
                    }
                }
            } else if !enable_do || do_mul {
                let l_factor = caps[1].parse::<usize>().unwrap();
                let r_factor = caps[2].parse::<usize>().unwrap();
                sum += l_factor * r_factor;
            }
        }

        (sum, do_mul)
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
