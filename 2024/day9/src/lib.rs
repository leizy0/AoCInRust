use std::{
    error,
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader},
    ops::Range,
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
        let mut file_id = 0;
        let mut is_free = false;
        let mut block_ind = 0;
        for c in value.chars() {
            let cur_block_n = usize::try_from(c.to_digit(10).unwrap()).unwrap();
            let block = if is_free {
                None
            } else {
                let cur_file_id = file_id;
                file_id += 1;
                Some(cur_file_id)
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
    pub fn compact_per_block(&mut self) {
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

    pub fn compact_per_file(&mut self) {
        let blocks_n = self.blocks.len();
        let mut search_ind = blocks_n - 1;
        while let Some(move_file_range) = self.back_file_range(search_ind) {
            if move_file_range.start == 0 {
                break;
            }

            search_ind = move_file_range.start - 1;
            if let Some(first_enough_free_range) =
                self.first_enough_free_range(move_file_range.len(), move_file_range.start)
            {
                for (move_ind, free_ind) in move_file_range.zip(first_enough_free_range) {
                    self.blocks.swap(move_ind, free_ind);
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

    fn back_file_range(&self, back_start_ind: usize) -> Option<Range<usize>> {
        if let Some((file_back_ind, file_id)) = (0..=back_start_ind)
            .rev()
            .filter_map(|block_ind| {
                self.blocks
                    .get(block_ind)
                    .and_then(|block| block.as_ref())
                    .map(|file_id| (block_ind, *file_id))
            })
            .next()
        {
            let file_start_ind = (0..file_back_ind)
                .rev()
                .filter(|ind| {
                    self.blocks
                        .get(*ind)
                        .and_then(|file_id_op| file_id_op.map(|cur_file_id| cur_file_id != file_id))
                        .unwrap_or(true)
                })
                .next()
                .map(|ind| ind + 1)
                .unwrap_or(0);

            Some(file_start_ind..(file_back_ind + 1))
        } else {
            None
        }
    }

    fn first_enough_free_range(
        &self,
        free_blocks_n: usize,
        search_end_ind: usize,
    ) -> Option<Range<usize>> {
        let mut search_ind = 0;
        let search_end_ind = search_end_ind.min(self.blocks.len());
        while search_ind < search_end_ind {
            if self.blocks[search_ind].is_none() {
                let free_end_ind = (search_ind..search_end_ind)
                    .filter(|ind| {
                        self.blocks
                            .get(*ind)
                            .map(|file_id_op| file_id_op.is_some())
                            .unwrap_or(true)
                    })
                    .next()
                    .unwrap_or(search_end_ind);
                if free_end_ind - search_ind >= free_blocks_n {
                    return Some(search_ind..free_end_ind);
                } else {
                    search_ind = free_end_ind + 1;
                    continue;
                }
            }

            search_ind += 1;
        }

        None
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
