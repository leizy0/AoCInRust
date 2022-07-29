use std::{io::{self, BufReader, BufRead}, fs::File, collections::{HashSet, HashMap}, fmt::Display};

use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    SampleParseError(String),
    RegisterGroupParseError(String),
    InstructionFormatError(String),
    InvalidOpCode(usize),
    InvalidRegisterIndex(usize),
    OpCodeNotfound(usize),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IOError(ioe) => write!(f, "IO Error({})", ioe),
            Error::SampleParseError(s) => write!(f, "Failed to parse sample text({})", s),
            Error::RegisterGroupParseError(s) => write!(f, "Failed to parse register group text({})", s),
            Error::InstructionFormatError(s) => write!(f, "Failed to parse instrction text({})", s),
            Error::InvalidOpCode(op_code) => write!(f, "Invalid operation code({})", op_code),
            Error::InvalidRegisterIndex(index) => write!(f, "Invalid register index({})", index),
            Error::OpCodeNotfound(op_code) => write!(f, "Can't find operation code({}) in map", op_code),
            
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct RegisterGroup {
    regs: [usize; 4],
}

impl TryFrom<&str> for RegisterGroup {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        static REGISTER_GROUP_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s*\[(\d+), (\d+), (\d+), (\d+)\]\s*").unwrap());

        let caps = REGISTER_GROUP_PATTERN.captures(value)
            .ok_or(Error::RegisterGroupParseError(value.to_string()))?;
        Ok(RegisterGroup {
            regs: [
                caps[1].parse::<usize>().unwrap(),
                caps[2].parse::<usize>().unwrap(),
                caps[3].parse::<usize>().unwrap(),
                caps[4].parse::<usize>().unwrap(),
            ]
        })
    }
}

impl Display for RegisterGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.regs)
    }
}

impl RegisterGroup {
    pub fn new() -> Self {
        RegisterGroup { regs: [0; 4] }
    }

    pub fn reg(&self, ind: usize) -> Result<&usize, Error> {
        if ind <= 3 {
            Ok(&self.regs[ind])
        } else {
            Err(Error::InvalidRegisterIndex(ind))
        }
    }

    pub fn reg_mut(&mut self, ind: usize) -> Result<&mut usize, Error> {
        if ind <= 3 {
            Ok(&mut self.regs[ind])
        } else {
            Err(Error::InvalidRegisterIndex(ind))
        }
    }
}

type Oprands = [usize; 3];

pub struct Instruction {
    op_code: usize,
    oprands: Oprands,
}

impl TryFrom<&str> for Instruction {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        static INST_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\d+) (\d+) (\d+) (\d+)").unwrap());
        const OP_CODE_COUNT: usize = 16;
        const OP_CODE_MAX: usize = OP_CODE_COUNT - 1;

        let caps = INST_PATTERN.captures(value)
            .ok_or(Error::InstructionFormatError(value.to_string()))?;
        let op_code = caps[1].parse::<usize>().unwrap();
        if op_code > OP_CODE_MAX {
            return Err(Error::InvalidOpCode(op_code));
        }
        Ok(Instruction {
            op_code,
            oprands: [
                caps[2].parse::<usize>().unwrap(),
                caps[3].parse::<usize>().unwrap(),
                caps[4].parse::<usize>().unwrap(),
            ]
        })
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "opcode: {}, oprands: {:?}", self.op_code, self.oprands)
    }
}

pub struct ExeSample {
    before: RegisterGroup,
    after:RegisterGroup,
    inst: Instruction,
}

impl ExeSample {
    pub fn from_str(before: &str, inst: &str, after: &str) -> Result<ExeSample, Error> {
        static SAMPLE_BEFORE_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"Before:(.+)").unwrap());
        static SAMPLE_AFTER_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"After:(.+)").unwrap());

        let before_caps = SAMPLE_BEFORE_PATTERN.captures(before)
            .ok_or(Error::SampleParseError(before.to_string()))?;
        let before = RegisterGroup::try_from(&before_caps[1])?;
        let after_caps = SAMPLE_AFTER_PATTERN.captures(after)
            .ok_or(Error::SampleParseError(after.to_string()))?;
        let after = RegisterGroup::try_from(&after_caps[1])?;

        let inst = Instruction::try_from(inst)?;
        Ok(ExeSample{ before, after, inst })
    }

    pub fn op_code(&self) -> usize {
        self.inst.op_code
    }
}

