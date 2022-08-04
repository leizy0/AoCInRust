use std::{io::{self, BufReader, BufRead}, fs::File, collections::{HashSet, HashMap}, fmt::Display, path::Path};

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
    InvalidInstructionPointer(usize),
    IPMapParseError(String),
    OperationCodeParseError(String),
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
            Error::InvalidInstructionPointer(ip) => write!(f, "Invalid instruction pointer({})", ip),
            Error::IPMapParseError(s) => write!(f, "Failed to parse ip map declaration from text({})", s),
            Error::OperationCodeParseError(s) => write!(f, "Failed to parse operation code from text({})", s),
        }
    }
}

const GENERAL_REGISTER_COUNT: usize = 6;
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct RegisterGroup {
    regs: [usize; GENERAL_REGISTER_COUNT],
    ip: usize,
    ip_map_ind: Option<usize>,
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
                0,
                0,
            ],
            ip: 0,
            ip_map_ind: None,
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
        RegisterGroup { regs: [0; 6], ip: 0, ip_map_ind: None }
    }

    pub fn from_arr(arr: &[usize]) -> Self {
        Self::from_arr_ip(arr, 0)
    }

    pub fn from_arr_ip(arr: &[usize], ip: usize) -> Self {
        let mut group = Self::new();
        for i in 0..arr.len().min(GENERAL_REGISTER_COUNT) {
            group.regs[i] = arr[i];
        }
        group.ip = ip;

        group
    }

    pub fn reg(&self, ind: usize) -> Result<&usize, Error> {
        if self.is_ip_ind(ind) {
            Ok(&self.ip)
        } else if ind < self.regs.len() {
            Ok(&self.regs[ind])
        } else {
            Err(Error::InvalidRegisterIndex(ind))
        }
    }

    pub fn reg_mut(&mut self, ind: usize) -> Result<&mut usize, Error> {
        if self.is_ip_ind(ind) {
            Ok(self.ip_mut())
        } else if ind < self.regs.len() {
            Ok(&mut self.regs[ind])
        } else {
            Err(Error::InvalidRegisterIndex(ind))
        }
    }

    pub fn ip(&self) -> usize {
        self.ip
    }

    pub fn ip_mut(&mut self) -> &mut usize {
        &mut self.ip
    }

    pub fn map_ip(&mut self, ind: usize) {
        self.ip_map_ind = Some(ind);
    }

    pub fn unmap_ip(&mut self) {
        self.ip_map_ind = None;
    }

    fn is_ip_ind(&self, ind: usize) -> bool {
        if let Some(ip_map_ind) = self.ip_map_ind {
            ip_map_ind == ind
        } else {
            false
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
        static INST_NUMBER_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\d+) (\d+) (\d+) (\d+)").unwrap());
        static INST_NAME_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\w+) (\d+) (\d+) (\d+)").unwrap());
        const OP_CODE_COUNT: usize = 16;
        const OP_CODE_MAX: usize = OP_CODE_COUNT - 1;

        let caps = INST_NUMBER_PATTERN.captures(value).or_else(|| INST_NAME_PATTERN.captures(value))
            .ok_or(Error::InstructionFormatError(value.to_string()))?;
        let op_code = if let Ok(op_code) = caps[1].parse::<usize>() {
            if op_code > OP_CODE_MAX {
                return Err(Error::InvalidOpCode(op_code));
            }
            op_code
        } else if let Some(op_code) = OP_NAME_MAP.get(&caps[1]) {
            *op_code
        } else {
            return Err(Error::OperationCodeParseError(caps[1].to_string()));
        };
        
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

impl Instruction {
    pub fn new(op_code: usize, oprands: Oprands) -> Self {
        Instruction { op_code, oprands }
    }

    pub fn op_code(&self) -> usize {
        self.op_code
    }

