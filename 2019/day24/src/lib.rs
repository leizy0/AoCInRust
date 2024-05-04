use std::{
    collections::VecDeque,
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

impl TileType {
    pub fn next_type(&self, adj_infested_n: usize) -> TileType {
        match self {
            TileType::Empty if adj_infested_n == 1 || adj_infested_n == 2 => TileType::Infested,
            TileType::Infested if adj_infested_n != 1 => TileType::Empty,
            _ => *self,
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

const ALL_DIRECTIONS: [Direction; 4] = [
    Direction::Up,
    Direction::Down,
    Direction::Left,
    Direction::Right,
];

#[derive(Debug, Clone, PartialEq, Eq)]
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

    pub fn ind_to_pos(&self, ind: usize) -> Option<Position> {
        if ind >= self.tiles.len() {
            None
        } else {
            Some(Position::new(ind / self.col_n, ind % self.col_n))
        }
    }

    pub fn clone_empty(&self) -> Self {
        Self {
            tiles: vec![TileType::Empty; self.tiles_n()],
            row_n: self.row_n,
            col_n: self.col_n,
        }
    }

    fn pos_to_ind(&self, pos: &Position) -> Option<usize> {
        if pos.r >= self.row_n || pos.c >= self.col_n {
            None
        } else {
            Some(pos.r * self.col_n + pos.c)
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

#[derive(Debug, Clone)]
pub struct Area {
    bufs: [AreaBuffer; 2],
    cur_ind: usize,
}

impl Area {
    pub fn step(&mut self) {
        self.update();
        self.switch_bufs();
    }

    pub fn bio_rating(&self) -> usize {
        self.cur_buf().bio_rating()
    }

    pub fn row_n(&self) -> usize {
        self.cur_buf().row_n
    }

    pub fn col_n(&self) -> usize {
        self.cur_buf().col_n
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

    fn update(&mut self) {
        for i in 0..self.cur_buf().tiles_n() {
            let pos = self.cur_buf().ind_to_pos(i).unwrap();
            let adj_infested_n = ALL_DIRECTIONS
                .iter()
                .map(|d| {
                    pos.adjacent(*d)
                        .and_then(|apos| self.cur_buf().tile(&apos))
                        .copied()
                        .unwrap_or(TileType::Empty)
                })
                .filter(|tt| *tt == TileType::Infested)
                .count();

            self.update_tile(&pos, adj_infested_n);
        }
    }

    fn update_tile(&mut self, pos: &Position, adj_infested_n: usize) {
        let (read_buf, write_buf) = self.rw_bufs();
        *write_buf.tile_mut(&pos).unwrap() = read_buf.tile(&pos).unwrap().next_type(adj_infested_n);
    }

    fn switch_bufs(&mut self) {
        self.cur_ind = 1 - self.cur_ind;
    }

    fn clone_empty(&self) -> Self {
        let new_buf = self.cur_buf().clone_empty();
        Self {
            bufs: [new_buf.clone(), new_buf],
            cur_ind: 0,
        }
    }
}

impl Display for Area {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.cur_buf())
    }
}

enum RecurPosIterator {
    Single(Option<RecursivePosition>),
    Row(RecursivePosition),
    Column(RecursivePosition),
}

#[derive(Debug, Clone)]
struct RecursivePosition {
    pos: Position,
    level: isize,
    row_n: usize,
    col_n: usize,
}

impl Iterator for RecurPosIterator {
    type Item = RecursivePosition;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            RecurPosIterator::Single(rp_op) => rp_op.take(),
            RecurPosIterator::Row(rp) => {
                if rp.pos.c >= rp.col_n {
                    None
                } else {
                    let res = rp.clone();
                    rp.pos.c += 1;

                    Some(res)
                }
            }
            RecurPosIterator::Column(rp) => {
                if rp.pos.r >= rp.row_n {
                    None
                } else {
                    let res = rp.clone();
                    rp.pos.r += 1;

                    Some(res)
                }
            }
        }
    }
}

impl RecursivePosition {
    pub fn new(pos: &Position, level: isize, row_n: usize, col_n: usize) -> Self {
        assert!(row_n % 2 == 1 && col_n % 2 == 1, "No center, no recursion.");
        Self {
            pos: pos.clone(),
            level,
            row_n,
            col_n,
        }
    }

    pub fn adjacent(&self, dir: Direction) -> RecurPosIterator {
        let center_pos = self.center_pos();
        if let Some(adj_pos) = self
            .pos
            .adjacent(dir)
            .filter(|p| p.r < self.row_n && p.c < self.col_n)
        {
            if adj_pos == center_pos {
                // Step into the next recursive level.
                match dir {
                    // Bottom row in area of the next level.
                    Direction::Up => RecurPosIterator::Row(RecursivePosition::new(
                        &Position::new(self.row_n - 1, 0),
                        self.level + 1,
                        self.row_n,
                        self.col_n,
                    )),
                    // Top row in area of the next level.
                    Direction::Down => RecurPosIterator::Row(RecursivePosition::new(
                        &Position::new(0, 0),
                        self.level + 1,
                        self.row_n,
                        self.col_n,
                    )),
                    // Column at right border in area of the next level.
                    Direction::Left => RecurPosIterator::Column(RecursivePosition::new(
                        &Position::new(0, self.col_n - 1),
                        self.level + 1,
                        self.row_n,
                        self.col_n,
                    )),
                    // Column at left border in area of the next level.
                    Direction::Right => RecurPosIterator::Column(RecursivePosition::new(
                        &Position::new(0, 0),
                        self.level + 1,
                        self.row_n,
                        self.col_n,
                    )),
                }
            } else {
                // Stay in the current level.
                RecurPosIterator::Single(Some(RecursivePosition::new(
                    &adj_pos, self.level, self.row_n, self.col_n,
                )))
            }
        } else {
            // Step out to the last rcursive level.
            RecurPosIterator::Single(
                center_pos
                    .adjacent(dir)
                    .map(|p| RecursivePosition::new(&p, self.level - 1, self.row_n, self.col_n)),
            )
        }
    }

    fn center_pos(&self) -> Position {
        Position::new(self.row_n / 2, self.col_n / 2)
    }
}

pub struct RecursiveArea {
    areas: VecDeque<Area>,
    top_level: isize,
    recur_pos: Position,
    recur_ind: usize,
}

impl Display for RecursiveArea {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bot_level = self.bot_level();
        let row_n = self.areas[0].row_n();
        let col_n = self.areas[0].col_n();
        for level in self.top_level..=bot_level {
            let area_ind = usize::try_from(level - self.top_level).unwrap();
            let area_str = self.areas[area_ind].to_string();
            let center_char_ind = (col_n + 1) * (row_n / 2) + col_n / 2;
            let recur_area_str = area_str
                .chars()
                .enumerate()
                .map(|(ind, c)| if ind == center_char_ind { '?' } else { c })
                .collect::<String>();

            writeln!(f, "Level {}:\n{}", level, recur_area_str)?;
        }

        Ok(())
    }
}

impl RecursiveArea {
    pub fn new(area: &Area) -> Self {
        let row_n = area.row_n();
        let col_n = area.col_n();
        assert!(row_n % 2 == 1 && col_n % 2 == 1, "No center, no recursion.");

        let recur_pos = Position::new(row_n / 2, col_n / 2);
        let recur_ind = recur_pos.r * col_n + recur_pos.c;

        Self {
            areas: VecDeque::from([area.clone()]),
            top_level: 0,
            recur_pos,
            recur_ind,
        }
    }

    pub fn step(&mut self) {
        let top_level = self.top_level;
        let bot_level = self.bot_level();
        self.new_top_area();
        self.new_bot_area();
        let (row_n, col_n) = self.size();
        // Update areas exist before this step.
        for level in top_level..=bot_level {
            let area_ind = usize::try_from(level - self.top_level).unwrap();
            for r in 0..row_n {
                for c in 0..col_n {
                    let pos = Position::new(r, c);
                    if pos != self.recur_pos {
                        let adj_infested_n = self
                            .adj_infested_n_at(&RecursivePosition::new(&pos, level, row_n, col_n));
                        self.areas[area_ind].update_tile(&pos, adj_infested_n);
                    }
                }
            }
        }

        for area in self.areas.iter_mut() {
            area.switch_bufs();
        }
    }

    pub fn infested_n(&self) -> usize {
        self.areas
            .iter()
            .map(|area| {
                area.cur_buf()
                    .tiles
                    .iter()
                    .enumerate()
                    .filter(|(ind, tt)| *ind != self.recur_ind && **tt == TileType::Infested)
                    .count()
            })
            .sum()
    }

    fn area(&self, level: isize) -> &Area {
        debug_assert!(level >= self.top_level && level <= self.bot_level());
        let area_ind = usize::try_from(level - self.top_level).unwrap();
        &self.areas[area_ind]
    }

    fn bot_level(&self) -> isize {
        self.top_level + isize::try_from(self.areas.len() - 1).unwrap()
    }

    fn new_top_area(&mut self) {
        let mut new_top_area = self.area(self.top_level).clone_empty();
        let (row_n, col_n) = self.size();
        // Only the tiles around recursive tile in new top area need to be updated.
        for rpos in ALL_DIRECTIONS.iter().flat_map(|d| {
            self.recur_pos
                .adjacent(*d)
                .map(|p| RecursivePosition::new(&p, self.top_level - 1, row_n, col_n))
        }) {
            let adj_infested_n = self.adj_infested_n_at(&rpos);
            new_top_area.update_tile(&rpos.pos, adj_infested_n);
        }

        self.areas.push_front(new_top_area);
        self.top_level -= 1;
    }

    fn new_bot_area(&mut self) {
        fn border_line_pos(
            dir: Direction,
            level: isize,
            row_n: usize,
            col_n: usize,
        ) -> RecurPosIterator {
            match dir {
                Direction::Up => RecurPosIterator::Row(RecursivePosition::new(
                    &Position::new(0, 0),
                    level,
                    row_n,
                    col_n,
                )),
                Direction::Down => RecurPosIterator::Row(RecursivePosition::new(
                    &Position::new(row_n - 1, 0),
                    level,
                    row_n,
                    col_n,
                )),
                Direction::Left => RecurPosIterator::Column(RecursivePosition::new(
                    &Position::new(0, 0),
                    level,
                    row_n,
                    col_n,
                )),
                Direction::Right => RecurPosIterator::Column(RecursivePosition::new(
                    &Position::new(0, col_n - 1),
                    level,
                    row_n,
                    col_n,
                )),
            }
        }

        let mut new_bot_area = self.area(self.top_level).clone_empty();
        let (row_n, col_n) = self.size();
        // Only the tiles on border line of new bottom area need to be updated.
        let bot_level = self.bot_level();
        for dir in ALL_DIRECTIONS {
            for rpos in border_line_pos(dir, bot_level + 1, row_n, col_n) {
                let adj_infested_n = self.adj_infested_n_at(&rpos);
                new_bot_area.update_tile(&rpos.pos, adj_infested_n);
            }
        }

        self.areas.push_back(new_bot_area);
    }

    fn adj_infested_n_at(&self, rpos: &RecursivePosition) -> usize {
        // Assume any tiles that aren't in current areas are empty(not infested).
        ALL_DIRECTIONS
            .iter()
            .flat_map(|d| rpos.adjacent(*d))
            .filter_map(|rp| self.tile(&rp).filter(|tt| **tt == TileType::Infested))
            .count()
    }

    fn tile(&self, rpos: &RecursivePosition) -> Option<&TileType> {
        usize::try_from(rpos.level - self.top_level)
            .ok()
            .and_then(|l_ind| {
                self.areas
                    .get(l_ind)
                    .and_then(|a| a.cur_buf().tile(&rpos.pos))
            })
    }

    fn size(&self) -> (usize, usize) {
        let top_area = &self.areas[0];
        (top_area.row_n(), top_area.col_n())
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
