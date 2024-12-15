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
    InconsistentRow(usize, usize),
    MultipleRobots(Position, Position),
    InvalidCharforMap(char),
    NoRobotInMap,
    InvalidCharforDirection(char),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InconsistentRow(expect_col_n, this_col_n) => write!(
                f,
                "Expect {} columns in each row, given {}.",
                expect_col_n, this_col_n
            ),
            Error::MultipleRobots(last_position, this_position) => write!(
                f,
                "Given two robots in map({}, {}), expect only one.",
                last_position, this_position
            ),
            Error::InvalidCharforMap(c) => write!(f, "Invalid character({}) for map.", c),
            Error::NoRobotInMap => write!(f, "No robot found in given map, expect one."),
            Error::InvalidCharforDirection(c) => {
                write!(f, "Invalid character({}) for direction.", c)
            }
        }
    }
}

impl error::Error for Error {}

#[derive(Debug, Parser)]
pub struct CLIArgs {
    pub input_path: PathBuf,
}

pub trait Tile: Copy + Eq + Display {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlainTile {
    Wall,
    Robot,
    Box,
    Floor,
}

impl Display for PlainTile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let tile_char = match self {
            PlainTile::Wall => '#',
            PlainTile::Robot => '@',
            PlainTile::Box => 'O',
            PlainTile::Floor => '.',
        };

        write!(f, "{}", tile_char)
    }
}

impl Tile for PlainTile {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WideTile {
    Wall,
    WideBoxLeft,
    WideBoxRight,
    Robot,
    Floor,
}

impl Display for WideTile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let tile_char = match self {
            WideTile::Wall => '#',
            WideTile::Robot => '@',
            WideTile::WideBoxLeft => '[',
            WideTile::WideBoxRight => ']',
            WideTile::Floor => '.',
        };

        write!(f, "{}", tile_char)
    }
}

impl Tile for WideTile {}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub fn new(r: usize, c: usize) -> Self {
        Self { r, c }
    }

    pub fn neighbor(&self, dir: Direction) -> Option<Self> {
        match dir {
            Direction::Up if self.r > 0 => Some(Position::new(self.r - 1, self.c)),
            Direction::Right => Some(Position::new(self.r, self.c + 1)),
            Direction::Down => Some(Position::new(self.r + 1, self.c)),
            Direction::Left if self.c > 0 => Some(Position::new(self.r, self.c - 1)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl TryFrom<char> for Direction {
    type Error = Error;

    fn try_from(value: char) -> std::result::Result<Self, Self::Error> {
        match value {
            '^' => Ok(Direction::Up),
            '>' => Ok(Direction::Right),
            'v' => Ok(Direction::Down),
            '<' => Ok(Direction::Left),
            other => Err(Error::InvalidCharforDirection(other)),
        }
    }
}

#[derive(Debug)]
pub struct Map<T: Tile> {
    tiles: Vec<T>,
    robot_pos: Position,
    row_n: usize,
    col_n: usize,
}

impl<T: Tile> Display for Map<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for r in 0..self.row_n {
            for c in 0..self.col_n {
                let pos = Position::new(r, c);
                write!(f, "{}", self.tile(&pos).unwrap())?;
            }
            writeln!(f, "")?;
        }

        Ok(())
    }
}

impl<T: Tile> Map<T> {
    pub fn position_iter(&self, tile: T) -> impl Iterator<Item = Position> + use<'_, T> {
        self.tiles
            .iter()
            .enumerate()
            .filter(move |(_, this_tile)| **this_tile == tile)
            .map(|(ind, _)| self.ind_to_pos(ind).unwrap())
    }

    fn tile(&self, pos: &Position) -> Option<&T> {
        self.pos_to_ind(pos).and_then(|ind| self.tiles.get(ind))
    }

    fn tile_mut(&mut self, pos: &Position) -> Option<&mut T> {
        self.pos_to_ind(pos).and_then(|ind| self.tiles.get_mut(ind))
    }

    fn pos_to_ind(&self, pos: &Position) -> Option<usize> {
        if pos.r < self.row_n && pos.c < self.col_n {
            Some(pos.r * self.col_n + pos.c)
        } else {
            None
        }
    }

    fn ind_to_pos(&self, ind: usize) -> Option<Position> {
        if ind < self.tiles.len() {
            Some(Position::new(ind / self.col_n, ind % self.col_n))
        } else {
            None
        }
    }
}

impl Map<PlainTile> {
    pub fn widen(&self) -> Map<WideTile> {
        let mut tiles = Vec::with_capacity(self.tiles.len() * 2);
        for tile in self.tiles.iter() {
            tiles.extend_from_slice(&match tile {
                PlainTile::Wall => [WideTile::Wall, WideTile::Wall],
                PlainTile::Robot => [WideTile::Robot, WideTile::Floor],
                PlainTile::Box => [WideTile::WideBoxLeft, WideTile::WideBoxRight],
                PlainTile::Floor => [WideTile::Floor, WideTile::Floor],
            });
        }
        let robot_pos = Position::new(self.robot_pos.r, self.robot_pos.c * 2);

        Map::<WideTile> {
            tiles,
            robot_pos,
            row_n: self.row_n,
            col_n: self.col_n * 2,
        }
    }
}

