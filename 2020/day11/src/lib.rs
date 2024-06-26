use std::{
    error,
    fmt::Display,
    fs::File,
    io::{self, BufRead, BufReader},
    iter,
    path::{Path, PathBuf},
};

use clap::Parser;
use int_enum::IntEnum;
use once_cell::sync::Lazy;

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    InconsisitentSeatMapRow(usize, usize), // (element count of current row, expect count of elements in earlier row).
    InvalidSeatChar(char),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IOError(ioe) => write!(f, "I/O error: {}", ioe),
            Error::InconsisitentSeatMapRow(cur_count, expect_count) => write!(
                f,
                "Given row({} elements), expect row which have {} elements.",
                cur_count, expect_count
            ),
            Error::InvalidSeatChar(c) => write!(f, "Invalid character({}) for seat", c),
        }
    }
}

impl error::Error for Error {}

#[derive(Debug, Parser)]
pub struct CLIArgs {
    pub input_path: PathBuf,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntEnum)]
pub enum TileType {
    Floor = 0,
    Empty = 1,
    Occupied = 2,
}

impl TryFrom<char> for TileType {
    type Error = Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '.' => Ok(TileType::Floor),
            'L' => Ok(TileType::Empty),
            '#' => Ok(TileType::Occupied),
            other => Err(Error::InvalidSeatChar(other)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    TopLeft,
    Top,
    TopRight,
    Left,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

impl Direction {
    pub fn all() -> &'static [Direction] {
        static ALL_DIRS: Lazy<[Direction; 8]> = Lazy::new(|| {
            [
                Direction::TopLeft,
                Direction::Top,
                Direction::TopRight,
                Direction::Left,
                Direction::Right,
                Direction::BottomLeft,
                Direction::Bottom,
                Direction::BottomRight,
            ]
        });

        ALL_DIRS.as_ref()
    }
}

pub struct Position {
    r: usize,
    c: usize,
}

impl Position {
    pub fn new(r: usize, c: usize) -> Self {
        Self { r, c }
    }

