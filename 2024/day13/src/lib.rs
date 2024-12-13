use std::{
    error,
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use clap::Parser;
use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Debug)]
enum Error {
    NoButtonBLine,
    NoPrize,
    InvalidButtonText(String),
    InvalidPrizeText(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NoButtonBLine => {
                write!(f, "Expect one more line for button, but can't find one.")
            }
            Error::NoPrize => write!(f, "Expect one more line for prize, but can't find one."),
            Error::InvalidButtonText(s) => write!(f, "Invalid text({}) for button.", s),
            Error::InvalidPrizeText(s) => write!(f, "Invalid text({}) for prize.", s),
        }
    }
}

impl error::Error for Error {}

#[derive(Debug, Parser)]
pub struct CLIArgs {
    pub input_path: PathBuf,
}

type Vector = (usize, usize);
type Position = (usize, usize);

#[derive(Debug, Clone)]
pub struct Solution {
    button_a_count: usize,
    button_b_count: usize,
}

impl Solution {
    pub fn new(button_a_count: usize, button_b_count: usize) -> Self {
        Self {
            button_a_count,
            button_b_count,
        }
    }

    pub fn tokens_n(&self) -> usize {
        self.button_a_count * 3 + self.button_b_count
    }
}

#[derive(Debug)]
pub struct ClawMachine {
    button_a_move: Vector,
    button_b_move: Vector,
    prize: Position,
}

impl ClawMachine {
    pub fn new(button_a_move: Vector, button_b_move: Vector, prize: Position) -> Self {
        Self {
            button_a_move,
            button_b_move,
            prize,
        }
    }

    pub fn solutions(&self) -> Vec<Solution> {
        let mut solutions = Vec::new();
        let x_b_factor = self.button_b_move.0 * self.button_a_move.1;
        let x_target = self.prize.0 * self.button_a_move.1;
        let y_b_factor = self.button_b_move.1 * self.button_a_move.0;
        let y_target = self.prize.1 * self.button_a_move.0;
        let (l_b_factor, s_b_factor, l_target, s_target) = if x_b_factor >= y_b_factor {
            (x_b_factor, y_b_factor, x_target, y_target)
        } else {
            (y_b_factor, x_b_factor, y_target, x_target)
        };

        if l_target > s_target {
            let target_diff = l_target - s_target;
            let factor_diff = l_b_factor - s_b_factor;
            if target_diff % factor_diff == 0 {
                let b_count = target_diff / factor_diff;
                let a_x_target = self.prize.0 - self.button_b_move.0 * b_count;
                if a_x_target % self.button_a_move.0 == 0 {
                    solutions.push(Solution::new(a_x_target / self.button_a_move.0, b_count));
                }
            }
        } else if l_target == s_target {
            if l_b_factor == s_b_factor {
                let mut a_count = 0;
                while self.button_a_move.0 * a_count > self.prize.0 {
                    let b_x_target = self.prize.0 - self.button_a_move.0 * a_count;
                    if b_x_target % self.button_b_move.0 == 0 {
                        solutions.push(Solution::new(a_count, b_x_target / self.button_b_move.0));
                    }

                    a_count += 1;
                }
            }
        }

        solutions
    }
}

pub fn read_machines<P: AsRef<Path>>(path: P) -> Result<Vec<ClawMachine>> {
    let file = File::open(&path)
        .with_context(|| format!("Failed to open given file({}).", path.as_ref().display()))?;
    let reader = BufReader::new(file);
    let mut machines = Vec::new();
    let mut lines = reader.lines();
    let mut line_ind = 1;
    loop {
        if let Some(button_a_line) = lines.next() {
            let button_a_line = button_a_line.with_context(|| {
                format!(
                    "Failed to read line {} of given file({}).",
                    line_ind,
                    path.as_ref().display()
                )
            })?;
            line_ind += 1;
            if button_a_line.is_empty() {
                continue;
            }

            let button_a_move = read_button(&button_a_line)?;
            let button_b_move =
                read_button(&lines.next().ok_or(Error::NoButtonBLine)?.with_context(|| {
                    format!(
                        "Failed to read line {} of given file({}).",
                        line_ind,
                        path.as_ref().display()
                    )
                })?)?;
            line_ind += 1;
            let prize = read_prize(&lines.next().ok_or(Error::NoPrize)?.with_context(|| {
                format!(
                    "Failed to read line {} of given file({}).",
                    line_ind,
                    path.as_ref().display()
                )
            })?)?;
            line_ind += 1;
            machines.push(ClawMachine::new(button_a_move, button_b_move, prize));
        } else {
            break;
        }
    }

    Ok(machines)
}

fn read_button(text: &str) -> Result<(usize, usize), Error> {
    static BUTTON_PATTERN: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"Button \w+: X\+(\d+), Y\+(\d+)").unwrap());

    if let Some(caps) = BUTTON_PATTERN.captures(text) {
        Ok((
            caps[1].parse::<usize>().unwrap(),
            caps[2].parse::<usize>().unwrap(),
        ))
    } else {
        Err(Error::InvalidButtonText(text.to_string()))
    }
}

fn read_prize(text: &str) -> Result<(usize, usize), Error> {
    static PRIZE_PATTERN: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"Prize: X=(\d+), Y=(\d+)").unwrap());

    if let Some(caps) = PRIZE_PATTERN.captures(text) {
        Ok((
            caps[1].parse::<usize>().unwrap(),
            caps[2].parse::<usize>().unwrap(),
        ))
    } else {
        Err(Error::InvalidPrizeText(text.to_string()))
    }
}
