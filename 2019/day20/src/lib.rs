use std::{
    cell::RefCell,
    cmp::Reverse,
    collections::{BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList},
    fmt::Display,
    fs::File,
    io::{self, BufRead, BufReader},
    ops::Range,
    path::Path,
    rc::Rc,
};

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

#[derive(Debug, PartialEq, Eq)]
pub enum TileType {
    Wall,
    Empty,
    Gate(String, Option<Position>), // (name of gate, warp position)
}

impl TileType {
    pub fn can_pass(&self) -> bool {
        match self {
            TileType::Wall => false,
            _ => true,
        }
    }
}

#[derive(Debug)]
struct MazeRange {
    range: Range<usize>,
    start_ind: Option<usize>, // Some, it's a range of tiles, the value inside is the start index of this range in tiles of MazeRow; None, it's a range of hole.
}

impl MazeRange {
    pub fn new(range: Range<usize>, start_ind: Option<usize>) -> Self {
        Self { range, start_ind }
    }
}

#[derive(Debug)]
struct MazeRow {
    tiles: Vec<TileType>,
    ranges: Vec<MazeRange>,
}

impl Display for MazeRow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let row_str = self
            .ranges
            .iter()
            .flat_map(|r| {
                (0..(r.range.len())).map(|ind| {
                    r.start_ind
                        .map(|start_ind| match self.tiles[ind + start_ind] {
                            TileType::Wall => '#',
                            TileType::Empty => '.',
                            TileType::Gate(_, _) => 'O',
                        })
                        .unwrap_or(' ')
                })
            })
            .collect::<String>();

        write!(f, "{}", row_str)
    }
}

impl MazeRow {
    pub fn new() -> Self {
        Self {
            tiles: Vec::new(),
            ranges: Vec::new(),
        }
    }

    pub fn tile(&self, ind: usize) -> Option<&TileType> {
        if self.is_empty() || ind >= self.ranges[self.ranges.len() - 1].range.end {
            // Out of ranges of this row.
            None
        } else {
            self.ranges
                .iter()
                .filter(|mr| mr.range.contains(&ind))
                .next()
                .and_then(|mr| {
                    mr.start_ind.and_then(|tiles_start_ind| {
                        self.tiles.get(tiles_start_ind + ind - mr.range.start)
                    })
                })
        }
    }

    pub fn tile_mut(&mut self, ind: usize) -> Option<&mut TileType> {
        if self.is_empty() || ind >= self.ranges[self.ranges.len() - 1].range.end {
            // Out of ranges of this row.
            None
        } else {
            self.ranges
                .iter()
                .filter(|mr| mr.range.contains(&ind))
                .next()
                .and_then(|mr| {
                    mr.start_ind.and_then(|tiles_start_ind| {
                        self.tiles.get_mut(tiles_start_ind + ind - mr.range.start)
                    })
                })
        }
    }

    pub fn is_empty(&self) -> bool {
        if self.tiles.is_empty() {
            debug_assert!(self.ranges.is_empty());
        }

        self.tiles.is_empty()
    }
}

type MazeMap = Vec<MazeRow>;

#[derive(Debug)]
pub struct Maze {
    map: MazeMap,
    start_pos: Position,
    stop_pos: Position,
}

impl Display for Maze {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in &self.map {
            writeln!(f, "{}", row)?;
        }

        Ok(())
    }
}

impl Maze {
    fn new(
        map: MazeMap,
        start_pos: &Position,
        start_name: &str,
        stop_pos: &Position,
        stop_name: &str,
    ) -> Self {
        let mut maze = Self {
            map,
            start_pos: start_pos.clone(),
            stop_pos: stop_pos.clone(),
        };
        assert!(maze
            .tile(start_pos)
            .is_some_and(|tt| *tt == TileType::Empty));
        assert!(maze.tile(stop_pos).is_some_and(|tt| *tt == TileType::Empty));
        *maze.tile_mut(start_pos).unwrap() = TileType::Gate(start_name.to_string(), None);
        *maze.tile_mut(stop_pos).unwrap() = TileType::Gate(stop_name.to_string(), None);

        maze
    }

    pub fn tile(&self, pos: &Position) -> Option<&TileType> {
        self.map.get(pos.r()).and_then(|row| row.tile(pos.c()))
    }

    pub fn tile_mut(&mut self, pos: &Position) -> Option<&mut TileType> {
        self.map
            .get_mut(pos.r())
            .and_then(|row| row.tile_mut(pos.c()))
    }

    pub fn start_pos(&self) -> &Position {
        &self.start_pos
    }

    pub fn stop_pos(&self) -> &Position {
        &self.stop_pos
    }

    pub fn can_pass(&self, pos: &Position) -> bool {
        self.tile(pos).is_some_and(|tt| tt.can_pass())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum MazeMapBuilderState {
    Idle,
    PuttingTile(Range<usize>),
    PuttingHole(Range<usize>),
}

impl Display for MazeMapBuilderState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MazeMapBuilderState::Idle => write!(f, "Idle"),
            MazeMapBuilderState::PuttingTile(r) => write!(f, "PuttingTile({:?})", r),
            MazeMapBuilderState::PuttingHole(r) => write!(f, "PuttingHole({:?})", r),
        }
    }
}

