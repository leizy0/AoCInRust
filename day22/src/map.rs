use std::{
    cell::RefCell,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use once_cell::sync::Lazy;
use regex::Regex;

use crate::{Error, Position};

#[derive(Debug, Clone, Copy)]
pub struct MapSetting {
    pub depth: usize,
    pub target: Position,
}

impl TryFrom<&[String]> for MapSetting {
    type Error = Error;

    fn try_from(value: &[String]) -> Result<Self, Self::Error> {
        static DEPTH_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"depth:\s+(\d+)").unwrap());
        static TARGET_PATTERN: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"target:\s+(\d+),\s*(\d+)").unwrap());

        let mut depth = None;
        let mut target = None;
        for s in value {
            if let Some(caps) = DEPTH_PATTERN.captures(s) {
                depth = caps[1].parse::<usize>().ok();
            }

            if let Some(caps) = TARGET_PATTERN.captures(s) {
                target = caps[1]
                    .parse::<usize>()
                    .and_then(|c| caps[2].parse::<usize>().map(|r| Position::new(r, c)))
                    .ok();
            }
        }

        Ok(MapSetting {
            depth: depth.ok_or(Error::NoDepthInInput)?,
            target: target.ok_or(Error::NoTargetInInput)?,
        })
    }
}

pub fn load_setting<P>(input_path: P) -> Result<MapSetting, Error>
where
    P: AsRef<Path>,
{
    let input_file = File::open(input_path).map_err(Error::IOError)?;
    let reader = BufReader::new(input_file);
    let lines = reader
        .lines()
        .map(|lr| lr.map_err(Error::IOError))
        .collect::<Result<Vec<_>, Error>>()?;

    MapSetting::try_from(&lines[..])
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    setting: MapSetting,
    blocks: RefCell<Vec<Vec<CaveBlock>>>,
    erosion_levels: RefCell<Vec<Vec<usize>>>,
}

impl CaveMap {
    pub fn new(setting: &MapSetting) -> CaveMap {
        CaveMap {
            setting: *setting,
            blocks: RefCell::new(Vec::new()),
            erosion_levels: RefCell::new(Vec::new()),
        }
    }

    pub fn at(&self, pos: &Position) -> CaveBlock {
        if pos.r >= self.row_count() || pos.c >= self.col_count(pos.r) {
            self.extend_to(pos);
        }

        self.blocks.borrow()[pos.r][pos.c]
    }

    fn row_count(&self) -> usize {
        self.blocks.borrow().len()
    }

    fn col_count(&self, r: usize) -> usize {
        if r >= self.row_count() {
            0
        } else {
            self.blocks.borrow()[r].len()
        }
    }

    fn extend_to(&self, pos: &Position) {
        let mut beg_r = pos.r;
        while self.col_count(beg_r) <= pos.c && beg_r > 0 {
            beg_r -= 1;
        }
        if beg_r > 0 {
            beg_r += 1;
        }

        for r in beg_r..=pos.r {
            self.extend_row_to(r, pos.c);
        }
    }

    fn extend_row_to(&self, r: usize, c: usize) {
        if r == self.row_count() {
            self.blocks.borrow_mut().push(Vec::new());
            self.erosion_levels.borrow_mut().push(Vec::new());
        } else if r > self.row_count() {
            panic!("Extended row must exist or be the next row of map end.");
        }

        let beg_c = self.col_count(r);
        for c in beg_c..=c {
            let geo_ind = if (r == 0 && c == 0)
                || (r == self.setting.target.r && c == self.setting.target.c)
            {
                0
            } else if r == 0 {
                c * 16807
            } else if c == 0 {
                r * 48271
            } else {
                let pos = Position::new(r, c);
                let up_pos = pos.up().unwrap();
                let left_pos = pos.left().unwrap();
                let ero_levels = self.erosion_levels.borrow();
                ero_levels[up_pos.r][up_pos.c] * ero_levels[left_pos.r][left_pos.c]
            };

            let ero_level = (geo_ind + self.setting.depth) % 20183;
            self.erosion_levels.borrow_mut()[r].push(ero_level);
            self.blocks.borrow_mut()[r].push(match ero_level % 3 {
                0 => CaveBlock::Rocky,
                1 => CaveBlock::Wet,
                2 => CaveBlock::Narrow,
                _ => panic!("Never panic"),
            });
        }
    }

    pub fn log(&self) {
        for r in 0..self.row_count() {
            for c in 0..self.col_count(r) {
                match self.at(&Position::new(r, c)) {
                    CaveBlock::Rocky => print!("."),
                    CaveBlock::Wet => print!("="),
                    CaveBlock::Narrow => print!("|"),
                }
            }
            println!();
        }
    }
}