pub trait Operation: Sync + Send {
    fn execute(&self, oprands: &Oprands, regs: &mut RegisterGroup) -> Result<(), Error>;
}

pub fn load_samples(input_path: &str) -> Result<Vec<ExeSample>, Error> {
    let input_file = File::open(input_path).map_err(Error::IOError)?;
    let reader = BufReader::new(input_file);
    let lines = reader.lines()
        .filter(|s| s.is_err() || !s.as_ref().unwrap().is_empty())
        .collect::<Result<Vec<_>, _>>()
        .map_err(Error::IOError)?;

    let sample_count = lines.len() / 3;
    let mut samples = Vec::with_capacity(sample_count);
    for chunk in lines.chunks_exact(3) {
        samples.push(ExeSample::from_str(&chunk[0], &chunk[1], &chunk[2])?);
    }

    Ok(samples)
}

pub fn load_insts(input_path: &str) -> Result<Vec<Instruction>, Error> {
    let input_file = File::open(input_path).map_err(Error::IOError)?;
    let reader = BufReader::new(input_file);
    reader.lines()
        .filter(|s| s.is_err() || !s.as_ref().unwrap().is_empty())
        .map(|se| se.map_err(Error::IOError)
            .and_then(|s| Instruction::try_from(s.as_str())))
        .collect::<Result<Vec<_>, _>>()
}

#[derive(Clone)]
struct AddR;
impl Operation for AddR {
    fn execute(&self, oprands: &Oprands, regs: &mut RegisterGroup) -> Result<(), Error> {
        Ok(*regs.reg_mut(oprands[2])? = regs.reg(oprands[0])? + regs.reg(oprands[1])?)
    }
}

#[derive(Clone)]
struct AddI;
impl Operation for AddI {
    fn execute(&self, oprands: &Oprands, regs: &mut RegisterGroup) -> Result<(), Error> {
        Ok(*regs.reg_mut(oprands[2])? = regs.reg(oprands[0])? + oprands[1])
    }
}

#[derive(Clone)]
struct MulR;
impl Operation for MulR {
    fn execute(&self, oprands: &Oprands, regs: &mut RegisterGroup) -> Result<(), Error> {
        Ok(*regs.reg_mut(oprands[2])? = regs.reg(oprands[0])? * regs.reg(oprands[1])?)
    }
}

#[derive(Clone)]
struct MulI;
impl Operation for MulI {
    fn execute(&self, oprands: &Oprands, regs: &mut RegisterGroup) -> Result<(), Error> {
        Ok(*regs.reg_mut(oprands[2])? = regs.reg(oprands[0])? * oprands[1])
    }
}

#[derive(Clone)]
struct BanR;
impl Operation for BanR {
    fn execute(&self, oprands: &Oprands, regs: &mut RegisterGroup) -> Result<(), Error> {
        Ok(*regs.reg_mut(oprands[2])? = regs.reg(oprands[0])? & regs.reg(oprands[1])?)
    }
}

#[derive(Clone)]
struct BanI;
impl Operation for BanI {
    fn execute(&self, oprands: &Oprands, regs: &mut RegisterGroup) -> Result<(), Error> {
        Ok(*regs.reg_mut(oprands[2])? = regs.reg(oprands[0])? & oprands[1])
    }
}

#[derive(Clone)]
struct BorR;
impl Operation for BorR {
    fn execute(&self, oprands: &Oprands, regs: &mut RegisterGroup) -> Result<(), Error> {
        Ok(*regs.reg_mut(oprands[2])? = regs.reg(oprands[0])? | regs.reg(oprands[1])?)
    }
}

#[derive(Clone)]
struct BorI;
impl Operation for BorI {
    fn execute(&self, oprands: &Oprands, regs: &mut RegisterGroup) -> Result<(), Error> {
        Ok(*regs.reg_mut(oprands[2])? = regs.reg(oprands[0])? | oprands[1])
    }
}

#[derive(Clone)]
struct SetR;
impl Operation for SetR {
    fn execute(&self, oprands: &Oprands, regs: &mut RegisterGroup) -> Result<(), Error> {
        Ok(*regs.reg_mut(oprands[2])? = *regs.reg(oprands[0])?)
    }
}

