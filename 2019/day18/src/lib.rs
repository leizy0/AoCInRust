use std::{
    cell::RefCell,
    cmp::Reverse,
    collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList},
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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
    entrance_ind: Vec<usize>,
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
            entrance_ind: vec![entrance_ind.unwrap()],
        })
    }

    pub fn entrance_n(&self) -> usize {
        self.entrance_ind.len()
    }

    pub fn entrance_pos(&self) -> Vec<Position> {
        self.entrance_ind
            .iter()
            .map(|ind| self.ind_to_pos(*ind))
            .collect()
    }

    pub fn can_pass(&self, pos: &Position) -> bool {
        *self.tile(pos) != TileType::Wall
    }

    pub fn tile(&self, pos: &Position) -> &TileType {
        &self.map[self.pos_to_ind(pos)]
    }

    pub fn tile_mut(&mut self, pos: &Position) -> &mut TileType {
        let ind = self.pos_to_ind(pos);
        &mut self.map[ind]
    }

    pub fn clear_entrance(&mut self) {
        self.entrance_ind.clear();
    }

    pub fn add_entrance(&mut self, pos: &Position) {
        assert!(*self.tile(pos) != TileType::Wall);
        let ind = self.pos_to_ind(pos);
        self.entrance_ind.push(ind);
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
struct CollectorTeamState {
    pos: BTreeMap<Position, usize>,
    hold_keys: BTreeSet<char>,
}

#[derive(Debug, Clone)]
struct CollectorTeam {
    members_pos: Vec<Position>,
    hold_keys: BTreeSet<char>,
    members_path: Vec<Vec<Direction>>,
    global_paths_to_keys: Rc<RefCell<HashMap<Position, PathsToKeys>>>,
}

impl PartialEq for CollectorTeam {
    fn eq(&self, other: &Self) -> bool {
        self.steps_n() == other.steps_n()
    }
}

impl Eq for CollectorTeam {}

impl PartialOrd for CollectorTeam {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.steps_n().partial_cmp(&other.steps_n())
    }
}

impl Ord for CollectorTeam {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.steps_n().cmp(&other.steps_n())
    }
}

impl Display for CollectorTeam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "collector(at {:?}, has {:?}, {} steps)",
            self.members_pos,
            self.hold_keys,
            self.steps_n()
        )
    }
}

