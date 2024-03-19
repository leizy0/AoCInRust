pub mod com;
mod inst;

use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use crate::Error;

pub fn read_int_code<P>(path: P) -> Result<Vec<i64>, Error>
where
    P: AsRef<Path>,
{
    let code_file = File::open(path).map_err(Error::IOError)?;
    let reader = BufReader::new(code_file);

    reader.lines().next().map_or(Err(Error::EmptyError), |res| {
        res.map_err(Error::IOError).and_then(|s| {
            s.split(',')
                .map(|s| str::parse::<i64>(s).map_err(|_| Error::ParseIntError(s.to_string())))
                .collect::<Result<Vec<_>, Error>>()
        })
    })
}