#[derive(Clone)]
struct SetI;
impl Operation for SetI {
    fn execute(&self, oprands: &Oprands, regs: &mut RegisterGroup) -> Result<(), Error> {
        Ok(*regs.reg_mut(oprands[2])? = oprands[0])
    }
}

#[derive(Clone)]
struct GtIR;
impl Operation for GtIR {
    fn execute(&self, oprands: &Oprands, regs: &mut RegisterGroup) -> Result<(), Error> {
        Ok(*regs.reg_mut(oprands[2])? = if oprands[0] > *regs.reg(oprands[1])? {
            1
        } else {
            0
        })
    }
}

#[derive(Clone)]
struct GtRI;
impl Operation for GtRI {
    fn execute(&self, oprands: &Oprands, regs: &mut RegisterGroup) -> Result<(), Error> {
        Ok(*regs.reg_mut(oprands[2])? = if *regs.reg(oprands[0])? > oprands[1] {
            1
        } else {
            0
        })
    }
}

#[derive(Clone)]
struct GtRR;
impl Operation for GtRR {
    fn execute(&self, oprands: &Oprands, regs: &mut RegisterGroup) -> Result<(), Error> {
        Ok(*regs.reg_mut(oprands[2])? = if regs.reg(oprands[0])? > regs.reg(oprands[1])? {
            1
        } else {
            0
        })
    }
}

#[derive(Clone)]
struct EqIR;
impl Operation for EqIR {
    fn execute(&self, oprands: &Oprands, regs: &mut RegisterGroup) -> Result<(), Error> {
        Ok(*regs.reg_mut(oprands[2])? = if oprands[0] == *regs.reg(oprands[1])? {
            1
        } else {
            0
        })
    }
}

#[derive(Clone)]
struct EqRI;
impl Operation for EqRI {
    fn execute(&self, oprands: &Oprands, regs: &mut RegisterGroup) -> Result<(), Error> {
        Ok(*regs.reg_mut(oprands[2])? = if *regs.reg(oprands[0])? == oprands[1] {
            1
        } else {
            0
        })
    }
}

#[derive(Clone)]
struct EqRR;
impl Operation for EqRR {
    fn execute(&self, oprands: &Oprands, regs: &mut RegisterGroup) -> Result<(), Error> {
        Ok(*regs.reg_mut(oprands[2])? = if regs.reg(oprands[0])? == regs.reg(oprands[1])? {
            1
        } else {
            0
        })
    }
}

pub struct Executor {
    regs: RegisterGroup,
}

impl Executor {
    pub fn new() -> Self {
        Executor { regs: RegisterGroup::new() }
    }

    pub fn with_regs(regs: &RegisterGroup) -> Self {
        Executor { regs: *regs }
    }

    pub fn regs_mut(&mut self) -> &mut RegisterGroup {
        &mut self.regs
    }

    pub fn regs(&self) -> &RegisterGroup {
        &self.regs
    }

    pub fn execute(&mut self, op: &dyn Operation, oprands: &Oprands) -> Result<(), Error> {
        op.execute(oprands, &mut self.regs)
    }

    pub fn execute_with_map(&mut self, inst: &Instruction, op_code_map: &HashMap<usize, usize>) -> Result<(), Error> {
        let op_ind = op_code_map.get(&inst.op_code).ok_or(Error::OpCodeNotfound(inst.op_code))?;
        OPERATIONS[*op_ind].execute(&inst.oprands, self.regs_mut())
    }
}

static OPERATIONS: Lazy<Vec<Box<dyn Operation>>> = Lazy::new(|| vec![Box::new(AddR), Box::new(AddI), Box::new(MulR), Box::new(MulI),
        Box::new(BanR), Box::new(BanI), Box::new(BorR), Box::new(BorI), Box::new(SetR), Box::new(SetI), Box::new(GtIR),
        Box::new(GtRI), Box::new(GtRR), Box::new(EqIR), Box::new(EqRI), Box::new(EqRR)]);

pub fn guess_insts(sample: &ExeSample) -> HashSet<usize> {
    let mut executor = Executor::new();
    let mut possibilities = HashSet::new();
    for (ind, op) in OPERATIONS.iter().enumerate() {
        *executor.regs_mut() = sample.before;
        if executor.execute(op.as_ref(), &sample.inst.oprands).is_err() {
            continue;
        }

        if *executor.regs() == sample.after {
            possibilities.insert(ind);
        }
    }

    possibilities
}