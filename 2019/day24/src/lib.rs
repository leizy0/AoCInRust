use std::{
    env, error,
    fmt::Display,
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    WrongNumberOfArgs(usize, usize), // (number of given arguments, expected number of arguments)
    InconsistentRowInArea(usize, usize), // (number of elements in current row, number of elements in earlier rows)
    InvalidTileTypeChar(char),           // Invalid character for tile type
    OverflowedArea(usize),               // Number of elements in given area which exceeds limit.
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IOError(ioe) => write!(f, "I/O error: {}", ioe),
            Error::WrongNumberOfArgs(args_n, expect_args_n) => write!(f, "Given wrong number({}) of arguments, expect {}", args_n, expect_args_n),
            Error::InconsistentRowInArea(this_ele_n, expect_ele_n) => write!(f, "One row in given area has different number({}) of elements, but ealier rows all have {}", this_ele_n, expect_ele_n),
            Error::InvalidTileTypeChar(c) => write!(f, "Invalid character({}) found in given area", c),
            Error::OverflowedArea(ele_n) => write!(f, "Too many elements({} already processed) in given area, it's limited by computation of biodiversity rating", ele_n),
        }
    }
}

impl error::Error for Error {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TileType {
    Empty,
    Infested,
}

impl TryFrom<char> for TileType {
    type Error = Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '.' => Ok(Self::Empty),
            '#' => Ok(Self::Infested),
            other => Err(Error::InvalidTileTypeChar(other)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

struct Position {
    r: usize,
    c: usize,
}

impl Position {
    pub fn new(r: usize, c: usize) -> Self {
        Self { r, c }
    }

    pub fn adjacent(&self, dir: Direction) -> Option<Self> {
        match dir {
            Direction::Up if self.r > 0 => Some(Self::new(self.r - 1, self.c)),
            Direction::Down => Some(Self::new(self.r + 1, self.c)),
            Direction::Left if self.c > 0 => Some(Self::new(self.r, self.c - 1)),
            Direction::Right => Some(Self::new(self.r, self.c + 1)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
struct AreaBuffer {
    tiles: Vec<TileType>,
    row_n: usize,
    col_n: usize,
}

impl AreaBuffer {
    pub fn bio_rating(&self) -> usize {
        self.tiles
            .iter()
            .map(|tt| match tt {
                TileType::Empty => 0usize,
                TileType::Infested => 1usize,
            })
            .rfold(0, |acc, v| (acc << 1) | v)
    }

    pub fn tiles_n(&self) -> usize {
        self.tiles.len()
    }

    pub fn tile(&self, pos: &Position) -> Option<&TileType> {
        self.pos_to_ind(pos).and_then(|i| self.tiles.get(i))
    }

    pub fn tile_mut(&mut self, pos: &Position) -> Option<&mut TileType> {
        self.pos_to_ind(pos).and_then(|i| self.tiles.get_mut(i))
    }

    pub fn pos_to_ind(&self, pos: &Position) -> Option<usize> {
        if pos.r >= self.row_n || pos.c >= self.col_n {
            None
        } else {
            Some(pos.r * self.col_n + pos.c)
        }
    }

    pub fn ind_to_pos(&self, ind: usize) -> Option<Position> {
        if ind >= self.tiles.len() {
            None
        } else {
            Some(Position::new(ind / self.col_n, ind % self.col_n))
        }
    }
}

impl Display for AreaBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for r in 0..self.row_n {
            let row_str = (0..self.col_n)
                .map(|c| match self.tile(&Position::new(r, c)).unwrap() {
                    TileType::Empty => '.',
                    TileType::Infested => '#',
                })
                .collect::<String>();
            writeln!(f, "{}", row_str)?;
        }

        Ok(())
    }
}

pub struct Area {
    bufs: [AreaBuffer; 2],
    cur_ind: usize,
}

impl Area {
    pub fn step(&mut self) {
        let (read_buf, write_buf) = self.rw_bufs();
        for i in 0..read_buf.tiles_n() {
            let pos = read_buf.ind_to_pos(i).unwrap();
            let infested_adj_n = [
                Direction::Up,
                Direction::Down,
                Direction::Left,
                Direction::Right,
            ]
            .iter()
            .map(|d| {
                pos.adjacent(*d)
                    .and_then(|apos| read_buf.tile(&apos))
                    .copied()
                    .unwrap_or(TileType::Empty)
            })
            .filter(|tt| *tt == TileType::Infested)
            .count();
            let cur_tt = *read_buf.tile(&pos).unwrap();
            *write_buf.tile_mut(&pos).unwrap() = match cur_tt {
                TileType::Empty if infested_adj_n == 1 || infested_adj_n == 2 => TileType::Infested,
                TileType::Infested if infested_adj_n != 1 => TileType::Empty,
                _ => cur_tt,
            }
        }

        self.cur_ind = 1 - self.cur_ind;
    }

    pub fn bio_rating(&self) -> usize {
        self.cur_buf().bio_rating()
    }

    fn cur_buf(&self) -> &AreaBuffer {
        &self.bufs[self.cur_ind]
    }

    fn rw_bufs(&mut self) -> (&AreaBuffer, &mut AreaBuffer) {
        let (left, right) = self.bufs.split_at_mut(1);
        if self.cur_ind == 0 {
            (&left[0], &mut right[0])
        } else {
            (&right[0], &mut left[0])
        }
    }
}

impl Display for Area {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.cur_buf())
    }
}

struct AreaBuilder {
    tiles: Vec<TileType>,
    row_n: usize,
    col_n: Option<usize>,
}

impl AreaBuilder {
    pub fn new() -> Self {
        Self {
            tiles: Vec::new(),
            row_n: 0,
            col_n: None,
        }
    }

    pub fn push_row(&mut self, row_str: &str) -> Result<(), Error> {
        let this_row_n = row_str.len();
        if *self.col_n.get_or_insert(this_row_n) != this_row_n {
            return Err(Error::InconsistentRowInArea(
                this_row_n,
                self.col_n.unwrap(),
            ));
        }

        for c in row_str.chars() {
            self.tiles.push(TileType::try_from(c)?);
        }
        self.row_n += 1;

        Ok(())
    }

    pub fn build(self) -> Result<Area, Error> {
        let tiles_n = self.tiles.len();
        if tiles_n > usize::try_from(usize::BITS).unwrap() {
            return Err(Error::OverflowedArea(tiles_n));
        }
        debug_assert!(tiles_n == self.row_n * self.col_n.unwrap_or(0));

        let buffer = AreaBuffer {
            tiles: self.tiles,
            row_n: self.row_n,
            col_n: self.col_n.unwrap_or(0),
        };
        Ok(Area {
            bufs: [buffer.clone(), buffer],
            cur_ind: 0,
        })
    }
}

pub fn check_args() -> Result<String, Error> {
    let args = env::args();
    let args_n = args.len();
    const EXPECT_ARGS_N: usize = 2;
    if args_n != EXPECT_ARGS_N {
        Err(Error::WrongNumberOfArgs(args_n, EXPECT_ARGS_N))
    } else {
        Ok(args.skip(1).next().unwrap().to_string())
    }
}

pub fn read_area<P: AsRef<Path>>(path: P) -> Result<Area, Error> {
    let area_file = File::open(path).map_err(Error::IOError)?;
    let reader = BufReader::new(area_file);

    let mut builder = AreaBuilder::new();
    for line in reader.lines() {
        line.map_err(Error::IOError)
            .and_then(|s| builder.push_row(s.as_str()))?;
    }

    Ok(builder.build()?)
}
