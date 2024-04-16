use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap, HashSet},
    fmt::Display,
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};

use maze::{
    walker::{MazeStep, MazeWalker},
    Maze, MazeMapBuilder, MazePosition, PlainMaze, TileType,
};

pub mod maze;

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    InvalidMapBuilderState(String, String), // (current state of map builder, procedure name that produced this error)
    NoMapInGivenInput,
    NoStartGateInMap(String), // Name of start gate
    NoStopGateInMap(String),  // Name of stop gate
    InvalidCharInMap(char),
    FoundLoneGatesInMap(HashMap<String, Position>), // Map of lone gates(name, position).
    NoPositionForGate(String, Position), // (name of gate, position of the first character in gate's name).
    NoPathToWalkThroughMaze,
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    North,
    South,
    West,
    East,
}

impl Direction {
    pub fn reverse(&self) -> Self {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
            Direction::East => Direction::West,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Position(usize, usize);

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x(), self.y())
    }
}

impl Position {
    pub fn new(x: usize, y: usize) -> Self {
        Self(x, y)
    }

    pub fn x(&self) -> usize {
        self.0
    }

    pub fn y(&self) -> usize {
        self.1
    }

    pub fn r(&self) -> usize {
        self.1
    }

    pub fn c(&self) -> usize {
        self.0
    }

    pub fn move_along(&self, dir: Direction) -> Option<Self> {
        match dir {
            Direction::North if self.y() > 0 => Some(Self::new(self.x(), self.y() - 1)),
            Direction::South => Some(Self::new(self.x(), self.y() + 1)),
            Direction::West if self.x() > 0 => Some(Self::new(self.x() - 1, self.y())),
            Direction::East => Some(Self::new(self.x() + 1, self.y())),
            _ => None,
        }
    }

    pub fn remove_offset(&self, x_offset: usize, y_offset: usize) -> Option<Self> {
        if self.x() >= x_offset && self.y() >= y_offset {
            Some(Self::new(self.x() - x_offset, self.y() - y_offset))
        } else {
            None
        }
    }
}

pub fn read_maze<P: AsRef<Path>>(
    path: P,
    start_gate_name: &str,
    stop_gate_name: &str,
) -> Result<PlainMaze, Error> {
    let input_file = File::open(path).map_err(Error::IOError)?;
    let reader = BufReader::new(input_file);
    let map_char_rows = reader
        .lines()
        .map(|l| {
            l.map_err(Error::IOError)
                .map(|s| s.chars().collect::<Vec<_>>())
        })
        .collect::<Result<Vec<_>, Error>>()?;

    let mut builder = MazeMapBuilder::new();
    let mut visited_name_pos = HashSet::new();
    let mut gates_pos = HashMap::new();
    let mut gate_pairs_pos = HashMap::new();
    let mut maze_row_offset = None;
    for (r_ind, row) in map_char_rows.iter().enumerate() {
        builder.new_row()?;
        for (c_ind, c) in row.iter().enumerate() {
            match *c {
                tc @ ('#' | '.') => {
                    maze_row_offset.get_or_insert(r_ind);
                    if tc == '#' {
                        builder.put_tile(TileType::Wall)?;
                    } else {
                        // '.'
                        builder.put_tile(TileType::Empty)?;
                    }
                }
                ' ' => builder.put_hole()?,
                a if a.is_alphabetic() => {
                    builder.put_hole()?;
                    let name_first_pos = Position::new(c_ind, r_ind);
                    if !visited_name_pos.contains(&name_first_pos) {
                        visited_name_pos.insert(name_first_pos.clone());
                        let (name, pos) =
                            find_gate(&map_char_rows, &name_first_pos, &mut visited_name_pos)?;
                        if let Some(last_pos) = gates_pos.remove(&name) {
                            // Found another gate with the same name, that is, found the warp pairs.
                            // Move tow positions to map for pairs.
                            gate_pairs_pos.insert(name, (last_pos, pos));
                        } else {
                            // Record the first gate position.
                            gates_pos.insert(name, pos);
                        }
                    }
                }
                other => return Err(Error::InvalidCharInMap(other)),
            }
        }
    }
    let map = builder.build()?;
    let maze_row_offset = maze_row_offset.ok_or(Error::NoMapInGivenInput)?;
    let start_pos = gates_pos
        .remove(start_gate_name)
        .ok_or(Error::NoStartGateInMap(start_gate_name.to_string()))?
        .remove_offset(0, maze_row_offset)
        .unwrap();
    let stop_pos = gates_pos
        .remove(stop_gate_name)
        .ok_or(Error::NoStopGateInMap(stop_gate_name.to_string()))?
        .remove_offset(0, maze_row_offset)
        .unwrap();
    let mut maze = PlainMaze::new(map, &start_pos, start_gate_name, &stop_pos, stop_gate_name);
    if !gates_pos.is_empty() {
        return Err(Error::FoundLoneGatesInMap(gates_pos));
    }

    // Add gate pairs
    for (name, (pos0, pos1)) in gate_pairs_pos {
        let pos0 = pos0.remove_offset(0, maze_row_offset).unwrap();
        let pos1 = pos1.remove_offset(0, maze_row_offset).unwrap();
        *maze.tile_mut(&MazePosition::new(&pos0, 0)).unwrap() =
            TileType::Gate(name.clone(), Some(pos1.clone()));
        *maze.tile_mut(&MazePosition::new(&pos1, 0)).unwrap() = TileType::Gate(name, Some(pos0));
    }

    Ok(maze)
}

