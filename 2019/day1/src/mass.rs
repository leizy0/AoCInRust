use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use crate::Error;

pub fn read_md_mass<P>(file_path: P) -> Result<Vec<u32>, Error>
where
    P: AsRef<Path>,
{
    let file = File::open(file_path).map_err(Error::IOError)?;
    let reader = BufReader::new(file);
    reader
        .lines()
        .map(|l| {
            l.map_err(Error::IOError).and_then(|s| {
                s.parse::<u32>()
                    .map_err(|_e| Error::ParseIntError(s.to_string()))
            })
        })
        .collect::<Result<Vec<_>, Error>>()
}
