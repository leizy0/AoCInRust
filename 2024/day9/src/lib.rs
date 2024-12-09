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
    InvalidChar(char),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidChar(c) => write!(f, "Invalid character({}) for file system text.", c),
        }
    }
}

impl error::Error for Error {}

#[derive(Debug, Parser)]
pub struct CLIArgs {
    pub input_path: PathBuf,
}

#[derive(Debug)]
pub struct FileSystem {
    blocks: Vec<Option<usize>>,
}

impl TryFrom<&str> for FileSystem {
    type Error = Error;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        let mut blocks_n = 0;
        for c in value.chars() {
            if let Some(d) = c.to_digit(10) {
                blocks_n += usize::try_from(d).unwrap();
            } else {
                return Err(Error::InvalidChar(c));
            }
        }

        let mut blocks = vec![None; blocks_n];
        let mut block_id = 0;
        let mut is_free = false;
        let mut block_ind = 0;
        for c in value.chars() {
            let cur_block_n = usize::try_from(c.to_digit(10).unwrap()).unwrap();
            let block = if is_free {
                None
            } else {
                let cur_block_id = block_id;
                block_id += 1;
                Some(cur_block_id)
            };
            for set_ind in block_ind..(block_ind + cur_block_n) {
                blocks[set_ind] = block;
            }

            block_ind += cur_block_n;
            is_free = !is_free;
        }

        Ok(Self { blocks })
    }
}

impl FileSystem {
    pub fn compact(&mut self) {
        let blocks_n = self.blocks.len();
        let mut free_ind = 0;
        for move_ind in (0..blocks_n).rev() {
            if self.blocks[move_ind].is_some() {
                if let Some(cur_free_ind) = (free_ind..blocks_n)
                    .filter(|ind| self.blocks[*ind].is_none())
                    .next()
                {
                    if cur_free_ind >= move_ind {
                        break;
                    }

                    self.blocks.swap(move_ind, cur_free_ind);
                    free_ind = cur_free_ind + 1;
                } else {
                    break;
                }
            }
        }
    }

    pub fn checksum(&self) -> usize {
        self.blocks
            .iter()
            .enumerate()
            .filter_map(|(ind, b)| b.as_ref().map(|id| ind * *id))
            .sum::<usize>()
    }
}

pub fn read_file_system<P: AsRef<Path>>(path: P) -> Result<FileSystem> {
    let file = File::open(&path)
        .with_context(|| format!("Failed to open given file({}).", path.as_ref().display()))?;
    let reader = BufReader::new(file);
    let file_system_text = reader
        .lines()
        .next()
        .unwrap_or(Ok(String::new()))
        .with_context(|| {
            format!(
                "Failed to read line 1 from given file({}).",
                path.as_ref().display()
            )
        })?;

    Ok(FileSystem::try_from(file_system_text.as_str())?)
}
