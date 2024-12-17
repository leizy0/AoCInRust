use std::{
    error,
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use clap::Parser;
use int_enum::IntEnum;

#[derive(Debug)]
pub enum Error {
    NoRegisterALine,
    NoRegisterBLine,
    NoRegisterCLine,
    NoProgram,
    InvalidRegisterText(String),
    InvalidRegisterValue(String),
    InvalidProgramText(String),
    InvalidCode(String),
    InvalidOpcode(usize),
    InvalidOperand(usize),
    InvalidComboOperand(usize),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NoRegisterALine => {
                write!(f, "Can't find line for register A in debug information.")
            }
            Error::NoRegisterBLine => {
                write!(f, "Can't find line for register B in debug information.")
            }
            Error::NoRegisterCLine => {
                write!(f, "Can't find line for register C in debug information.")
            }
            Error::NoProgram => write!(f, "Can't find line for program in debug information."),
            Error::InvalidRegisterText(s) => write!(f, "Invalid text({}) for register.", s),
            Error::InvalidRegisterValue(s) => write!(f, "Invalid text({}) for register value.", s),
            Error::InvalidProgramText(s) => write!(f, "Invalid text({}) for program.", s),
            Error::InvalidCode(s) => write!(f, "Invalid text({}) for code.", s),
            Error::InvalidOpcode(n) => write!(f, "Invalid operation code({}).", n),
            Error::InvalidOperand(n) => write!(f, "Invalid operation number({}).", n),
            Error::InvalidComboOperand(n) => write!(f, "Invalid combo operation number({}).", n),
        }
    }
}

impl error::Error for Error {}

#[derive(Debug, Parser)]
pub struct CLIArgs {
    pub input_path: PathBuf,
}

trait ExecutionContext {
    fn reg(&self, name: RegisterName) -> &usize;
    fn reg_mut(&mut self, name: RegisterName) -> &mut usize;
    fn inst_ptr_mut(&mut self) -> &mut usize;
    fn output(&mut self, n: usize);
}

#[derive(Debug)]
struct ProgramContext<'a> {
    computer: &'a mut Computer,
    inst_ptr: &'a mut usize,
}

impl<'a> ExecutionContext for ProgramContext<'a> {
    fn reg(&self, name: RegisterName) -> &usize {
        &self.computer.registers[usize::from(name)]
    }

    fn reg_mut(&mut self, name: RegisterName) -> &mut usize {
        &mut self.computer.registers[usize::from(name)]
    }

    fn inst_ptr_mut(&mut self) -> &mut usize {
        &mut self.inst_ptr
    }

    fn output(&mut self, n: usize) {
        self.computer.output.push(n);
    }
}

impl<'a> ProgramContext<'a> {
    pub fn new(computer: &'a mut Computer, inst_ptr: &'a mut usize) -> Self {
        Self { computer, inst_ptr }
    }
}

#[derive(Debug, Clone, Copy, IntEnum)]
#[repr(usize)]
enum Instruction {
    Adv = 0,
    Bxl = 1,
    Bst = 2,
    Jnz = 3,
    Bxc = 4,
    Out = 5,
    Bdv = 6,
    Cdv = 7,
}

impl Instruction {
    pub fn exec_in(&self, operand: usize, mut context: impl ExecutionContext) -> Result<(), Error> {
        if operand > 7 {
            return Err(Error::InvalidOperand(operand));
        }

        match self {
            Instruction::Adv => {
                *context.reg_mut(RegisterName::A) /=
                    2usize.pow(u32::try_from(Self::combo_operand(operand, &context)?).unwrap())
            }
            Instruction::Bxl => *context.reg_mut(RegisterName::B) ^= operand,
            Instruction::Bst => {
                *context.reg_mut(RegisterName::B) = Self::combo_operand(operand, &context)? % 8
            }
            Instruction::Jnz => {
                if *context.reg(RegisterName::A) != 0 {
                    *context.inst_ptr_mut() = operand;
                    return Ok(());
                }
            }
            Instruction::Bxc => *context.reg_mut(RegisterName::B) ^= *context.reg(RegisterName::C),
            Instruction::Out => context.output(Self::combo_operand(operand, &context)? % 8),
            Instruction::Bdv => {
                *context.reg_mut(RegisterName::B) = context.reg(RegisterName::A)
                    / 2usize.pow(u32::try_from(Self::combo_operand(operand, &context)?).unwrap())
            }
            Instruction::Cdv => {
                *context.reg_mut(RegisterName::C) = context.reg(RegisterName::A)
                    / 2usize.pow(u32::try_from(Self::combo_operand(operand, &context)?).unwrap())
            }
        }
        *context.inst_ptr_mut() += 2;

        Ok(())
    }

