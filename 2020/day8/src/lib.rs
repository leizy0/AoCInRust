use std::{
    fmt::Display,
    fs::File,
    io::{self, BufRead, BufReader},
    path::{Path, PathBuf},
};

use clap::Parser;
use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    InvalidInstText(String),
    InvalidInstPtr(usize, usize),
    InvalidJmp(usize, isize),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IOError(ioe) => write!(f, "I/O error: {}", ioe),
            Error::InvalidInstText(s) => write!(f, "Invalid instruction text: {}", s),
            Error::InvalidInstPtr(p, len) => write!(
                f,
                "Invalid instruction pointer({}), expect in range([0, {}))",
                p, len
            ),
            Error::InvalidJmp(p, offset) => {
                write!(f, "Invalid jump(from {}, offset is {})", p, offset)
            }
        }
    }
}

#[derive(Debug, Parser)]
pub struct CliArgs {
    pub input_path: PathBuf,
}

#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    Acc(isize),
    Nop(isize),
    Jmp(isize),
}

impl TryFrom<&str> for Instruction {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        type InstCtorFn = fn(&str) -> Option<Instruction>;
        static CONSTRUCTORS: [InstCtorFn; 3] = [
            Instruction::try_new_acc,
            Instruction::try_new_nop,
            Instruction::try_new_jmp,
        ];
        CONSTRUCTORS
            .iter()
            .filter_map(|f| f(value))
            .next()
            .ok_or(Error::InvalidInstText(value.to_string()))
    }
}

impl Instruction {
    pub fn try_new_acc(text: &str) -> Option<Self> {
        static PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"acc ([+|-]?\d+)").unwrap());

        PATTERN
            .captures(text)
            .map(|caps| Self::Acc(caps[1].parse::<isize>().unwrap()))
    }

    pub fn try_new_nop(text: &str) -> Option<Self> {
        static PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"nop ([+|-]?\d+)").unwrap());

        PATTERN
            .captures(text)
            .map(|caps| Self::Nop(caps[1].parse::<isize>().unwrap()))
    }

    pub fn try_new_jmp(text: &str) -> Option<Self> {
        static PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"jmp ([+|-]?\d+)").unwrap());

        PATTERN
            .captures(text)
            .map(|caps| Self::Jmp(caps[1].parse::<isize>().unwrap()))
    }
}

pub struct GCState {
    pub inst_ptr: usize,
    pub acc: isize,
}

impl GCState {
    pub fn new() -> Self {
        Self {
            inst_ptr: 0,
            acc: 0,
        }
    }
}

pub struct GameConsole;
impl GameConsole {
    pub fn new() -> Self {
        Self
    }

    pub fn run_while(
        &mut self,
        code: &[Instruction],
        cond: &mut dyn FnMut(&GCState) -> bool,
    ) -> Result<GCState, Error> {
        let mut state = GCState::new();
        while cond(&state) {
            let inst = code
                .get(state.inst_ptr)
                .ok_or(Error::InvalidInstPtr(state.inst_ptr, code.len()))?;
            match inst {
                Instruction::Acc(n) => {
                    state.acc += n;
                    state.inst_ptr += 1;
                }
                Instruction::Nop(_) => state.inst_ptr += 1,
                Instruction::Jmp(offset) => {
                    state.inst_ptr = state
                        .inst_ptr
                        .checked_add_signed(*offset)
                        .ok_or(Error::InvalidJmp(state.inst_ptr, *offset))?
                }
            }
        }

        Ok(state)
    }
}

pub fn read_code<P: AsRef<Path>>(path: P) -> Result<Vec<Instruction>, Error> {
    let file = File::open(path).map_err(Error::IOError)?;
    let reader = BufReader::new(file);
    reader
        .lines()
        .map(|l| {
            l.map_err(Error::IOError)
                .and_then(|s| Instruction::try_from(s.as_str()))
        })
        .collect::<Result<Vec<_>, Error>>()
}