    pub fn along(&self, dir: Direction) -> Option<Self> {
        match dir {
            Direction::TopLeft if self.r > 0 && self.c > 0 => {
                Some(Position::new(self.r - 1, self.c - 1))
            }
            Direction::Top if self.r > 0 => Some(Position::new(self.r - 1, self.c)),
            Direction::TopRight if self.r > 0 => Some(Position::new(self.r - 1, self.c + 1)),
            Direction::Left if self.c > 0 => Some(Position::new(self.r, self.c - 1)),
            Direction::Right => Some(Position::new(self.r, self.c + 1)),
            Direction::BottomLeft if self.c > 0 => Some(Position::new(self.r + 1, self.c - 1)),
            Direction::Bottom => Some(Position::new(self.r + 1, self.c)),
            Direction::BottomRight => Some(Position::new(self.r + 1, self.c + 1)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SeatMapBuffer {
    tiles: Vec<TileType>,
    row_n: usize,
    col_n: usize,
}

impl SeatMapBuffer {
    pub fn tile(&self, pos: &Position) -> Option<&TileType> {
        self.pos_to_ind(pos).map(|ind| &self.tiles[ind])
    }

    pub fn tile_mut(&mut self, pos: &Position) -> Option<&mut TileType> {
        self.pos_to_ind(pos).map(|ind| &mut self.tiles[ind])
    }

    pub fn count(&self, c_tt: TileType) -> usize {
        self.tiles.iter().filter(|tt| **tt == c_tt).count()
    }

    pub fn row_n(&self) -> usize {
        self.row_n
    }

    pub fn col_n(&self) -> usize {
        self.col_n
    }

    fn pos_to_ind(&self, pos: &Position) -> Option<usize> {
        if pos.r >= self.row_n || pos.c >= self.col_n {
            None
        } else {
            Some(pos.r * self.col_n + pos.c)
        }
    }
}

pub struct Environment {
    tt_counts: [usize; 3],
}

impl Environment {
    pub fn new(tt_counts: &[usize; 3]) -> Self {
        Self {
            tt_counts: *tt_counts,
        }
    }

    pub fn count(&self, tt: TileType) -> usize {
        self.tt_counts[u8::from(tt) as usize]
    }
}

pub trait EnvChecker {
    fn look_around(&self, pos: &Position, seats: &SeatMapBuffer) -> Environment;
}

pub trait ChangeRule {
    fn new_state(&self, old_state: TileType, env: &Environment) -> Option<TileType>;
}

pub struct Env8Neighbors;
impl EnvChecker for Env8Neighbors {
    fn look_around(&self, pos: &Position, seats: &SeatMapBuffer) -> Environment {
        let mut counts = [0usize; 3];
        for neighbor_p in Direction::all().iter().filter_map(|dir| pos.along(*dir)) {
            if let Some(tt) = seats.tile(&neighbor_p) {
                counts[u8::from(*tt) as usize] += 1;
            }
        }

        Environment::new(&counts)
    }
}

pub struct Env8Rays;
impl EnvChecker for Env8Rays {
    fn look_around(&self, pos: &Position, seats: &SeatMapBuffer) -> Environment {
        let mut counts = [0usize; 3];
        for (first_p, dir) in Direction::all()
            .iter()
            .filter_map(|dir| pos.along(*dir).map(|p| (p, *dir)))
        {
            let mut has_seat = false;
            for tt in iter::successors(Some(first_p), |p| p.along(dir))
                .map_while(|p| seats.tile(&p))
            {
                if *tt != TileType::Floor {
                    counts[u8::from(*tt) as usize] += 1;
                    has_seat = true;
                    break;
                }
            }

            if !has_seat {
                counts[u8::from(TileType::Floor) as usize] += 1;
            }
        }

        Environment::new(&counts)
    }
}

pub struct ChgForOccupiedN {
    occ_th: usize,
}

impl ChgForOccupiedN {
    pub fn new(occ_threshold: usize) -> Self {
        assert!(
            occ_threshold > 0,
            "Threshold for changing from empty to occupied must be larger than 0, given {}.",
            occ_threshold
        );
        Self {
            occ_th: occ_threshold,
        }
    }
}

impl ChangeRule for ChgForOccupiedN {
    fn new_state(&self, old_state: TileType, env: &Environment) -> Option<TileType> {
        let occ_count = env.count(TileType::Occupied);
        match old_state {
            TileType::Empty if occ_count == 0 => Some(TileType::Occupied),
            TileType::Occupied if occ_count >= self.occ_th => Some(TileType::Empty),
            _ => None,
        }
    }
}

pub struct SeatMap {
    seat_bufs: [SeatMapBuffer; 2],
    cur_buf_ind: usize,
}

impl SeatMap {
    pub fn step(&mut self, checker: &dyn EnvChecker, rule: &dyn ChangeRule) -> usize {
        let (r_buf, w_buf) = self.rw_buf();
        let mut chg_count = 0;
        for r in 0..r_buf.row_n() {
            for c in 0..r_buf.col_n() {
                let pos = Position::new(r, c);
                let env = checker.look_around(&pos, r_buf);
                let cur_tt = *r_buf.tile(&pos).unwrap();
                *w_buf.tile_mut(&pos).unwrap() = if let Some(new_tt) = rule.new_state(cur_tt, &env)
                {
                    chg_count += 1;
                    new_tt
                } else {
                    cur_tt
                };
            }
        }
        self.swap_buf();

        chg_count
    }

    pub fn count(&self, c_tt: TileType) -> usize {
        self.cur_buf().count(c_tt)
    }

    fn rw_buf(&mut self) -> (&SeatMapBuffer, &mut SeatMapBuffer) {
        let (left, right) = self.seat_bufs.split_at_mut(1);
        if self.cur_buf_ind == 0 {
            (&left[0], &mut right[0])
        } else {
            (&right[0], &mut left[0])
        }
    }

    fn cur_buf(&self) -> &SeatMapBuffer {
        &self.seat_bufs[self.cur_buf_ind]
    }

    fn swap_buf(&mut self) {
        self.cur_buf_ind = 1 - self.cur_buf_ind;
    }
}

struct SeatMapBuilder {
    tiles: Vec<TileType>,
    row_n: usize,
    col_n: Option<usize>,
}

impl SeatMapBuilder {
    pub fn new() -> Self {
        Self {
            tiles: Vec::new(),
            row_n: 0,
            col_n: None,
        }
    }

    pub fn add_row(&mut self, row_text: &str) -> Result<(), Error> {
        let tile_n = row_text.chars().count();
        if *self.col_n.get_or_insert(tile_n) != tile_n {
            return Err(Error::InconsisitentSeatMapRow(tile_n, self.col_n.unwrap()));
        }

        for c in row_text.chars() {
            self.tiles.push(TileType::try_from(c)?);
        }
        self.row_n += 1;

        Ok(())
    }

    pub fn build(self) -> SeatMap {
        let buf = SeatMapBuffer {
            tiles: self.tiles,
            row_n: self.row_n,
            col_n: self.col_n.unwrap_or(0),
        };
        SeatMap {
            seat_bufs: [buf.clone(), buf],
            cur_buf_ind: 0,
        }
    }
}

pub fn read_sm<P: AsRef<Path>>(path: P) -> Result<SeatMap, Error> {
    let file = File::open(path).map_err(Error::IOError)?;
    let reader = BufReader::new(file);
    let mut builder = SeatMapBuilder::new();
    for l in reader.lines() {
        let s = l.map_err(Error::IOError)?;
        builder.add_row(&s)?;
    }

    Ok(builder.build())
}