impl CollectorTeam {
    pub fn new(pos: &[Position]) -> Self {
        Self {
            members_pos: Vec::from(pos),
            hold_keys: BTreeSet::new(),
            members_path: vec![Vec::new(); pos.len()],
            global_paths_to_keys: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn search_keys(&self, map: &VaultMap) -> Vec<LinkedList<char>> {
        self.members_pos
            .iter()
            .map(|pos| {
                LinkedList::from_iter(
                    self.global_paths_to_keys
                        .borrow_mut()
                        .entry(*pos)
                        // If paths from current position are not cached, do search and save.
                        .or_insert_with(|| Self::bfs_for_keys(pos, map))
                        .iter()
                        .filter(|(key, (keys_needed, _))| {
                            // Keys are not hold now, and are reachable with current state(from current position, hold current keys).
                            !self.hold_keys.contains(key) && self.hold_keys.is_superset(keys_needed)
                        })
                        .map(|(key, _)| *key),
                )
            })
            .collect()
    }

    pub fn steps_n(&self) -> usize {
        self.members_path.iter().map(|p| p.len()).sum()
    }

    pub fn path(&self) -> &Vec<Vec<Direction>> {
        &self.members_path
    }

    pub fn state(&self) -> CollectorTeamState {
        let mut pos = BTreeMap::new();
        for p in &self.members_pos {
            *pos.entry(*p).or_insert(0) += 1;
        }

        CollectorTeamState {
            pos,
            hold_keys: self.hold_keys.clone(),
        }
    }

    pub fn move_mem_to_key(&mut self, member_ind: usize, key: char, map: &VaultMap) {
        assert!(member_ind < self.members_pos.len());
        // Assume the shortest path to this key from gien member's position has been found in earlier search(bfs_for_keys).
        if let Some((keys_needed, path)) = self
            .global_paths_to_keys
            .borrow()
            .get(&self.members_pos[member_ind])
            .and_then(|paths| paths.get(&key))
        {
            if !self.hold_keys.is_superset(keys_needed) {
                panic!("Collector member(at {}, have {:?}) doesn't have enough keys({:?}) to reach given key({})", self.members_pos[member_ind], self.hold_keys, keys_needed, key);
            }

            // Add the shortest path to given key to member's path
            let member_pos = &mut self.members_pos[member_ind];
            let member_path = &mut self.members_path[member_ind];
            for dir in path {
                *member_pos = member_pos.move_along(*dir).unwrap();
                assert!(map.can_pass(member_pos));
                member_path.push(*dir);
            }
            assert!(*map.tile(&member_pos) == TileType::Key(key));
            // Hold the key when reach its position.
            self.hold_keys.insert(key);
        } else {
            panic!(
                "Given key({}) isn't reachable to this collector(at {}, have {:?})",
                key, self.members_pos[member_ind], self.hold_keys
            );
        }
    }

    fn bfs_for_keys(pos: &Position, map: &VaultMap) -> PathsToKeys {
        let init_pos = *pos;
        let mut positions = LinkedList::new();
        positions.push_back(init_pos);
        let mut paths = HashMap::new();
        let mut paths_to_keys = HashMap::new();

        // Breadth first search for shortest path to all reachable position from given position.
        while let Some(pos) = positions.pop_front() {
            // Positions can move to from current position, wasn't reached before, and collector can pass it(not a barrier).
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

pub fn split_entrance(map: &mut VaultMap) {
    let entrances = map.entrance_pos();
    assert!(entrances.len() == 1);

    // Change position of entrance to wall.
    let entrance_pos = entrances[0];
    *map.tile_mut(&entrance_pos) = TileType::Wall;
    // Change four neighbors of entrance to wall too.
    for pos in [
        Direction::North,
        Direction::South,
        Direction::West,
        Direction::East,
    ]
    .iter()
    .flat_map(|dir| entrance_pos.move_along(*dir))
    {
        *map.tile_mut(&pos) = TileType::Wall;
    }

    // Replace the entrance to new entrances at four diagonal positions of the original one.
    map.clear_entrance();
    map.add_entrance(&Position::new(entrance_pos.x() - 1, entrance_pos.y() - 1));
    map.add_entrance(&Position::new(entrance_pos.x() - 1, entrance_pos.y() + 1));
    map.add_entrance(&Position::new(entrance_pos.x() + 1, entrance_pos.y() - 1));
    map.add_entrance(&Position::new(entrance_pos.x() + 1, entrance_pos.y() + 1));
}

pub fn find_shortest_collect_path(map: &VaultMap) -> Vec<Vec<Direction>> {
    let init_team = CollectorTeam::new(&map.entrance_pos());
    let mut collector_teams = BinaryHeap::new();
    let mut searched_team_states = HashSet::new();
    collector_teams.push(Reverse(init_team));

    // Dijkstra's search for the shortest path to collect all keys. The states of collector teams are nodes in graph.
    while let Some(Reverse(team)) = collector_teams.pop() {
        let state = team.state();
        if searched_team_states.contains(&state) {
            // Ignore collector teams with state has been searched
            continue;
        }

        searched_team_states.insert(state);
        let team_reachable_keys = team.search_keys(map);
        if team_reachable_keys
            .iter()
            .all(|member_reachable_keys| member_reachable_keys.is_empty())
        {
            // The first completed path is one of the shortest path.
            return team.path().clone();
        }

        for (member_ind, member_reachable_keys) in team_reachable_keys.iter().enumerate() {
            for key in member_reachable_keys {
                // Move one cloned collector team to each reachable keys, and record them for future search.
                let mut future_team = team.clone();
                future_team.move_mem_to_key(member_ind, *key, map);
                collector_teams.push(Reverse(future_team));
            }
        }
    }

    unreachable!("No path found for collecting all keys, some error may occur in given map.");
}