#[derive(Debug)]
pub struct BoxGame {
    map: Map<PlainTile>,
}

impl BoxGame {
    pub fn new(map: Map<PlainTile>) -> Self {
        Self { map }
    }

    pub fn simulate(&mut self, dirs: &[Direction]) {
        for dir in dirs {
            self.try_move(&self.map.robot_pos.clone(), *dir);
        }
    }

    fn try_move(&mut self, pos: &Position, dir: Direction) -> bool {
        if let Some(cur_tile) = self.map.tile(pos).cloned() {
            match cur_tile {
                PlainTile::Robot | PlainTile::Box => {
                    if let Some(next_position) = pos.neighbor(dir) {
                        if let Some(next_tile) = self.map.tile(&next_position).cloned() {
                            match next_tile {
                                PlainTile::Box | PlainTile::Floor => {
                                    if self.try_move(&next_position, dir) {
                                        *self.map.tile_mut(&next_position).unwrap() = cur_tile;
                                        *self.map.tile_mut(&pos).unwrap() = PlainTile::Floor;
                                        if cur_tile == PlainTile::Robot {
                                            self.map.robot_pos = next_position;
                                        }
                                        return true;
                                    }
                                }
                                _ => (),
                            }
                        }
                    }
                }
                PlainTile::Floor => return true,
                _ => (),
            }
        }

        false
    }

    pub fn map(&self) -> &Map<PlainTile> {
        &self.map
    }
}

#[derive(Debug)]
pub struct WideBoxGame {
    map: Map<WideTile>,
}

impl WideBoxGame {
    pub fn new(map: Map<WideTile>) -> Self {
        Self { map }
    }

    pub fn simulate(&mut self, dirs: &[Direction]) {
        for dir in dirs {
            self.try_move(&self.map.robot_pos.clone(), *dir);
        }
    }

    fn try_move(&mut self, pos: &Position, dir: Direction) -> bool {
        if let Some(cur_tile) = self.map.tile(pos).cloned() {
            match cur_tile {
                WideTile::WideBoxLeft => {
                    return self.try_move_box(pos, &pos.neighbor(Direction::Right).unwrap(), dir)
                }
                WideTile::WideBoxRight => {
                    return self.try_move_box(&pos.neighbor(Direction::Left).unwrap(), pos, dir)
                }
                WideTile::Robot => {
                    if let Some(next_position) = pos.neighbor(dir) {
                        if let Some(next_tile) = self.map.tile(&next_position).cloned() {
                            if match next_tile {
                                WideTile::WideBoxLeft => self.try_move_box(
                                    &next_position,
                                    &next_position.neighbor(Direction::Right).unwrap(),
                                    dir,
                                ),
                                WideTile::WideBoxRight => self.try_move_box(
                                    &next_position.neighbor(Direction::Left).unwrap(),
                                    &next_position,
                                    dir,
                                ),
                                WideTile::Floor => true,
                                _ => false,
                            } {
                                *self.map.tile_mut(&next_position).unwrap() = cur_tile;
                                *self.map.tile_mut(&pos).unwrap() = WideTile::Floor;
                                self.map.robot_pos = next_position;
                                return true;
                            }
                        }
                    }
                }
                WideTile::Floor => return true,
                _ => (),
            }
        }

        false
    }

    pub fn map(&self) -> &Map<WideTile> {
        &self.map
    }

