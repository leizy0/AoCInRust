use std::{path::Path, fs::File, io::{BufReader, BufRead}, ops::Index, fmt::Display};

use once_cell::sync::Lazy;
use regex::Regex;

use crate::Error;

#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub r: usize,
    pub c: usize,
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.r, self.c)
    }
}

impl Position {
    pub fn new(r: usize, c: usize) -> Position {
        Position { r, c }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MapSetting {
    pub depth: usize,
    pub target: Position,
}

impl TryFrom<&[String]> for MapSetting {
    type Error = Error;

    fn try_from(value: &[String]) -> Result<Self, Self::Error> {
        static DEPTH_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"depth:\s+(\d+)").unwrap());
        static TARGET_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"target:\s+(\d+),\s*(\d+)").unwrap());

        let mut depth = None;
        let mut target = None;
        for s in value {
            if let Some(caps) = DEPTH_PATTERN.captures(s) {
                depth = caps[1].parse::<usize>().ok();
            }

            if let Some(caps) = TARGET_PATTERN.captures(s) {
                target = caps[1].parse::<usize>().and_then(|c| caps[2].parse::<usize>().map(|r| Position::new(r, c))).ok();
            }
        }

        Ok(MapSetting{depth: depth.ok_or(Error::NoDepthInInput)?, target: target.ok_or(Error::NoTargetInInput)?})
    }
}

pub fn load_setting<P>(input_path: P) -> Result<MapSetting, Error> where P: AsRef<Path> {
    let input_file = File::open(input_path).map_err(Error::IOError)?;
    let reader = BufReader::new(input_file);
    let lines = reader.lines()
        .map(|lr| lr.map_err(Error::IOError))
        .collect::<Result<Vec<_>, Error>>()?;
    
    MapSetting::try_from(&lines[..])
}

pub enum CaveBlock {
    Rocky,
    Wet,
    Narrow,
}

impl CaveBlock {
    pub fn risk(&self) -> usize {
        match self {
            CaveBlock::Rocky => 0,
            CaveBlock::Wet => 1,
            CaveBlock::Narrow => 2,
        }
    }
}

pub struct CaveMap {
    blocks: Vec<CaveBlock>,
    erosion_levels: Vec<usize>,
    col_count: usize,
    row_count: usize,
}

impl Index<Position> for CaveMap {
    type Output = CaveBlock;

    fn index(&self, index: Position) -> &Self::Output {
        let ind = index.r * self.col_count + index.c;
        &self.blocks[ind]
    }
}

impl CaveMap {
    pub fn new(setting: &MapSetting) -> Self {
        let row_count = setting.target.r + 1;
        let col_count = setting.target.c + 1;
        let mut blocks = Vec::with_capacity(row_count * col_count);
        let mut erosion_levels = Vec::with_capacity(row_count * col_count);
        for r in 0..row_count {
            for c in 0..col_count {
                let geo_ind = if (r == 0 && c == 0) ||
                    (r == setting.target.r && c == setting.target.c)  {
                    0
                } else if r == 0 {
                    c * 16807
                } else if c == 0 {
                    r * 48271
                } else {
                    let up_ind = (r - 1) * col_count + c;
                    let left_ind = r * col_count + c - 1;
                    erosion_levels[up_ind] * erosion_levels[left_ind]
                };
    
                let ero_level = (geo_ind + setting.depth) % 20183;
                erosion_levels.push(ero_level);
                blocks.push(match ero_level % 3 {
                    0 => CaveBlock::Rocky,
                    1 => CaveBlock::Wet,
                    2 => CaveBlock::Narrow,
                    _ => panic!("Never panic"),
                })
            }
        }
    
        CaveMap { blocks, erosion_levels, col_count, row_count }
    }

    pub fn log(&self) {
        for r in 0..self.row_count {
            for c in 0..self.col_count {
                match self[Position::new(r, c)] {
                    CaveBlock::Rocky => print!("."),
                    CaveBlock::Wet => print!("="),
                    CaveBlock::Narrow => print!("|"),
                }
            }
            println!();
        }
    }
}
