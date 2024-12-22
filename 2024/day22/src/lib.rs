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
    InvalidSecretNumberText(String),
    InvalidSliceForChangeSeq(usize, usize),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidSecretNumberText(s) => {
                write!(f, "Invalid text({}) for secret number.", s)
            }
            Error::InvalidSliceForChangeSeq(expect_len, this_len) => write!(
                f,
                "Invalid slice for change sequence, expect {} numbers in sequence, given {}.",
                expect_len, this_len
            ),
        }
    }
}

impl error::Error for Error {}

#[derive(Debug, Parser)]
pub struct CLIArgs {
    pub input_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChangeSeq<const N: usize> {
    seq: [isize; N],
}

impl<const N: usize> TryFrom<&[isize]> for ChangeSeq<N> {
    type Error = Error;

    fn try_from(value: &[isize]) -> std::result::Result<Self, Self::Error> {
        if value.len() != N {
            Err(Error::InvalidSliceForChangeSeq(N, value.len()))
        } else {
            let mut seq = [0; N];
            for ind in 0..N {
                seq[ind] = value[ind];
            }

            Ok(Self { seq })
        }
    }
}

impl<const N: usize> Display for ChangeSeq<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.seq)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SecretNumber {
    n: usize,
}

impl TryFrom<&str> for SecretNumber {
    type Error = Error;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            n: value
                .parse::<usize>()
                .map_err(|_| Error::InvalidSecretNumberText(value.to_string()))?,
        })
    }
}

impl Iterator for SecretNumber {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.generate())
    }
}

impl SecretNumber {
    pub fn new(n: usize) -> Self {
        Self { n }
    }

    pub fn n(&self) -> usize {
        self.n
    }

    pub fn generate(&mut self) -> usize {
        self.mix(self.n * 64);
        self.prune();

        self.mix(self.n / 32);
        self.prune();

        self.mix(self.n * 2048);
        self.prune();

        self.n
    }

    fn mix(&mut self, value: usize) {
        self.n ^= value;
    }

    fn prune(&mut self) {
        self.n %= 16777216;
    }
}

#[test]
fn test_generate() {
    let sec_n = SecretNumber::new(123);
    let target_seq = vec![
        15887950, 16495136, 527345, 704524, 1553684, 12683156, 11100544, 12249484, 7753432, 5908254,
    ];
    let generate_seq = sec_n.take(target_seq.len()).collect::<Vec<_>>();
    assert!(target_seq == generate_seq);
}

#[test]
fn test_mix() {
    let mut sec_n = SecretNumber::new(42);
    sec_n.mix(15);
    assert!(sec_n.n == 37);
}

#[test]
fn test_prune() {
    let mut sec_n = SecretNumber::new(100000000);
    sec_n.prune();
    assert!(sec_n.n == 16113920);
}

pub fn read_init_numbers<P: AsRef<Path>>(path: P) -> Result<Vec<SecretNumber>> {
    let file = File::open(&path)
        .with_context(|| format!("Failed to open given file({}).", path.as_ref().display()))?;
    let reader = BufReader::new(file);
    reader
        .lines()
        .enumerate()
        .map(|(ind, line)| {
            line.with_context(|| {
                format!(
                    "Failed to read line {} in given file({}).",
                    ind + 1,
                    path.as_ref().display()
                )
            })
            .and_then(|s| {
                SecretNumber::try_from(s.as_str()).with_context(|| {
                    format!("Failed to parse secret number from given line({}).", s)
                })
            })
        })
        .collect()
}
