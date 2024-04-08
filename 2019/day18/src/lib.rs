use std::{
    cell::RefCell,
    cmp::Reverse,
    collections::{BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList},
    fmt::Display,
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
    rc::Rc,
};

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    InconsistentRowInMap(usize, usize),
    MultipleEntrancesInMap(usize, usize),
    InvalidMapChar(char),
    NoEntranceInMap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileType {
    Empty,
    Wall,
    Key(char),
    Door(char),
}

impl TryFrom<char> for TileType {
    type Error = Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '#' => Ok(Self::Wall),
            '.' => Ok(Self::Empty),
            c if value.is_ascii_uppercase() => Ok(Self::Door(c)),
            c if value.is_ascii_lowercase() => Ok(Self::Key(c)),
            c => Err(Error::InvalidMapChar(c)),
        }
    }
}

impl TileType {
    pub fn key_of_door(&self) -> Option<char> {
        if let Self::Door(c) = self {
            Some(c.to_ascii_lowercase())
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

    pub fn move_along(&self, dir: Direction) -> Option<Self> {
        match dir {
            Direction::North if self.y() > 0 => Some(Position::new(self.x(), self.y() - 1)),
            Direction::South => Some(Position::new(self.x(), self.y() + 1)),
            Direction::West if self.x() > 0 => Some(Position::new(self.x() - 1, self.y())),
            Direction::East => Some(Position::new(self.x() + 1, self.y())),
            _ => None,
        }
    }
}

pub struct VaultMap {
    map: Vec<TileType>,
    row_n: usize,
    col_n: usize,
    entrance_ind: usize,
}

impl VaultMap {
    pub fn try_from_lines<'a, I: Iterator<Item = &'a str>>(iter: I) -> Result<Self, Error> {
        let mut map = Vec::new();
        let mut row_n = 0;
        let mut entrance_ind = None;
        let mut col_n = None;
        for l in iter {
            let this_col_n = l.len();
            if *col_n.get_or_insert(this_col_n) != this_col_n {
                return Err(Error::InconsistentRowInMap(this_col_n, col_n.unwrap()));
            }

            for (ind, c) in l.chars().enumerate() {
                if c == '@' {
                    let ind = row_n * col_n.unwrap_or(0) + ind;
                    if *entrance_ind.get_or_insert(ind) != ind {
                        return Err(Error::MultipleEntrancesInMap(ind, entrance_ind.unwrap()));
                    }
                    map.push(TileType::Empty);
                } else {
                    map.push(TileType::try_from(c)?);
                }
            }
            row_n += 1;
        }

        if entrance_ind.is_none() {
            return Err(Error::NoEntranceInMap);
        }

        Ok(Self {
            map,
            row_n,
            col_n: col_n.unwrap_or(0),
            entrance_ind: entrance_ind.unwrap(),
        })
    }

    pub fn entrance_pos(&self) -> Position {
        self.ind_to_pos(self.entrance_ind)
    }

    pub fn can_pass(&self, pos: &Position) -> bool {
        *self.tile(pos) != TileType::Wall
    }

    pub fn tile(&self, pos: &Position) -> &TileType {
        &self.map[self.pos_to_ind(pos)]
    }

    fn ind_to_pos(&self, ind: usize) -> Position {
        assert!(ind < self.row_n * self.col_n);
        Position::new(ind % self.col_n, ind / self.col_n)
    }

    fn pos_to_ind(&self, pos: &Position) -> usize {
        assert!(pos.x() < self.col_n && pos.y() < self.row_n);
        pos.y() * self.col_n + pos.x()
    }
}

type PathsToKeys = HashMap<char, (BTreeSet<char>, Vec<Direction>)>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CollectorState {
    pos: Position,
    hold_keys: BTreeSet<char>,
}

#[derive(Debug, Clone)]
struct Collector {
    state: CollectorState,
    path: Vec<Direction>,
    global_paths_to_keys: Rc<RefCell<HashMap<Position, PathsToKeys>>>,
}

impl PartialEq for Collector {
    fn eq(&self, other: &Self) -> bool {
        self.steps_n() == other.steps_n()
    }
}

impl Eq for Collector {}

impl PartialOrd for Collector {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.steps_n().partial_cmp(&other.steps_n())
    }
}

impl Ord for Collector {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.steps_n().cmp(&other.steps_n())
    }
}

impl Display for Collector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "collector(at {}, has {:?}, {} steps)",
            self.state.pos,
            self.state.hold_keys,
            self.steps_n()
        )
    }
}

