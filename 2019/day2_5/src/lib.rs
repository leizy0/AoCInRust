use std::fmt::Display;

pub mod int_code;

#[derive(Debug)]
pub enum Error {
    IOError(std::io::Error),
    EmptyError,
    ParseIntError(String),
    ImageIndexError(i32),
    InvalidWriteImageMode(u8),
    InvalidOpcode(i32),
    InvalidOpcodeIndex(u32),
    UnknownParameterMode(u32),
    OpcodeNotMatchingForInstruction(String, u32),
    InvalidModeChar(usize, String),
    MissingCodeForInstruction(u32),
    ExecutionExceedIntCode(usize, usize),
    NotEnoughInput,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IOError(ioe) => write!(f, "I/O Error: {}", ioe),
            Error::EmptyError => write!(f, "Get empty code in file"),
            Error::ParseIntError(s) => write!(f, "Failed to parse integer from string({})", s),
            Error::ImageIndexError(i) => write!(f, "Invalid index({}) found in execution", i),
            Error::InvalidWriteImageMode(m) => write!(f, "Invalid parameter mode({}) found when write into image", m),
            Error::InvalidOpcodeIndex(c) => write!(f, "Invalid operation code({}) found in execution", c),
            Error::UnknownParameterMode(i) => write!(f, "Unknown parameter mode({}) found", i),
            Error::InvalidOpcode(c) => write!(f, "Invalid Operation code({}) found", c),
            Error::OpcodeNotMatchingForInstruction(c, i) => write!(f, "Given operation code({}) isn't match to expected operation code index({})", c, i),
            Error::InvalidModeChar(ref p, c) => write!(f, "Found invalid parameter character({}) in operation code({})", c.chars().nth(*p).unwrap_or(' '), c),
            Error::MissingCodeForInstruction(i) => write!(f, "Missing more code for instruction whose operation code index is {}", i),
            Error::ExecutionExceedIntCode(cur_inst_p, code_len) => write!(f, "Current instruction pointer({}) exceeds total code length({})", cur_inst_p, code_len),
            Error::NotEnoughInput => write!(f, "Not enough input in execution, inputs exhausted"),
        }
    }
}