    fn combo_operand(operand: usize, context: &impl ExecutionContext) -> Result<usize, Error> {
        match operand {
            0 | 1 | 2 | 3 => Ok(operand),
            4 => Ok(*context.reg(RegisterName::A)),
            5 => Ok(*context.reg(RegisterName::B)),
            6 => Ok(*context.reg(RegisterName::C)),
            other => Err(Error::InvalidComboOperand(other)),
        }
    }
}

#[derive(Debug, Clone, Copy, IntEnum)]
#[repr(usize)]
pub enum RegisterName {
    A = 0,
    B = 1,
    C = 2,
}

#[derive(Debug)]
pub struct Computer {
    registers: [usize; 3],
    output: Vec<usize>,
}

impl Computer {
    pub fn new(registers: &[usize; 3]) -> Self {
        Self {
            registers: registers.clone(),
            output: Vec::new(),
        }
    }

    pub fn run(&mut self, program: &[usize]) -> Result<(), Error> {
        let mut inst_ptr = 0;
        while inst_ptr < program.len() {
            let inst = Instruction::try_from(program[inst_ptr])
                .map_err(|code| Error::InvalidOpcode(code))?;
            let Some(operand) = program.get(inst_ptr + 1).copied() else {
                break;
            };

            inst.exec_in(operand, ProgramContext::new(self, &mut inst_ptr))?;
        }

        Ok(())
    }

    pub fn output(&self) -> &[usize] {
        &self.output
    }
}

pub fn read_debug_info<P: AsRef<Path>>(path: P) -> Result<([usize; 3], Vec<usize>)> {
    let file = File::open(&path)
        .with_context(|| format!("Failed to open given file({}).", path.as_ref().display()))?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    let mut registers = [0usize; 3];

    registers[0] = read_register(
        "A",
        lines
            .next()
            .ok_or(Error::NoRegisterALine)?
            .with_context(|| {
                format!(
                    "Failed to read line 1 in given file({}).",
                    path.as_ref().display()
                )
            })?
            .as_str(),
    )?;
    registers[1] = read_register(
        "B",
        lines
            .next()
            .ok_or(Error::NoRegisterBLine)?
            .with_context(|| {
                format!(
                    "Failed to read line 2 in given file({}).",
                    path.as_ref().display()
                )
            })?
            .as_str(),
    )?;
    registers[2] = read_register(
        "C",
        lines
            .next()
            .ok_or(Error::NoRegisterCLine)?
            .with_context(|| {
                format!(
                    "Failed to read line 3 in given file({}).",
                    path.as_ref().display()
                )
            })?
            .as_str(),
    )?;
    let mut line_ind = 4;
    while let Some(line) = lines.next() {
        let line = line.with_context(|| {
            format!(
                "Failed to read line {} in given file({}).",
                line_ind,
                path.as_ref().display()
            )
        })?;
        line_ind += 1;
        if !line.is_empty() {
            return Ok((registers, read_program(line.as_str())?));
        }
    }

    Err(Error::NoProgram.into())
}

fn read_register(name: &str, text: &str) -> Result<usize, Error> {
    let header = format!("Register {}:", name);
    let start_ind = text
        .find(&header)
        .ok_or(Error::InvalidRegisterText(text.to_string()))?;
    let value_text = text[(start_ind + header.len())..].trim();
    value_text
        .parse::<usize>()
        .map_err(|_| Error::InvalidRegisterValue(value_text.to_string()))
}

fn read_program(text: &str) -> Result<Vec<usize>, Error> {
    static HEADER: &'static str = "Program:";
    let start_ind = text
        .find(&HEADER)
        .ok_or(Error::InvalidProgramText(text.to_string()))?;
    let code_text = text[(start_ind + HEADER.len())..].trim();
    code_text
        .split(',')
        .map(|s| {
            s.parse::<usize>()
                .map_err(|_| Error::InvalidCode(s.to_string()))
        })
        .collect()
}
