use std::io;

pub mod map;

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    NoDepthInInput,
    NoTargetInInput,
}