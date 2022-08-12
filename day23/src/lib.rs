use std::io;

pub mod bot;

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    NotMatchNanobotPattern(String),
}
