use std::{
    collections::{HashMap, LinkedList},
    error,
    fmt::Display,
};

use int_enum::IntEnum;

use crate::int_code::io::{InputPort, OutputPort};

#[derive(Debug)]
pub enum Error {
    UnknownMoveResult(i64, Direction, Position),
    InvalidInitMoves {
        cur_pos: Position,
        move_steps: usize,
        init_moves: Vec<Direction>,
    },
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::UnknownMoveResult(res, dir, pos) => write!(
                f,
                "Get unknown move result({}) when try to move to {:?} of {:?}",
                res, dir, pos
            ),
            Error::InvalidInitMoves {
                cur_pos,
                move_steps,
                init_moves,
            } => write!(
                f,
                "Knock into a wall at {}, after {} steps of invalid initial moves({:?})",
                cur_pos, move_steps, init_moves
            ),
        }
    }
}

impl error::Error for Error {}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Default, IntEnum, PartialEq, Eq)]
enum TileType {
    #[default]
    Empty = 1,
    Wall = 0,
    OxygenSystem = 2,
}

impl Display for TileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TileType::Empty => write!(f, "empty"),
            TileType::Wall => write!(f, "wall"),
            TileType::OxygenSystem => write!(f, "oxygen system"),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Default, IntEnum, PartialEq, Eq)]
pub enum Direction {
    #[default]
    North = 1,
    South = 2,
    West = 3,
    East = 4,
}

impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::North => write!(f, "north"),
            Direction::South => write!(f, "south"),
            Direction::West => write!(f, "west"),
            Direction::East => write!(f, "east"),
        }
    }
}