    fn try_move_box(&mut self, left_pos: &Position, right_pos: &Position, dir: Direction) -> bool {
        debug_assert!(self
            .map()
            .tile(left_pos)
            .is_some_and(|tile| *tile == WideTile::WideBoxLeft));
        debug_assert!(self
            .map()
            .tile(right_pos)
            .is_some_and(|tile| *tile == WideTile::WideBoxRight));

        match dir {
            Direction::Left | Direction::Right => {
                if let Some(next_pos) = if dir == Direction::Left {
                    left_pos.neighbor(dir)
                } else {
                    right_pos.neighbor(dir)
                } {
                    if let Some(next_tile) = self.map().tile(&next_pos).copied() {
                        if match next_tile {
                            WideTile::WideBoxLeft => self.try_move_box(
                                &next_pos,
                                &next_pos.neighbor(Direction::Right).unwrap(),
                                dir,
                            ),
                            WideTile::WideBoxRight => self.try_move_box(
                                &next_pos.neighbor(Direction::Left).unwrap(),
                                &next_pos,
                                dir,
                            ),
                            WideTile::Floor => true,
                            _ => false,
                        } {
                            if dir == Direction::Left {
                                *self.map.tile_mut(&next_pos).unwrap() = WideTile::WideBoxLeft;
                                *self.map.tile_mut(left_pos).unwrap() = WideTile::WideBoxRight;
                                *self.map.tile_mut(right_pos).unwrap() = WideTile::Floor;
                            } else {
                                *self.map.tile_mut(&next_pos).unwrap() = WideTile::WideBoxRight;
                                *self.map.tile_mut(right_pos).unwrap() = WideTile::WideBoxLeft;
                                *self.map.tile_mut(left_pos).unwrap() = WideTile::Floor;
                            }

                            return true;
                        }
                    }
                }
            }
            Direction::Up | Direction::Down => {
                if let Some(next_left_pos) = left_pos.neighbor(dir) {
                    if let Some(next_right_pos) = right_pos.neighbor(dir) {
                        if self.can_move_to(&next_left_pos, dir)
                            && self.can_move_to(&next_right_pos, dir)
                        {
                            self.try_move(&next_left_pos, dir);
                            self.try_move(&next_right_pos, dir);
                            *self.map.tile_mut(&next_left_pos).unwrap() = WideTile::WideBoxLeft;
                            *self.map.tile_mut(&next_right_pos).unwrap() = WideTile::WideBoxRight;
                            *self.map.tile_mut(left_pos).unwrap() = WideTile::Floor;
                            *self.map.tile_mut(right_pos).unwrap() = WideTile::Floor;
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    fn can_move_to(&self, pos: &Position, dir: Direction) -> bool {
        if let Some(tile) = self.map.tile(pos) {
            match tile {
                WideTile::WideBoxLeft | WideTile::WideBoxRight => {
                    if let Some(next_pos) = pos.neighbor(dir) {
                        if let Some((next_left_pos, next_right_pos)) =
                            if *tile == WideTile::WideBoxLeft {
                                pos.neighbor(Direction::Right)
                                    .and_then(|right_pos| right_pos.neighbor(dir))
                                    .map(|next_right_pos| (next_pos, next_right_pos))
                            } else {
                                pos.neighbor(Direction::Left)
                                    .and_then(|left_pos| left_pos.neighbor(dir))
                                    .map(|next_left_pos| (next_left_pos, next_pos))
                            }
                        {
                            return self.can_move_to(&next_left_pos, dir)
                                && self.can_move_to(&next_right_pos, dir);
                        }
                    }
                }
                WideTile::Floor => return true,
                _ => (),
            }
        }

        false
    }
}

pub struct PlainMapBuilder {
    tiles: Vec<PlainTile>,
    robot_pos: Option<Position>,
    row_n: usize,
    col_n: Option<usize>,
}

impl PlainMapBuilder {
    pub fn new() -> Self {
        Self {
            tiles: Vec::new(),
            robot_pos: None,
            row_n: 0,
            col_n: None,
        }
    }

    pub fn add_row(&mut self, text: &str) -> Result<(), Error> {
        let this_col_n = text.chars().count();
        if *self.col_n.get_or_insert(this_col_n) != this_col_n {
            return Err(Error::InconsistentRow(self.col_n.unwrap(), this_col_n));
        }

        for (ind, c) in text.chars().enumerate() {
            self.tiles.push(match c {
                '#' => PlainTile::Wall,
                '.' => PlainTile::Floor,
                '@' => {
                    let this_robot_pos = Position::new(self.row_n, ind);
                    if *self.robot_pos.get_or_insert(this_robot_pos.clone()) != this_robot_pos {
                        return Err(Error::MultipleRobots(
                            self.robot_pos.as_ref().unwrap().clone(),
                            this_robot_pos,
                        ));
                    }

                    PlainTile::Robot
                }
                'O' => PlainTile::Box,
                other => return Err(Error::InvalidCharforMap(other)),
            });
        }
        self.row_n += 1;

        Ok(())
    }

    pub fn build(self) -> Result<Map<PlainTile>, Error> {
        if let Some(robot_pos) = self.robot_pos {
            Ok(Map::<PlainTile> {
                tiles: self.tiles,
                robot_pos,
                row_n: self.row_n,
                col_n: self.col_n.unwrap_or(0),
            })
        } else {
            Err(Error::NoRobotInMap)
        }
    }
}

pub fn read_game<P: AsRef<Path>>(path: P) -> Result<(Map<PlainTile>, Vec<Direction>)> {
    let file = File::open(&path)
        .with_context(|| format!("Failed to open given file({}).", path.as_ref().display()))?;
    let reader = BufReader::new(file);
    let mut builder = PlainMapBuilder::new();
    let mut enum_lines = reader.lines().enumerate();
    while let Some((ind, line)) = enum_lines.next() {
        let line = line.with_context(|| {
            format!(
                "Failed to read line {} in given file({}).",
                ind + 1,
                path.as_ref().display()
            )
        })?;
        if line.is_empty() {
            break;
        }

        builder.add_row(line.as_str())?;
    }

    let mut move_dirs = Vec::new();
    while let Some((ind, line)) = enum_lines.next() {
        let line = line.with_context(|| {
            format!(
                "Failed to read line {} in given file({}).",
                ind + 1,
                path.as_ref().display()
            )
        })?;
        for c in line.chars() {
            move_dirs.push(Direction::try_from(c)?);
        }
    }

    Ok((builder.build()?, move_dirs))
}