impl Collector {
    pub fn new(pos: &Position) -> Self {
        Self {
            state: CollectorState {
                pos: *pos,
                hold_keys: BTreeSet::new(),
            },
            path: Vec::new(),
            global_paths_to_keys: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn search_keys(&self, map: &VaultMap) -> LinkedList<char> {
        LinkedList::from_iter(
            self.global_paths_to_keys
                .borrow_mut()
                .entry(self.state.pos)
                // If paths from current position are not cached, do search and save.
                .or_insert_with(|| self.bfs_for_keys(map))
                .iter()
                .filter(|(key, (keys_needed, _))| {
                    // Keys are not hold now, and are reachable with current state(from current position, hold current keys).
                    !self.state.hold_keys.contains(key)
                        && self.state.hold_keys.is_superset(keys_needed)
                })
                .map(|(key, _)| *key),
        )
    }

    pub fn steps_n(&self) -> usize {
        self.path.len()
    }

    pub fn path(&self) -> &Vec<Direction> {
        &self.path
    }

    pub fn state(&self) -> &CollectorState {
        &self.state
    }

    pub fn move_to_key(&mut self, key: char, map: &VaultMap) {
        // Assume the shortest path to this key from current position has been found in earlier search(bfs_for_keys).
        if let Some((keys_needed, path)) = self
            .global_paths_to_keys
            .borrow()
            .get(&self.state.pos)
            .and_then(|paths| paths.get(&key))
        {
            if !self.state.hold_keys.is_superset(keys_needed) {
                panic!("Collector(at {}, have {:?}) doesn't have enough keys({:?}) to reach given key({})", self.state.pos, self.state.hold_keys, keys_needed, key);
            }

            // Add the shortest path to given key to self path
            for dir in path {
                self.state.pos = self.state.pos.move_along(*dir).unwrap();
                assert!(map.can_pass(&self.state.pos));
                self.path.push(*dir);
            }
            assert!(*map.tile(&self.state.pos) == TileType::Key(key));
            // Hold the key when reach its position.
            self.state.hold_keys.insert(key);
        } else {
            panic!(
                "Given key({}) isn't reachable to this collector(at {}, have {:?})",
                key, self.state.pos, self.state.hold_keys
            );
        }
    }

    fn bfs_for_keys(&self, map: &VaultMap) -> PathsToKeys {
        let init_pos = self.state.pos;
        let mut positions = LinkedList::new();
        positions.push_back(init_pos);
        let mut paths = HashMap::new();
        let mut paths_to_keys = HashMap::new();

        // Breadth first search for shortest path to all reachable position from current position.
        while let Some(pos) = positions.pop_front() {
            // Positions can move to from current position, wasn't reached before, and can pass(not a barrier).
            let move_positions = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East,
            ]
            .iter()
            .flat_map(|dir| pos.move_along(*dir).map(|p| (p, *dir)))
            .filter(|(pos, _)| *pos != init_pos && !paths.contains_key(pos) && map.can_pass(pos))
            .collect::<Vec<_>>();

            for (move_pos, dir_to_move_pos) in move_positions {
                // Record positions for future search, and the final part of shortest path to these positions.
                paths.insert(move_pos, dir_to_move_pos);
                positions.push_back(move_pos);

                // If a position has a key in there, backtrack and save the shortest path to this key.
                if let TileType::Key(k) = map.tile(&move_pos) {
                    let mut path = Vec::new();
                    let mut cur_pos = pos;
                    let mut keys_needed = BTreeSet::new();
                    while let Some(dir_to_cur_pos) = paths.get(&cur_pos) {
                        path.push(*dir_to_cur_pos);
                        if let Some(key) = map.tile(&cur_pos).key_of_door() {
                            // If a door sit in the middle of the shortest path, record its key as a part of path's requirement(keys needed)
                            debug_assert!(!keys_needed.contains(&key));
                            keys_needed.insert(key);
                        }
                        cur_pos = cur_pos.move_along(dir_to_cur_pos.reverse()).unwrap();
                    }
                    path.reverse();
                    path.push(dir_to_move_pos);

                    paths_to_keys.insert(*k, (keys_needed, path));
                }
            }
        }

        paths_to_keys
    }
}

pub fn read_vault_map<P: AsRef<Path>>(path: P) -> Result<VaultMap, Error> {
    let input_file = File::open(path).map_err(Error::IOError)?;
    let reader = BufReader::new(input_file);
    let map_lines = reader
        .lines()
        .map(|l| l.map_err(Error::IOError))
        .collect::<Result<Vec<_>, Error>>()?;

    VaultMap::try_from_lines(map_lines.iter().map(|s| s.as_str()))
}

pub fn find_shortest_collect_path(map: &VaultMap) -> Vec<Direction> {
    let init_collector = Collector::new(&map.entrance_pos());
    let mut collectors = BinaryHeap::new();
    let mut searched_collector_states = HashSet::new();
    collectors.push(Reverse(init_collector));

    // Dijkstra's search for the shortest path to collect all keys. The states of collectors are nodes in graph.
    while let Some(Reverse(collector)) = collectors.pop() {
        if searched_collector_states.contains(collector.state()) {
            // Ignore collectors with state has been searched
            continue;
        }

        searched_collector_states.insert(collector.state().clone());
        let reachable_keys = collector.search_keys(map);
        if reachable_keys.is_empty() {
            // The first completed path is one of the shortest path.
            return collector.path().clone();
        }

        for key in reachable_keys {
            // Move one cloned collector to each reachable keys, and record them for future search.
            let mut future_collector = collector.clone();
            future_collector.move_to_key(key, map);
            collectors.push(Reverse(future_collector));
        }
    }

    unreachable!("No path found for collecting all keys, some error may occur in given map.");
}
