pub mod map;
pub mod sim;

use std::{io, fmt::Display};

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    BlockRectParseFormat(String),
    BlockRangeParseError(String),
    WaterFlowThroughClay(Position),
    WaterBlockedByGhost(Position),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IOError(ioe) => write!(f, "IOError: {}", ioe),
            Error::BlockRectParseFormat(s) => write!(f, "Failed to parse block rectangle from string({})", s),
            Error::BlockRangeParseError(s) => write!(f, "Failed to parse block range from string({})", s),
            Error::WaterFlowThroughClay(p) => write!(f, "Water can't pass through clay block, but passed at {}", p),
            Error::WaterBlockedByGhost(p) => write!(f, "Water is blocked at {}, but there isn't any clay block", p),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    r: usize,
    c: usize,
}

impl Position {
    pub fn new(r: usize, c: usize) -> Position {
        Position { r, c }
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "position({}, {})", self.r, self.c)
    }
}
