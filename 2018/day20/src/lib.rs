use std::io;

use map::PathExpToken;

pub mod map;

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    EmptyInput(String),
    PathExpTokenParseError(char),
    InvalidCharInInput { ind: usize, c: char },
    InvalidDirectionChar(char),
    InvalidIndexToRoomDirection(usize),
    NoBeginInPathExp,
    NoEndInPathExp,
    RParenNotFoundInBranchEnd(usize),
    InvalidTokenInExpParsing(PathExpToken),
    AdjacentPathTokenFound,
    InvalidTokenInBranchParsing(PathExpToken),
    EmptyTokensLeftInBranchParsing,
    InvalidEndInBranchParsing,
    EmptyResultStackInExpParsing,
}