    pub fn oprands(&self) -> &Oprands {
        &self.oprands
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

pub fn load_samples<P>(input_path: P) -> Result<Vec<ExeSample>, Error> where P: AsRef<Path> {
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

pub fn load_insts<P>(input_path: P) -> Result<Vec<Instruction>, Error> where P: AsRef<Path> {
    let input_file = File::open(input_path).map_err(Error::IOError)?;
    let reader = BufReader::new(input_file);
    reader.lines()
        .filter(|s| s.is_err() || !s.as_ref().unwrap().is_empty())
        .map(|se| se.map_err(Error::IOError)
            .and_then(|s| Instruction::try_from(s.as_str())))
        .collect::<Result<Vec<_>, _>>()
}

static OP_NAME_MAP: Lazy<HashMap<&'static str, usize>> = Lazy::new(|| HashMap::from([
    ("addr", 0),
    ("addi", 1),
    ("mulr", 2),
    ("muli", 3),
    ("banr", 4),
    ("bani", 5),
    ("borr", 6),
    ("bori", 7),
    ("setr", 8),
    ("seti", 9),
    ("gtir", 10),
    ("gtri", 11),
    ("gtrr", 12),
    ("eqir", 13),
    ("eqri", 14),
    ("eqrr", 15),
]));

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

    pub fn execute(&mut self, program: &Program) -> Result<(), Error> {
        for decl in &program.decls {
            decl.apply(self)?;
        }

        let mut tick_ind: usize = 0;
        loop {
            println!("Tick #{}: general registers: {}, ip: {}", tick_ind, self.regs(), self.regs().ip());
            let inst = program.insts.get(self.regs.ip()).ok_or(Error::InvalidInstructionPointer(self.regs.ip()))?;
            self.execute_inst(inst)?;
            *self.regs.ip_mut() += 1;
            tick_ind += 1;
        }
    }

    pub fn execute_op(&mut self, op: &dyn Operation, oprands: &Oprands) -> Result<(), Error> {
        op.execute(oprands, &mut self.regs)
    }

    pub fn execute_inst(&mut self, inst: &Instruction) -> Result<(), Error> {
        OPERATIONS[inst.op_code].execute(&inst.oprands, self.regs_mut())
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
        if executor.execute_op(op.as_ref(), &sample.inst.oprands).is_err() {
            continue;
        }

        if *executor.regs() == sample.after {
            possibilities.insert(ind);
        }
    }

    possibilities
}

trait Declaration {
    fn apply(&self, executor: &mut Executor) -> Result<(), Error>;
}

struct IPMap {
    reg_ind: usize,
}

impl Declaration for IPMap {
    fn apply(&self, executor: &mut Executor) -> Result<(), Error> {
        executor.regs_mut().map_ip(self.reg_ind);
        Ok(())
    }
}

impl TryFrom<&str> for IPMap {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        static IP_MAP_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"#ip (\d+)").unwrap());
        let reg_ind_text = &IP_MAP_PATTERN.captures(value).ok_or(Error::IPMapParseError(value.to_string()))?[1];
        Ok(IPMap{reg_ind: reg_ind_text.parse::<usize>().unwrap()})
    }
}

enum Statement {
    Declaration(Box<dyn Declaration>),
    Istruction(Instruction),
}

impl TryFrom<&str> for Statement {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        IPMap::try_from(value).map(|i| Statement::Declaration(Box::new(i)))
            .or_else(|_| Instruction::try_from(value).map(|i| Statement::Istruction(i)))
    }
}

pub struct Program {
    decls: Vec<Box<dyn Declaration>>,
    insts: Vec<Instruction>,
}

struct ProgramBuilder {
    decls: Vec<Box<dyn Declaration>>,
    insts: Vec<Instruction>,
}

impl ProgramBuilder {
    pub fn new() -> ProgramBuilder {
        ProgramBuilder { decls: Vec::new(), insts: Vec::new() }
    }

    pub fn add_stm(&mut self, text: &str) -> Result<(), Error> {
        match Statement::try_from(text)? {
            Statement::Declaration(d) => self.decls.push(d),
            Statement::Istruction(inst) => self.insts.push(inst),
        }

        Ok(())
    }

    pub fn build(self) -> Program {
        Program { decls: self.decls, insts: self.insts }
    }
}

pub fn load_program<P>(input_path: P) -> Result<Program, Error> where P: AsRef<Path> {
    let input_file = File::open(input_path).map_err(Error::IOError)?;
    let reader = BufReader::new(input_file);
    let lines = reader.lines()
        .map(|lr| lr.map_err(Error::IOError))
        .collect::<Result<Vec<_>, _>>()?;
    
    let mut builder = ProgramBuilder::new();
    for line in lines {
        builder.add_stm(line.as_str())?;
    }

    Ok(builder.build())
}