#[derive(Debug)]
struct MazeMapBuilder {
    map: MazeMap,
    building_row: Option<MazeRow>,
    state: MazeMapBuilderState,
}

impl MazeMapBuilder {
    pub fn new() -> Self {
        Self {
            map: Vec::new(),
            building_row: None,
            state: MazeMapBuilderState::Idle,
        }
    }

    pub fn new_row(&mut self) -> Result<(), Error> {
        self.end_row()?;
        self.building_row = Some(MazeRow::new());
        self.state = MazeMapBuilderState::PuttingTile(0..0);

        Ok(())
    }

    pub fn put_tile(&mut self, tt: TileType) -> Result<(), Error> {
        match self.state.clone() {
            s @ (MazeMapBuilderState::PuttingTile(_) | MazeMapBuilderState::PuttingHole(_)) => {
                let row_mut = self
                    .building_row
                    .as_mut()
                    .expect("No row is building when state is putting");
                row_mut.tiles.push(tt);
                match s {
                    MazeMapBuilderState::PuttingTile(r) => {
                        self.state = MazeMapBuilderState::PuttingTile((r.start)..(r.end + 1));
                    }
                    MazeMapBuilderState::PuttingHole(r) => {
                        self.state = MazeMapBuilderState::PuttingTile((r.end)..(r.end + 1));
                        row_mut.ranges.push(MazeRange::new(r.clone(), None));
                    }
                    _ => unreachable!(),
                }
            }
            other => {
                return Err(Error::InvalidMapBuilderState(
                    other.to_string(),
                    "put_tile".to_string(),
                ))
            }
        }

        Ok(())
    }

    pub fn put_hole(&mut self) -> Result<(), Error> {
        match self.state.clone() {
            MazeMapBuilderState::PuttingTile(r) => {
                self.end_tiles_range();
                self.state = MazeMapBuilderState::PuttingHole((r.end)..(r.end + 1));
            }
            MazeMapBuilderState::PuttingHole(r) => {
                self.state = MazeMapBuilderState::PuttingHole((r.start)..(r.end + 1));
            }
            other => {
                return Err(Error::InvalidMapBuilderState(
                    other.to_string(),
                    "put_tile".to_string(),
                ))
            }
        }

        Ok(())
    }

    pub fn build(mut self) -> Result<MazeMap, Error> {
        if self.state != MazeMapBuilderState::Idle {
            self.end_row()?;
        }

        // Discards empty rows at bottom.
        for ind in (0..(self.map.len())).rev() {
            if self.map[ind].is_empty() {
                self.map.pop();
            } else {
                break;
            }
        }

        Ok(self.map)
    }

    fn end_row(&mut self) -> Result<(), Error> {
        match self.state.clone() {
            s @ (MazeMapBuilderState::PuttingTile(_) | MazeMapBuilderState::PuttingHole(_)) => {
                if let MazeMapBuilderState::PuttingTile(_) = s {
                    // Ignore the last hole range in row, only process range of tiles
                    self.end_tiles_range();
                }
                let row = self
                    .building_row
                    .take()
                    .expect("No row is building when state is putting");
                // Discard empty rows at top
                if !self.map.is_empty() || !row.is_empty() {
                    self.map.push(row);
                }
            }
            _ => (),
        }

        Ok(())
    }

    fn end_tiles_range(&mut self) {
        if let MazeMapBuilderState::PuttingTile(r) = self.state.clone() {
            if !r.is_empty() {
                // Only process non-empty range of tiles
                let row_mut = self
                    .building_row
                    .as_mut()
                    .expect("No row is building when state is putting");
                let tiles_start_ind = row_mut.tiles.len() - r.len();
                row_mut
                    .ranges
                    .push(MazeRange::new(r, Some(tiles_start_ind)));
            }
        } else {
            panic!(
                "No tiles range to end, the current state is {:?}, expects PuttingTile.",
                self.state
            )
        }
    }
}

#[derive(Debug, Clone)]
pub enum MazeStep {
    Move(Direction),
    Warp(Position),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct MazeWalkerState {
    pos: Position,
    warped_pos: BTreeSet<Position>,
}

#[derive(Debug, Clone)]
struct MazeWalker {
    state: MazeWalkerState,
    path: Vec<MazeStep>,
    visited_gates_pos: HashSet<Position>,
    global_paths_for_gates: Rc<RefCell<HashMap<Position, HashMap<Position, Vec<MazeStep>>>>>,
}

impl PartialOrd for MazeWalker {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.path.len().partial_cmp(&other.path.len())
    }
}

impl Ord for MazeWalker {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.path.len().cmp(&other.path.len())
    }
}

impl PartialEq for MazeWalker {
    fn eq(&self, other: &Self) -> bool {
        self.path.len() == other.path.len()
    }
}

impl Eq for MazeWalker {}