pub fn find_shortest_path(maze: &dyn Maze) -> Result<Vec<MazeStep>, Error> {
    let init_walker = MazeWalker::new(maze.start_pos());
    let mut visited_walker_states = HashSet::new();
    let mut walkers = BinaryHeap::new();
    walkers.push(Reverse(init_walker));

    // Dijkstra's search for the shortest path to walk through given maze.
    while let Some(Reverse(walker)) = walkers.pop() {
        if !visited_walker_states.contains(walker.state()) {
            visited_walker_states.insert(walker.state().clone());
        } else {
            // This walker is at some visited state, ignore it.
            continue;
        }

        if walker.pos() == maze.stop_pos() {
            // The first walker reaches the stop position has one of the shortest paths.
            return Ok(walker.path().clone());
        }

        // println!("Search walker(at {}, walked {} steps).", walker.state().pos, walker.path().len());
        let reachable_gates_pos = walker.search_gates(maze);
        for gate_pos in reachable_gates_pos {
            let mut future_walker = walker.clone();
            future_walker.move_and_warp(maze, &gate_pos);
            // println!("Add future walker(at {}, walked {} steps) for searching.", future_walker.state().pos, future_walker.path().len());
            walkers.push(Reverse(future_walker));
        }
    }

    Err(Error::NoPathToWalkThroughMaze)
}

fn find_gate(
    map_char_rows: &Vec<Vec<char>>,
    pos: &Position,
    visited_name_pos: &mut HashSet<Position>,
) -> Result<(String, Position), Error> {
    let first_name_char = map_char_rows[pos.r()][pos.c()];
    assert!(first_name_char.is_alphabetic());

    let mut name = String::new();
    // Decide the direction to which the name extends
    let extend_dir = [Direction::East, Direction::South]
        .iter()
        .map(|dir| {
            (
                *dir,
                pos.move_along(*dir)
                    .and_then(|p| map_char_rows.get(p.r()).and_then(|chars| chars.get(p.c()))),
            )
        })
        .filter(|(_, c_op)| c_op.is_some_and(|c| c.is_alphabetic() || *c == '.'))
        .map(|(dir, _)| dir)
        .next();

    let mut cur_pos = pos.clone();
    if let Some(extend_dir) = extend_dir {
        while let Some(name_char) = map_char_rows
            .get(cur_pos.r())
            .and_then(|chars| chars.get(cur_pos.c()))
        {
            if !name_char.is_alphabetic() {
                break;
            }

            visited_name_pos.insert(cur_pos.clone());
            name.push(*name_char);
            cur_pos = cur_pos.move_along(extend_dir).unwrap();
        }
    }

    // Three positions left to find where this gate is in map, find the first one has a '.' in map.
    [
        pos.move_along(Direction::North),
        pos.move_along(Direction::West),
        Some(cur_pos.clone()),
    ]
    .iter()
    .flat_map(|op| op)
    .filter(|p| map_char_rows[p.r()][p.c()] == '.')
    .next()
    .ok_or(Error::NoPositionForGate(name.clone(), pos.clone()))
    .map(|p| (name, p.clone()))
}