impl Direction {
    pub fn reverse(&self) -> Self {
        match self {
            Self::North => Self::South,
            Self::South => Self::North,
            Self::West => Self::East,
            Self::East => Self::West,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    x: i32,
    y: i32,
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn origin() -> Self {
        Self::new(0, 0)
    }

    pub fn move_along(&self, dir: Direction) -> Self {
        match dir {
            Direction::North => Position::new(self.x, self.y + 1),
            Direction::South => Position::new(self.x, self.y - 1),
            Direction::West => Position::new(self.x - 1, self.y),
            Direction::East => Position::new(self.x + 1, self.y),
        }
    }

    pub fn neighbors_and_dir(&self) -> [(Position, Direction); 4] {
        [
            (self.move_along(Direction::North), Direction::North),
            (self.move_along(Direction::South), Direction::South),
            (self.move_along(Direction::West), Direction::West),
            (self.move_along(Direction::East), Direction::East),
        ]
    }
}

pub struct Autopilot {
    map: HashMap<Position, (Option<Direction>, TileType)>,
    search_org: Option<Position>,
    cur_pos: Position,
    oxygen_sys_pos: Option<Position>,
    last_move_dir: Option<Direction>,
    search_plan: LinkedList<(Position, Direction)>,
    move_plan: LinkedList<Direction>,
    init_moves: Vec<Direction>,
    init_move_ind: usize,
    is_end: Box<dyn Fn(&Self) -> bool>,
}

impl Autopilot {
    pub fn new(init_moves: Vec<Direction>, is_end: Box<dyn Fn(&Self) -> bool>) -> Self {
        let mut res = Self {
            map: HashMap::new(),
            search_org: None,
            cur_pos: Position::origin(),
            oxygen_sys_pos: None,
            last_move_dir: None,
            search_plan: LinkedList::new(),
            move_plan: LinkedList::new(),
            init_moves,
            init_move_ind: 0,
            is_end,
        };

        if res.init_moves.is_empty() {
            res.search_cur_neighbor();
            res.search_org = Some(res.cur_pos);
        }

        res
    }

    pub fn cur_pos(&self) -> Position {
        self.cur_pos
    }

    pub fn oxygen_sys_pos(&self) -> Option<Position> {
        self.oxygen_sys_pos
    }

    pub fn moves_from_origin(&self, pos: &Position) -> Vec<Direction> {
        let mut moves = Vec::new();
        let mut cur_pos = *pos;
        let origin = self.search_org.unwrap();
        while cur_pos != origin {
            if let Some((dir_op, tile_type)) = self.map.get(&cur_pos) {
                debug_assert!(*tile_type != TileType::Wall);
                let dir = dir_op.unwrap();
                moves.push(dir);
                cur_pos = cur_pos.move_along(dir.reverse());
            } else {
                panic!(
                    "Unexpected break(at {}) in path from {} to {}",
                    cur_pos, origin, pos
                );
            }
        }

        moves.reverse();
        moves
    }

    pub fn moves_to_origin(&self, pos: &Position) -> Vec<Direction> {
        let mut moves = Vec::new();
        let mut cur_pos = *pos;
        let origin = self.search_org.unwrap();
        while cur_pos != origin {
            if let Some((dir_op, tile_type)) = self.map.get(&cur_pos) {
                debug_assert!(*tile_type != TileType::Wall);
                let rev_dir = dir_op.unwrap().reverse();
                moves.push(rev_dir);
                cur_pos = cur_pos.move_along(rev_dir);
            } else {
                panic!(
                    "Unexpected break(at {}) in path from {} to {}",
                    cur_pos, pos, origin
                );
            }
        }

        moves
    }

    fn search_cur_neighbor(&mut self) {
        // Push back neighbor positions to search later.
        let neighbors = self.cur_pos.neighbors_and_dir();
        self.search_plan
            .extend(neighbors.iter().filter(|(p, _)| !self.map.contains_key(p)));
    }

    fn plan_moves_to(&mut self, target_pos: &Position, final_dir: Direction) {
        if self.cur_pos != self.search_org.unwrap() {
            // Move to origin along known shortest path.
            self.move_plan
                .extend(self.moves_to_origin(&self.cur_pos).iter());
        }

        if *target_pos != self.search_org.unwrap() {
            // Move from origin to target position along known shortest path.
            let known_nearest_pos = target_pos.move_along(final_dir.reverse());
            let mut origin_to_target = self.moves_from_origin(&known_nearest_pos);
            origin_to_target.push(final_dir);

            // Cut reverse direction pairs, eliminates any unnecessary moves.
            let mut extend_start = 0;
            while extend_start < origin_to_target.len() {
                if let Some(dir) = self.move_plan.back() {
                    if dir.reverse() == origin_to_target[extend_start] {
                        // Found reverse pairs, delete them from two slice.
                        self.move_plan.pop_back();
                        extend_start += 1;
                    } else {
                        // No more reverse pairs.
                        break;
                    }
                } else {
                    // Moves to origin is empty.
                    break;
                }
            }

            self.move_plan
                .extend(origin_to_target.iter().skip(extend_start));
        }
    }

    fn try_move(&mut self, dir: Direction) -> Direction {
        self.last_move_dir = Some(dir);
        dir
    }
}

impl InputPort for Autopilot {
    fn get(&mut self) -> Option<i64> {
        if let Some((_, TileType::OxygenSystem)) = self.map.get(&self.cur_pos) {
            if self.oxygen_sys_pos.is_none() {
                // Found the oxygen system, update state.
                self.oxygen_sys_pos = Some(self.cur_pos);
            }
        }

        if (self.is_end)(self) {
            // End condition reached, ends the program by blocking it with empty input.
            return None;
        }

        if self.init_move_ind < self.init_moves.len() {
            // Move along the initial moves first.
            let dir = self.init_moves[self.init_move_ind];
            self.init_move_ind += 1;
            return Some(self.try_move(dir).int_value().into());
        }

        if let Some(dir) = self.move_plan.pop_front() {
            // Follow the move plan, move to target position for continuing search
            return Some(self.try_move(dir).int_value().into());
        }

        debug_assert!(self.move_plan.is_empty());
        // Breadth first search
        while let Some((pos, dir)) = self.search_plan.pop_front() {
            // Set move plan for moving to this position.
            if !self.map.contains_key(&pos) {
                self.plan_moves_to(&pos, dir);
                break;
            }
        }

        self.move_plan
            .pop_front()
            .map(|dir| self.try_move(dir).int_value().into())
    }

    fn reg_proc(&mut self, _proc_id: usize) {}
}

impl OutputPort for Autopilot {
    fn put(&mut self, value: i64) -> Result<(), crate::Error> {
        let expected_pos = self.cur_pos.move_along(self.last_move_dir.unwrap());
        if self.init_move_ind < self.init_moves.len() {
            // Skip further process before finish the initial moves.
            if value == 0 {
                let error = Error::InvalidInitMoves {
                    cur_pos: expected_pos,
                    move_steps: self.init_move_ind,
                    init_moves: self.init_moves.clone(),
                };
                return Err(crate::Error::IOProcessError(error.to_string()));
            }

            self.cur_pos = expected_pos;
            return Ok(());
        } else if self.search_org.is_none() {
            // Reach origin of search, update state.
            self.search_org = Some(expected_pos);
        }

        match value {
            0 => {
                // Move failed, hit wall.
                debug_assert!(expected_pos != self.search_org.unwrap());
                self.map.entry(expected_pos).or_insert_with(|| {
                    println!("Observed position({}), it's a wall.", expected_pos);
                    (self.last_move_dir, TileType::Wall)
                });
            }
            1 | 2 => {
                // Move successfully.
                self.cur_pos = expected_pos;
                let dir_op = if self.cur_pos != self.search_org.unwrap() {
                    self.last_move_dir
                } else {
                    None
                };

                let tile_type = TileType::from_int(u8::try_from(value).unwrap()).unwrap();
                self.map.entry(self.cur_pos).or_insert_with(|| {
                    println!("Reached position({}), it's {}.", self.cur_pos, tile_type);
                    (dir_op, tile_type)
                });
                self.search_cur_neighbor();
            }
            _ => {
                return Err(crate::Error::IOProcessError(
                    Error::UnknownMoveResult(value, self.last_move_dir.unwrap(), self.cur_pos)
                        .to_string(),
                ))
            }
        }

        Ok(())
    }

    fn wait_proc_id(&self) -> Option<usize> {
        None
    }
}