impl MazeWalker {
    pub fn new(start_pos: &Position) -> Self {
        Self {
            state: MazeWalkerState {
                pos: start_pos.clone(),
                warped_pos: BTreeSet::new(),
            },
            path: Vec::new(),
            visited_gates_pos: HashSet::new(),
            global_paths_for_gates: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn state(&self) -> &MazeWalkerState {
        &self.state
    }

    pub fn path(&self) -> &Vec<MazeStep> {
        &self.path
    }

    pub fn pos(&self) -> &Position {
        &self.state.pos
    }

    pub fn search_gates(&self, maze: &Maze) -> Vec<Position> {
        self.global_paths_for_gates
            .borrow_mut()
            .entry(self.state.pos.clone())
            .or_insert_with(|| self.bfs_for_gates(maze))
            .iter()
            .filter(|(pos, _)| !self.visited_gates_pos.contains(pos))
            .map(|(pos, _)| pos.clone())
            .collect()
    }

    pub fn move_and_warp(&mut self, maze: &Maze, pos: &Position) {
        if let Some(TileType::Gate(_, warp_pos)) = maze.tile(pos) {
            if let Some(path) = self
                .global_paths_for_gates
                .borrow()
                .get(self.pos())
                .and_then(|map| map.get(pos))
            {
                let mut cur_pos = self.state.pos.clone();
                for step in path {
                    let next_pos = match step {
                        MazeStep::Move(dir) => cur_pos.move_along(*dir),
                        MazeStep::Warp(_) => panic!("Warp isn't allowed in BFS path"),
                    };
                    if let Some(next_pos) = next_pos {
                        assert!(maze.can_pass(&next_pos));
                        self.path.push(step.clone());
                        cur_pos = next_pos;
                    } else {
                        panic!("Searched path(from {} to {}) arrives at invalid position, position before it is {}.", self.pos(), pos, cur_pos);
                    }
                }

                assert!(cur_pos == *pos);
                self.state.pos = if let Some(warp_pos) = warp_pos {
                    // Do warp
                    self.state.warped_pos.insert(pos.clone());
                    self.visited_gates_pos.insert(warp_pos.clone());
                    self.path.push(MazeStep::Warp(warp_pos.clone()));
                    warp_pos.clone()
                } else {
                    cur_pos
                };
                self.visited_gates_pos.insert(pos.clone());
            } else {
                panic!(
                    "No Known path from {} to given position({})",
                    self.pos(),
                    pos
                );
            }
        } else {
            panic!("Given position({}) in maze isn't a gate", pos);
        }
    }

    fn bfs_for_gates(&self, maze: &Maze) -> HashMap<Position, Vec<MazeStep>> {
        let mut positions = LinkedList::new();
        positions.push_back(self.pos().clone());
        let mut shortest_path_steps = HashMap::new();
        let mut paths_for_gates = HashMap::new();

        while let Some(pos) = positions.pop_front() {
            for (dir, next_pos) in [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East,
            ]
            .iter()
            .flat_map(|dir| pos.move_along(*dir).map(|pos| (*dir, pos)))
            .filter(|(_, pos)| pos != self.pos() && maze.can_pass(pos))
            {
                if shortest_path_steps.contains_key(&next_pos) {
                    continue;
                }

                shortest_path_steps.insert(next_pos.clone(), dir);
                positions.push_back(next_pos.clone());
                if let Some(TileType::Gate(_, _)) = maze.tile(&next_pos) {
                    // Backtrace and save path to gate
                    let mut path_to_gate = Vec::new();
                    let mut cur_pos = next_pos.clone();
                    while cur_pos != *self.pos() {
                        let dir = shortest_path_steps
                            .get(&cur_pos)
                            .expect(&format!("Broken path in BFS at {}.", cur_pos));
                        cur_pos = cur_pos
                            .move_along(dir.reverse())
                            .expect("Move to boundry of maze in BFS path.");
                        path_to_gate.push(MazeStep::Move(*dir));
                    }

                    path_to_gate.reverse();
                    paths_for_gates.insert(next_pos, path_to_gate);
                }
            }
        }

        paths_for_gates
    }
}

pub fn read_maze<P: AsRef<Path>>(
    path: P,
    start_gate_name: &str,
    stop_gate_name: &str,
) -> Result<Maze, Error> {
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
    let mut maze = Maze::new(map, &start_pos, start_gate_name, &stop_pos, stop_gate_name);
    if !gates_pos.is_empty() {
        return Err(Error::FoundLoneGatesInMap(gates_pos));
    }

    // Add gate pairs
    for (name, (pos0, pos1)) in gate_pairs_pos {
        let pos0 = pos0.remove_offset(0, maze_row_offset).unwrap();
        let pos1 = pos1.remove_offset(0, maze_row_offset).unwrap();
        *maze.tile_mut(&pos0).unwrap() = TileType::Gate(name.clone(), Some(pos1.clone()));
        *maze.tile_mut(&pos1).unwrap() = TileType::Gate(name, Some(pos0));
    }

    Ok(maze)
}

pub fn find_shortest_path(maze: &Maze) -> Result<Vec<MazeStep>, Error> {
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

        let reachable_gates_pos = walker.search_gates(maze);
        for gate_pos in reachable_gates_pos {
            let mut future_walker = walker.clone();
            future_walker.move_and_warp(maze, &gate_pos);
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
