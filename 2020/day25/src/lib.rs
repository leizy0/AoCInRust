use std::{
    error,
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use clap::Parser;

#[derive(Debug)]
pub enum Error {
    NoCardPubKey,
    NoDoorPubKey,
    InvalidPubKey(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NoCardPubKey => write!(
                f,
                "Can't find the public key of the card, expect it appears at the first line"
            ),
            Error::NoDoorPubKey => write!(
                f,
                "Can't find the public key of the door, expect it appears at the second line"
            ),
            Error::InvalidPubKey(s) => write!(
                f,
                "Invalid public key found: {}, expect an non-negative number",
                s
            ),
        }
    }
}

impl error::Error for Error {}

#[derive(Debug, Parser)]
pub struct CLIArgs {
    pub input_path: PathBuf,
}

pub fn read_pub_keys<P: AsRef<Path>>(path: P) -> Result<(usize, usize)> {
    let file = File::open(path.as_ref())
        .with_context(|| format!("Failed to open input file: {}.", path.as_ref().display()))?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    let card_pub_key = lines
        .next()
        .ok_or(Error::NoCardPubKey)?
        .with_context(|| {
            format!(
                "Failed to read the first line of input file: {}.",
                path.as_ref().display()
            )
        })
        .and_then(|s| {
            s.parse::<usize>()
                .map_err(|_| Error::InvalidPubKey(s.clone()).into())
        })?;
    let door_pub_key = lines
        .next()
        .ok_or(Error::NoDoorPubKey)?
        .with_context(|| {
            format!(
                "Failed to read the second line of input file: {}.",
                path.as_ref().display()
            )
        })
        .and_then(|s| {
            s.parse::<usize>()
                .map_err(|_| Error::InvalidPubKey(s.clone()).into())
        })?;

    Ok((card_pub_key, door_pub_key))
}

pub fn find_loop_size(pub_key: usize, subject_n: usize, divisor: usize) -> usize {
    let mut loop_size = 0;
    let mut cur_pub_key = 1;
    while cur_pub_key != pub_key {
        cur_pub_key *= subject_n;
        cur_pub_key %= divisor;
        loop_size += 1;
    }

    loop_size
}

pub fn transform_subject_n(subject_n: usize, divisor: usize, loop_size: usize) -> usize {
    let mut encryp_key = 1;
    for _ in 0..loop_size {
        encryp_key *= subject_n;
        encryp_key %= divisor;
    }

    encryp_key
}
