use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use crate::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LumberBlock {
    Empty,
    Tree,
    Lumberyard,
}

impl LumberBlock {
    pub fn change<'a>(&self, neighbors: impl Iterator<Item = &'a Self>) -> (bool, LumberBlock) {
        let new_block = match self {
            LumberBlock::Empty => {
                if neighbors.filter(|n| **n == LumberBlock::Tree).count() >= 3 {
                    LumberBlock::Tree
                } else {
                    LumberBlock::Empty
                }
            }
            LumberBlock::Tree => {
                if neighbors.filter(|n| **n == LumberBlock::Lumberyard).count() >= 3 {
                    LumberBlock::Lumberyard
                } else {
                    LumberBlock::Tree
                }
            }
            LumberBlock::Lumberyard => {
                let mut tree_count = 0;
                let mut lumberyard_count = 0;
                for block in neighbors {
                    match block {
                        LumberBlock::Tree => tree_count += 1,
                        LumberBlock::Lumberyard => lumberyard_count += 1,
                        _ => (),
                    }
                }
                if tree_count >= 1 && lumberyard_count >= 1 {
                    LumberBlock::Lumberyard
                } else {
                    LumberBlock::Empty
                }
            }
        };

        (*self == new_block, new_block)
    }
}

#[derive(Clone)]
pub struct LumberMap {
    map: Vec<LumberBlock>,
    row_count: usize,
    col_count: usize,
}

impl LumberMap {
    pub fn len(&self) -> usize {
        assert!(self.map.len() == self.row_count * self.col_count);
        self.map.len()
    }

    pub fn row_major_at(&self, ind: usize) -> Result<&LumberBlock, Error> {
        self.map.get(ind).ok_or(Error::InvalidMapIndex(ind))
    }

    pub fn row_major_at_mut(&mut self, ind: usize) -> Result<&mut LumberBlock, Error> {
        self.map.get_mut(ind).ok_or(Error::InvalidMapIndex(ind))
    }

    pub fn neighbor_8<'a>(
        &'a self,
        ind: usize,
    ) -> Result<
        (
            impl Iterator<Item = usize>,
            impl Iterator<Item = &'a LumberBlock>,
        ),
        Error,
    > {
        let r: isize = (ind / self.col_count).try_into().unwrap();
        let c: isize = (ind % self.col_count).try_into().unwrap();
        if r >= self.row_count.try_into().unwrap() {
            return Err(Error::InvalidMapIndex(ind));
        }

        let ind_vec = [
            (r - 1, c - 1),
            (r - 1, c),
            (r - 1, c + 1),
            (r, c - 1),
            (r, c + 1),
            (r + 1, c - 1),
            (r + 1, c),
            (r + 1, c + 1),
        ]
        .iter()
        .filter(|(r, c)| {
            *r >= 0
                && *r < self.row_count.try_into().unwrap()
                && *c >= 0
                && *c < self.col_count.try_into().unwrap()
        })
        .map(|(r, c)| usize::try_from(*r).unwrap() * self.col_count + usize::try_from(*c).unwrap())
        .collect::<Vec<_>>();
        let ref_vec = ind_vec
            .iter()
            .map(|ind| &self.map[*ind])
            .collect::<Vec<_>>();

        Ok((ind_vec.into_iter(), ref_vec.into_iter()))
    }
}

struct LumberMapBuilder {
    map: Vec<LumberBlock>,
    col_count: usize,
}

impl LumberMapBuilder {
    pub fn new() -> LumberMapBuilder {
        LumberMapBuilder {
            map: Vec::new(),
            col_count: 0,
        }
    }

    pub fn add_row(&mut self, row_text: &str) -> Result<(), Error> {
        let mut this_col_count = 0;
        for c in row_text.chars() {
            self.map.push(match c {
                '.' => LumberBlock::Empty,
                '|' => LumberBlock::Tree,
                '#' => LumberBlock::Lumberyard,
                other => return Err(Error::InvalidInputChar(other)),
            });
            this_col_count += 1;
        }

        if self.col_count == 0 {
            self.col_count = this_col_count;
        } else if self.col_count != this_col_count {
            return Err(Error::InconsistentInputRowSize {
                old_size: self.col_count,
                new_size: this_col_count,
            });
        }

        Ok(())
    }

    pub fn build(self) -> LumberMap {
        let row_count = self.map.len() / self.col_count;
        LumberMap {
            map: self.map,
            row_count,
            col_count: self.col_count,
        }
    }
}

pub fn load_lumber_map<P>(input_path: P) -> Result<LumberMap, Error>
where
    P: AsRef<Path>,
{
    let input_file = File::open(input_path).map_err(Error::IOError)?;
    let reader = BufReader::new(input_file);
    let mut builder = LumberMapBuilder::new();
    let lines = reader
        .lines()
        .map(|l| l.map_err(Error::IOError))
        .collect::<Result<Vec<_>, Error>>()?;

    for line in lines {
        builder.add_row(line.as_str())?;
    }

    Ok(builder.build())
}
