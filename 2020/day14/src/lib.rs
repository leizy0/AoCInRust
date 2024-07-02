use std::{
    error,
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
    InvalidOpText(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IOError(ioe) => write!(f, "I/O error: {}", ioe),
            Error::InvalidOpText(s) => write!(f, "Invalid operation text: {}", s),
        }
    }
}

impl error::Error for Error {}

#[derive(Debug, Parser)]
pub struct CLIArgs {
    pub input_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct Mask {
    float_mask: usize,
    fixed_mask: usize,
}

impl Mask {
    fn proc_value(&self, value: usize) -> usize {
        value & self.float_mask | self.fixed_mask
    }

    fn proc_addr(&self, addr: usize) -> MemAddrGroup {
        let fixed_addr = addr & (!self.float_mask) | self.fixed_mask;
        MemAddrGroup::new(self.float_mask, fixed_addr)
    }
}

pub enum Operation {
    SetMask(Mask),
    SetMemWithMask(usize, usize),
}

impl TryFrom<&str> for Operation {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        type OpCtorFn = fn(&str) -> Option<Operation>;
        static OPERATION_CTORS: Lazy<Vec<OpCtorFn>> = Lazy::new(|| {
            vec![
                Operation::try_new_set_mask,
                Operation::try_new_set_mem_with_mask,
            ]
        });

        OPERATION_CTORS
            .iter()
            .filter_map(|f| f(value))
            .next()
            .ok_or(Error::InvalidOpText(value.to_string()))
    }
}

impl Operation {
    pub fn try_new_set_mask(text: &str) -> Option<Self> {
        static PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"mask = ([10Xx]{36})").unwrap());

        PATTERN.captures(text).map(|caps| {
            let mut and_mask = 0;
            let mut or_mask = 0;
            for c in caps[1].chars() {
                and_mask <<= 1;
                or_mask <<= 1;
                match c {
                    '1' => {
                        or_mask |= 1;
                    }
                    '0' => (),
                    'X' | 'x' => {
                        and_mask |= 1;
                    }
                    invalid_char => unreachable!(
                        "Invalid character({}) pass regular expression!",
                        invalid_char
                    ),
                }
            }

            Operation::SetMask(Mask {
                float_mask: and_mask,
                fixed_mask: or_mask,
            })
        })
    }

    pub fn try_new_set_mem_with_mask(text: &str) -> Option<Operation> {
        static PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"mem\[(\d+)\] = (\d+)").unwrap());

        PATTERN.captures(text).map(|caps| {
            Operation::SetMemWithMask(
                caps[1].parse::<usize>().unwrap(),
                caps[2].parse::<usize>().unwrap(),
            )
        })
    }
}

struct MemAddrIterator {
    float_bit_masks: Vec<usize>,
    fixed_addr: usize,
    float_ind: usize,
    float_limit: usize,
}

impl Iterator for MemAddrIterator {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.float_ind < self.float_limit {
            let mut float_addr = self.float_ind;
            let mut addr = self.fixed_addr;
            for mask in &self.float_bit_masks {
                if float_addr == 0 {
                    break;
                }

                if float_addr & 1 != 0 {
                    addr |= mask;
                }

                float_addr >>= 1;
            }
            self.float_ind += 1;

            Some(addr)
        } else {
            None
        }
    }
}

impl MemAddrIterator {
    pub fn new(mut float_mask: usize, fixed_addr: usize) -> Self {
        let mut float_bit_masks = Vec::new();
        let mut bit_mask = 1;
        while float_mask > 0 {
            if float_mask & 1 != 0 {
                float_bit_masks.push(bit_mask);
            }

            bit_mask <<= 1;
            float_mask >>= 1;
        }

        let float_limit = 1 << float_bit_masks.len();
        Self {
            float_bit_masks,
            fixed_addr,
            float_ind: 0,
            float_limit,
        }
    }
}

#[derive(Debug, Clone)]
struct MemAddrGroup {
    float_mask: usize,
    fixed_addr: usize,
}

impl IntoIterator for &MemAddrGroup {
    type Item = usize;

    type IntoIter = MemAddrIterator;

    fn into_iter(self) -> Self::IntoIter {
        MemAddrIterator::new(self.float_mask, self.fixed_addr)
    }
}

impl MemAddrGroup {
    pub fn new(float_mask: usize, fixed_addr: usize) -> Self {
        debug_assert!(
            float_mask & fixed_addr == 0,
            "The range of fixed address should be out of range of float mask."
        );
        Self {
            float_mask,
            fixed_addr,
        }
    }

    pub fn new_single(addr: usize) -> Self {
        Self {
            float_mask: 0,
            fixed_addr: addr,
        }
    }

    pub fn contain(&self, addr: usize) -> bool {
        addr & !self.float_mask == self.fixed_addr
    }
}

#[derive(Debug, Clone)]
struct MemAssignment {
    addr_grp: MemAddrGroup,
    value: usize,
}

impl MemAssignment {
    pub fn new(addr_grp: MemAddrGroup, value: usize) -> Self {
        Self { addr_grp, value }
    }

    pub fn addr_grp(&self) -> &MemAddrGroup {
        &self.addr_grp
    }

    pub fn value(&self) -> usize {
        self.value
    }
}

pub enum MaskMode {
    MaskAddr,
    MaskValue,
}

pub struct SPComputer {
    mem: Vec<MemAssignment>,
    mask: Mask,
    mask_mode: MaskMode,
}

impl SPComputer {
    pub fn new(mask_mode: MaskMode) -> Self {
        Self {
            mem: Vec::new(),
            mask: Mask {
                float_mask: 0,
                fixed_mask: 0,
            },
            mask_mode,
        }
    }

    pub fn execute(&mut self, op: &Operation) {
        match op {
            Operation::SetMask(new_mask) => self.mask = new_mask.clone(),
            Operation::SetMemWithMask(ind, value) => {
                self.set_mem(*ind, *value);
            }
        }
    }

    pub fn non_zero_mem_sum(&self) -> usize {
        let mut sum = 0;
        for (ind, mem_ass) in self.mem.iter().enumerate() {
            let mut final_left_count = 0;
            for addr in mem_ass.addr_grp() {
                if !self.mem[(ind + 1)..]
                    .iter()
                    .any(|later_ass| later_ass.addr_grp().contain(addr))
                {
                    final_left_count += 1;
                }
            }

            sum += final_left_count * mem_ass.value();
        }

        sum
    }

    fn set_mem(&mut self, ind: usize, value: usize) {
        let mem_ass = match self.mask_mode {
            MaskMode::MaskAddr => MemAssignment::new(self.mask.proc_addr(ind), value),
            MaskMode::MaskValue => {
                MemAssignment::new(MemAddrGroup::new_single(ind), self.mask.proc_value(value))
            }
        };

        self.mem.push(mem_ass);
    }
}

pub fn read_ops<P: AsRef<Path>>(path: P) -> Result<Vec<Operation>, Error> {
    let file = File::open(path).map_err(Error::IOError)?;
    let reader = BufReader::new(file);
    reader
        .lines()
        .map(|l| {
            l.map_err(Error::IOError)
                .and_then(|s| Operation::try_from(s.as_str()))
        })
        .collect()
}
