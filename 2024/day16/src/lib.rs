use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap, HashSet, LinkedList},
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
    MultipleStartPosition(Position, Position),
    MultipleEndPosition(Position, Position),
    InvalidCharForMap(char),
    NoStartPosition,
    NoEndPosition,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InconsistentRow(expect_col_n, this_col_n) => write!(
                f,
                "Expect {} column(s) in each row, given {}.",
                expect_col_n, this_col_n
            ),
            Error::MultipleStartPosition(last_pos, pos) => write!(
                f,
                "Expect only one start position, given two({}, {}).",
                last_pos, pos
            ),
            Error::MultipleEndPosition(last_pos, pos) => write!(
                f,
                "Expect only one end position, given two({}, {}).",
                last_pos, pos
            ),
            Error::InvalidCharForMap(c) => write!(f, "Invalid character({}) for map.", c),
            Error::NoStartPosition => write!(f, "No start position in map."),
            Error::NoEndPosition => write!(f, "No end position in map."),
        }
    }
}

impl error::Error for Error {}

#[derive(Debug, Parser)]
pub struct CLIArgs {
    pub input_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Position {
    r: usize,
    c: usize,
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
            Direction::North if self.r > 0 => Some(Self::new(self.r - 1, self.c)),
            Direction::East => Some(Self::new(self.r, self.c + 1)),
            Direction::South => Some(Self::new(self.r + 1, self.c)),
            Direction::West if self.c > 0 => Some(Self::new(self.r, self.c - 1)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    Wall,
    Floor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    Forward,
    TurnClockwise,
    TurnCounterclockwise,
}

impl Action {
    pub fn all_actions() -> &'static [Action] {
        static ALL_ACTIONS: [Action; 3] = [
            Action::Forward,
            Action::TurnClockwise,
            Action::TurnCounterclockwise,
        ];

        &ALL_ACTIONS
    }

    pub fn score(&self) -> usize {
        match self {
            Action::Forward => 1,
            Action::TurnClockwise | Action::TurnCounterclockwise => 1000,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    pub fn turn_clockwise(&self) -> Self {
        match self {
            Direction::North => Direction::East,
            Direction::East => Direction::South,
            Direction::South => Direction::West,
            Direction::West => Direction::North,
        }
    }

    pub fn turn_counterclockwise(&self) -> Self {
        match self {
            Direction::North => Direction::West,
            Direction::East => Direction::North,
            Direction::South => Direction::East,
            Direction::West => Direction::South,
        }
    }

    pub fn reverse(&self) -> Self {
        match self {
            Direction::North => Direction::South,
            Direction::East => Direction::West,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Reindeer {
    pos: Position,
    dir: Direction,
}

impl Reindeer {
    pub fn new(pos: &Position, dir: Direction) -> Self {
        Self {
            pos: pos.clone(),
            dir,
        }
    }

    pub fn clone_and_do(&self, action: Action, map: &Map) -> Option<Reindeer> {
        match action {
            Action::Forward => {
                if let Some(next_pos) = self.pos.neighbor(self.dir) {
                    if map.tile(&next_pos).is_some_and(|tile| *tile == Tile::Floor) {
                        return Some(Self::new(&next_pos, self.dir));
                    }
                }

                None
            }
            Action::TurnClockwise => Some(Self::new(&self.pos, self.dir.turn_clockwise())),
            Action::TurnCounterclockwise => {
                Some(Self::new(&self.pos, self.dir.turn_counterclockwise()))
            }
        }
    }

    pub fn clone_and_do_reverse(&self, action: Action, map: &Map) -> Option<Reindeer> {
        match action {
            Action::Forward => {
                if let Some(next_pos) = self.pos.neighbor(self.dir.reverse()) {
                    if map.tile(&next_pos).is_some_and(|tile| *tile == Tile::Floor) {
                        return Some(Self::new(&next_pos, self.dir));
                    }
                }

                None
            }
            Action::TurnClockwise => Some(Self::new(&self.pos, self.dir.turn_counterclockwise())),
            Action::TurnCounterclockwise => Some(Self::new(&self.pos, self.dir.turn_clockwise())),
        }
    }
}

#[derive(Debug, Clone)]
struct State {
    deer: Reindeer,
    score: usize,
    src_action: Option<Action>,
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.score.cmp(&other.score)
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}

impl Eq for State {}

impl State {
    pub fn new(deer: Reindeer, score: usize) -> Self {
        Self {
            deer,
            score,
            src_action: None,
        }
    }

    pub fn from_action(deer: Reindeer, score: usize, action: Action) -> Self {
        Self {
            deer,
            score,
            src_action: Some(action),
        }
    }
}

#[derive(Debug)]
pub struct Map {
    tiles: Vec<Tile>,
    row_n: usize,
    col_n: usize,
    start_pos: Position,
    end_pos: Position,
}

impl Map {
    pub fn min_score_action_graph(&self) -> Option<(HashMap<Reindeer, HashSet<Action>>, usize)> {
        let mut min_score = None;
        let mut src_actions_of_deer = HashMap::from([(self.init_deer(), (0, HashSet::new()))]);
        let mut possible_states = BinaryHeap::from([Reverse(State::new(self.init_deer(), 0))]);
        let mut min_score_action_graph = None;
        while let Some(Reverse(cur_state)) = possible_states.pop() {
            if src_actions_of_deer
                .get(&cur_state.deer)
                .map(|(score, _)| cur_state.score <= *score)
                .unwrap_or(true)
            {
                if let Some(src_action) = cur_state.src_action {
                    if let Some(last_score) = src_actions_of_deer
                        .get(&cur_state.deer)
                        .map(|(score, _)| score)
                    {
                        debug_assert!(
                            *last_score == cur_state.score,
                            "last_score({}) != cur_score({})",
                            last_score,
                            cur_state.score,
                        );
                    }

                    src_actions_of_deer
                        .entry(cur_state.deer.clone())
                        .or_insert_with(|| (cur_state.score, HashSet::new()))
                        .1
                        .insert(src_action);
                }
            }

            if min_score.is_some_and(|min_score| cur_state.score > min_score) {
                break;
            }

            if cur_state.deer.pos == self.end_pos {
                min_score.get_or_insert(cur_state.score);
                let mut search_deers = LinkedList::from([cur_state.deer.clone()]);
                let mut searched_deers = HashSet::from([cur_state.deer.clone()]);
                while let Some(cur_deer) = search_deers.pop_front() {
                    if let Some((_, src_actions)) = src_actions_of_deer.get(&cur_deer) {
                        for action in src_actions {
                            let src_deer = cur_deer.clone_and_do_reverse(*action, self).unwrap();
                            min_score_action_graph
                                .get_or_insert_with(|| HashMap::new())
                                .entry(src_deer.clone())
                                .or_insert_with(|| HashSet::new())
                                .insert(*action);
                            if searched_deers.insert(src_deer.clone()) {
                                search_deers.push_back(src_deer);
                            }
                        }
                    }
                }
            }

            for action in Action::all_actions() {
                if let Some(next_deer) = cur_state.deer.clone_and_do(*action, self) {
                    let next_score = cur_state.score + action.score();
                    if src_actions_of_deer
                        .get(&next_deer)
                        .map(|(score, _)| next_score <= *score)
                        .unwrap_or(true)
                    {
                        if let Some(last_score) =
                            src_actions_of_deer.get(&next_deer).map(|(score, _)| score)
                        {
                            debug_assert!(
                                *last_score == next_score,
                                "last_score({}) != next_score({})",
                                last_score,
                                next_score
                            );
                        }

                        possible_states
                            .push(Reverse(State::from_action(next_deer, next_score, *action)));
                    }
                }
            }
        }

        min_score_action_graph.and_then(|graph| min_score.map(|score| (graph, score)))
    }

    pub fn pos_n_on_graph(&self, actions_graph: &HashMap<Reindeer, HashSet<Action>>) -> usize {
        let mut pos_on_path = HashSet::from([self.start_pos.clone()]);
        let mut search_deers = LinkedList::from([self.init_deer()]);
        let mut searched_deers = HashSet::from([self.init_deer()]);
        while let Some(cur_deer) = search_deers.pop_front() {
            if let Some(actions) = actions_graph.get(&cur_deer) {
                for action in actions {
                    if let Some(next_deer) = cur_deer.clone_and_do(*action, self) {
                        if searched_deers.insert(next_deer.clone()) {
                            pos_on_path.insert(next_deer.pos.clone());
                            search_deers.push_back(next_deer);
                        }
                    }
                }
            }
        }

        pos_on_path.len()
    }

    pub fn tile(&self, pos: &Position) -> Option<&Tile> {
        if pos.r < self.row_n && pos.c < self.col_n {
            self.tiles.get(pos.r * self.col_n + pos.c)
        } else {
            None
        }
    }

    fn init_deer(&self) -> Reindeer {
        Reindeer::new(&self.start_pos, Direction::East)
    }
}

#[derive(Debug)]
struct MapBuilder {
    tiles: Vec<Tile>,
    row_n: usize,
    col_n: Option<usize>,
    start_pos: Option<Position>,
    end_pos: Option<Position>,
}

impl MapBuilder {
    pub fn new() -> Self {
        Self {
            tiles: Vec::new(),
            row_n: 0,
            col_n: None,
            start_pos: None,
            end_pos: None,
        }
    }

    pub fn add_row(&mut self, text: &str) -> Result<(), Error> {
        let this_col_n = text.chars().count();
        if *self.col_n.get_or_insert(this_col_n) != this_col_n {
            return Err(Error::InconsistentRow(self.col_n.unwrap(), this_col_n));
        }

        for (ind, c) in text.chars().enumerate() {
            let pos = Position::new(self.row_n, ind);
            self.tiles.push(match c {
                'S' => {
                    if let Some(last_pos) = self.start_pos.as_ref().take() {
                        return Err(Error::MultipleStartPosition(last_pos.clone(), pos));
                    }

                    self.start_pos = Some(pos);
                    Tile::Floor
                }
                'E' => {
                    if let Some(last_pos) = self.end_pos.as_ref().take() {
                        return Err(Error::MultipleEndPosition(last_pos.clone(), pos));
                    }

                    self.end_pos = Some(pos);
                    Tile::Floor
                }
                '#' => Tile::Wall,
                '.' => Tile::Floor,
                other => return Err(Error::InvalidCharForMap(other)),
            });
        }
        self.row_n += 1;

        Ok(())
    }

    pub fn build(self) -> Result<Map, Error> {
        let Some(start_pos) = self.start_pos else {
            return Err(Error::NoStartPosition);
        };
        let Some(end_pos) = self.end_pos else {
            return Err(Error::NoEndPosition);
        };

        Ok(Map {
            tiles: self.tiles,
            row_n: self.row_n,
            col_n: self.col_n.unwrap_or(0),
            start_pos,
            end_pos,
        })
    }
}

pub fn read_map<P: AsRef<Path>>(path: P) -> Result<Map> {
    let file = File::open(&path)
        .with_context(|| format!("Failed to open given file({}).", path.as_ref().display()))?;
    let reader = BufReader::new(file);
    let mut builder = MapBuilder::new();
    for (ind, line) in reader.lines().enumerate() {
        let line = line.with_context(|| {
            format!(
                "Failed to read line {} in given file({}).",
                ind + 1,
                path.as_ref().display()
            )
        })?;
        builder.add_row(line.as_str())?
    }

    Ok(builder.build()?)
}
