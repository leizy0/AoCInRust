use std::{fmt::Display, io};

use play::Player;

pub mod map;
pub mod play;

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    NoDepthInInput,
    NoTargetInInput,
    UnreachableTarget(Player),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash)]
pub struct Position {
    pub r: usize,
    pub c: usize,
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.r, self.c)
    }
}

impl Position {
    pub fn new(r: usize, c: usize) -> Position {
        Position { r, c }
    }

    pub fn up(&self) -> Option<Position> {
        self.r.checked_sub(1).map(|r| Position::new(r, self.c))
    }

    pub fn down(&self) -> Position {
        Position {
            r: self.r + 1,
            c: self.c,
        }
    }

    pub fn left(&self) -> Option<Position> {
        self.c.checked_sub(1).map(|c| Position::new(self.r, c))
    }

    pub fn right(&self) -> Position {
        Position {
            r: self.r,
            c: self.c + 1,
        }
    }
}
