pub mod wire;

#[derive(Debug)]
pub enum Error {
    IOError(std::io::Error),
    ParsePathError(String),
    UnknownPathDirection(char),
}
