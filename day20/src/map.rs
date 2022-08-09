use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet, HashMap, LinkedList},
    fmt::Debug,
    fs::File,
    io::{BufRead, BufReader},
    mem,
    path::Path,
};

use crate::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash)]
pub struct Position {
    r: isize,
    c: isize,
}

impl Ord for Position {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.r.cmp(&other.r) {
            Ordering::Equal => self.c.cmp(&other.c),
            other => other,
        }
    }
}

impl Position {
    pub fn new(r: isize, c: isize) -> Position {
        Position { r, c }
    }

    pub fn step_dir(&self, dir: Direction) -> Position {
        match dir {
            Direction::North => Position {
                r: self.r - 1,
                c: self.c,
            },
            Direction::West => Position {
                r: self.r,
                c: self.c - 1,
            },
            Direction::East => Position {
                r: self.r,
                c: self.c + 1,
            },
            Direction::South => Position {
                r: self.r + 1,
                c: self.c,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
struct Room {
    position: Position,
    connected: [bool; 4], // [north, west, east, south]
}

impl Ord for Room {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.position.cmp(&other.position)
    }
}

impl Room {
    pub fn with_pos(position: &Position) -> Room {
        Room {
            position: *position,
            connected: [false; 4],
        }
    }

    pub fn connect(&mut self, dir: Direction) {
        let dir_ind = Self::dir_to_ind(dir);
        self.connected[dir_ind] = true;
    }

    pub fn neighbors_pos(&self) -> impl Iterator<Item = Position> + '_ {
        self.connected.iter().enumerate().filter_map(|(ind, c)| {
            if *c {
                let dir = Self::ind_to_dir(ind).unwrap();
                Some(self.position.step_dir(dir))
            } else {
                None
            }
        })
    }

    fn dir_to_ind(dir: Direction) -> usize {
        match dir {
            Direction::North => 0,
            Direction::West => 1,
            Direction::East => 2,
            Direction::South => 3,
        }
    }

    fn ind_to_dir(ind: usize) -> Result<Direction, Error> {
        match ind {
            0 => Ok(Direction::North),
            1 => Ok(Direction::West),
            2 => Ok(Direction::East),
            3 => Ok(Direction::South),
            other => Err(Error::InvalidIndexToRoomDirection(other)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    North,
    West,
    East,
    South,
}

impl TryFrom<char> for Direction {
    type Error = Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'N' => Ok(Direction::North),
            'W' => Ok(Direction::West),
            'E' => Ok(Direction::East),
            'S' => Ok(Direction::South),
            other => Err(Error::InvalidDirectionChar(other)),
        }
    }
}

impl Direction {
    pub fn opposite(self) -> Self {
        match self {
            Self::North => Self::South,
            Self::West => Self::East,
            Self::East => Self::West,
            Self::South => Self::North,
        }
    }
}

type NeighborMat = HashMap<Position, LinkedList<Position>>;
#[derive(Debug, Clone)]
pub struct RoomMap {
    map: BTreeMap<Position, Room>,
}

impl RoomMap {
    pub fn new() -> RoomMap {
        RoomMap {
            map: BTreeMap::new(),
        }
    }

    pub fn add_pass(&mut self, pos: &Position, dir: Direction) -> Position {
        let new_pos = pos.step_dir(dir);
        let cur_room = self.map.entry(*pos).or_insert(Room::with_pos(pos));
        cur_room.connect(dir);
        let new_room = self.map.entry(new_pos).or_insert(Room::with_pos(&new_pos));
        new_room.connect(dir.opposite());

        new_pos
    }

    pub fn neighbor_mat(&self) -> NeighborMat {
        let mut mat = HashMap::new();
        for (_, room) in &self.map {
            mat.entry(room.position)
                .or_insert(LinkedList::from_iter(room.neighbors_pos()));
        }

        mat
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }
}

pub fn load_map<P>(input_path: P) -> Result<RoomMap, Error>
where
    P: AsRef<Path>,
{
    let input_file = File::open(&input_path).map_err(Error::IOError)?;
    let reader = BufReader::new(input_file);
    let path_str = reader
        .lines()
        .next()
        .map(|r| r.map_err(Error::IOError))
        .unwrap_or(Err(Error::EmptyInput(
            input_path.as_ref().to_str().unwrap_or("").to_string(),
        )))?;

    let tokens = lex_path_exp(path_str.as_str())?;
    let (scan_token_count, path_exp_node) = parse_path_exp_iter(&tokens)?;
    assert!(scan_token_count == tokens.len());

    let mut map = RoomMap::new();
    let origin = Position::new(0, 0);
    let mut pos_set = BTreeSet::from([origin]);
    path_exp_node.chart(&mut pos_set, &mut map)?;
    Ok(map)
}

pub fn bfs(beg_pos: &Position, map: &NeighborMat) -> HashMap<Position, Vec<Position>> {
    let mut pos_paths = HashMap::new();
    let mut pos_links = HashMap::from([(*beg_pos, *beg_pos)]);
    let mut pos_queue = LinkedList::from([*beg_pos]);
    while let Some(pos) = pos_queue.pop_front() {
        for neighbor in map[&pos].iter() {
            if !pos_links.contains_key(neighbor) {
                pos_links.insert(*neighbor, pos);
                pos_queue.push_back(*neighbor);
            }
        }
    }

    for end in pos_links.keys() {
        let mut cur_pos = *end;
        let mut path = Vec::new();
        while cur_pos != *beg_pos {
            path.push(cur_pos);
            cur_pos = pos_links[&cur_pos];
        }
        path.reverse();
        pos_paths.insert(*end, path);
    }

    pos_paths
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathExpToken {
    Path(String),
    OpLParen,
    OpRParen,
    OpOr,
}

impl TryFrom<char> for PathExpToken {
    type Error = Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        Ok(match value {
            '|' => PathExpToken::OpOr,
            '(' => PathExpToken::OpLParen,
            ')' => PathExpToken::OpRParen,
            other => return Err(Error::PathExpTokenParseError(other)),
        })
    }
}

#[derive(Clone, Copy)]
enum LexState {
    Begin,
    Lexing,
    End,
}

fn lex_path_exp(text: &str) -> Result<Vec<PathExpToken>, Error> {
    let mut temp_path: Option<String> = None;
    let mut tokens = Vec::new();
    let mut state = LexState::Begin;
    for (c_ind, c) in text.chars().enumerate() {
        match state {
            LexState::Begin => {
                if c == '^' {
                    state = LexState::Lexing;
                }
            }
            LexState::Lexing => match c {
                'N' | 'W' | 'E' | 'S' => {
                    temp_path = match temp_path.take() {
                        Some(mut p) => {
                            p.push(c);
                            Some(p)
                        }
                        None => Some(String::from(c)),
                    };
                }
                '|' | '(' | ')' | '$' => {
                    if let Some(s) = temp_path.take() {
                        tokens.push(PathExpToken::Path(s));
                    }

                    if c == '$' {
                        state = LexState::End;
                    } else {
                        tokens.push(PathExpToken::try_from(c)?);
                    }
                }
                other => {
                    return Err(Error::InvalidCharInInput {
                        ind: c_ind,
                        c: other,
                    })
                }
            },
            LexState::End => {
                break;
            }
        }
    }

    match state {
        LexState::Begin => Err(Error::NoBeginInPathExp),
        LexState::Lexing => Err(Error::NoEndInPathExp),
        LexState::End => Ok(tokens),
    }
}

// Exp -> GroupExp | PathExp
// GroupExp -> [OpLParen] Branch [OpRParen] Exp |
//             [OpLParen] Branch [OpRParen]
// Pathexp -> [Path] GroupExp |
//            [Path]
// Branch -> Alter [OpOr] Branch |
//          Alter
// Alter -> Exp |
//          \epsilon
trait PathProduct: Debug {
    fn chart(&self, pos_set: &mut BTreeSet<Position>, map: &mut RoomMap) -> Result<(), Error>;
}

// PathExp -> [Path] GroupExp |
//            [Path]
#[derive(Debug)]
struct PathLeadingExp {
    path: String,
    tail_exp: Option<Box<dyn PathProduct>>,
}

impl PathProduct for PathLeadingExp {
    fn chart(&self, pos_set: &mut BTreeSet<Position>, map: &mut RoomMap) -> Result<(), Error> {
        let dirs = self
            .path
            .chars()
            .map(|c| Direction::try_from(c))
            .collect::<Result<Vec<_>, Error>>()?;
        let mut new_pos_set = BTreeSet::new();
        for pos in pos_set.iter() {
            let mut new_pos = *pos;
            for dir in &dirs {
                new_pos = map.add_pass(&new_pos, *dir);
            }
            new_pos_set.insert(new_pos);
        }
        mem::swap(pos_set, &mut new_pos_set);

        if let Some(exp) = self.tail_exp.as_ref() {
            return exp.chart(pos_set, map);
        }

        Ok(())
    }
}

// GroupExp -> [OpLParen] Branch [OpRParen] Exp |
//             [OpLParen] Branch [OpRParen]
#[derive(Debug)]
struct BranchLeadingExp {
    branch: Box<dyn PathProduct>,
    tail_exp: Option<Box<dyn PathProduct>>,
}

impl PathProduct for BranchLeadingExp {
    fn chart(&self, pos_set: &mut BTreeSet<Position>, map: &mut RoomMap) -> Result<(), Error> {
        self.branch.chart(pos_set, map)?;
        if let Some(exp) = self.tail_exp.as_ref() {
            return exp.chart(pos_set, map);
        }

        Ok(())
    }
}

// Alter -> \epsilon
#[derive(Debug)]
struct EmptyAlter;
impl PathProduct for EmptyAlter {
    fn chart(&self, _: &mut BTreeSet<Position>, _: &mut RoomMap) -> Result<(), Error> {
        Ok(())
    }
}

// Alter -> Exp
#[derive(Debug)]
struct ExpAlter {
    exp: Box<dyn PathProduct>,
}

impl PathProduct for ExpAlter {
    fn chart(&self, pos_set: &mut BTreeSet<Position>, map: &mut RoomMap) -> Result<(), Error> {
        self.exp.chart(pos_set, map)
    }
}

// Branch -> Alter [OpOr] Branch |
//           Alter
#[derive(Debug)]
struct ExpLeadingBranch {
    exp: Box<dyn PathProduct>,
    tail_branch: Option<Box<dyn PathProduct>>,
}

impl PathProduct for ExpLeadingBranch {
    fn chart(&self, pos_set: &mut BTreeSet<Position>, map: &mut RoomMap) -> Result<(), Error> {
        let mut branch_pos_set = pos_set.clone();
        self.exp.chart(pos_set, map)?;
        if let Some(branch) = self.tail_branch.as_ref() {
            branch.chart(&mut branch_pos_set, map)?;
        }

        pos_set.extend(branch_pos_set.iter());
        Ok(())
    }
}

#[derive(Debug)]
enum ParseState {
    ParsingExpEnter(usize),
    ParsingExpInPathExpLeave(String),
    ParsingExpInBranchExpLeave(Box<dyn PathProduct>),
    ParsingExpInBranchLeave(usize),
    ParsingBranchEnter(usize),
    ParsingBranchInGroupExpLeave,
    ParsingBranchInBranchLeave(usize, Box<dyn PathProduct>),
}

fn parse_path_exp_iter(tokens: &[PathExpToken]) -> Result<(usize, Box<dyn PathProduct>), Error> {
    let mut state_stack = LinkedList::from([ParseState::ParsingExpEnter(0)]);
    let mut res_stack: LinkedList<Result<(usize, Box<dyn PathProduct>), Error>> = LinkedList::new();
    while let Some(state) = state_stack.pop_back() {
        match state {
            ParseState::ParsingExpEnter(mut t_ind) => {
                let cur_token = &tokens[t_ind];
                match cur_token {
                    PathExpToken::Path(s) => {
                        // PathExp -> [Path] GroupExp |
                        //            [Path]
                        let next_token = tokens.get(t_ind + 1);
                        if let Some(next_token) = next_token {
                            match next_token {
                                PathExpToken::OpLParen => {
                                    // PathExp -> [Path] GroupExp
                                    t_ind += 1;
                                    state_stack
                                        .push_back(ParseState::ParsingExpInPathExpLeave(s.clone()));
                                    state_stack.push_back(ParseState::ParsingExpEnter(t_ind));
                                }
                                PathExpToken::OpOr | PathExpToken::OpRParen => {
                                    // PathExp -> [Path]
                                    t_ind += 1;
                                    res_stack.push_back(Ok((
                                        t_ind,
                                        Box::new(PathLeadingExp {
                                            path: s.clone(),
                                            tail_exp: None,
                                        }),
                                    )))
                                }
                                other => {
                                    res_stack.push_back(Err(Error::InvalidTokenInExpParsing(
                                        other.clone(),
                                    )));
                                }
                            }
                        } else {
                            t_ind += 1;
                            res_stack.push_back(Ok((
                                t_ind,
                                Box::new(PathLeadingExp {
                                    path: s.clone(),
                                    tail_exp: None,
                                }),
                            )));
                        }
                    }
                    PathExpToken::OpLParen => {
                        t_ind += 1;
                        state_stack.push_back(ParseState::ParsingBranchInGroupExpLeave);
                        state_stack.push_back(ParseState::ParsingBranchEnter(t_ind));
                    }
                    other => {
                        res_stack.push_back(Err(Error::InvalidTokenInExpParsing(other.clone())));
                    }
                }
            }
            ParseState::ParsingExpInPathExpLeave(s) => {
                // PathExp -> [Path] GroupExp
                let (t_ind, tail_exp) = res_stack
                    .pop_back()
                    .unwrap_or(Err(Error::EmptyResultStackInExpParsing))?;
                res_stack.push_back(Ok((
                    t_ind,
                    Box::new(PathLeadingExp {
                        path: s.clone(),
                        tail_exp: Some(tail_exp),
                    }),
                )))
            }
            ParseState::ParsingExpInBranchExpLeave(branch) => {
                // GroupExp -> [OpLParen] Branch [OpRParen] Exp
                let (t_ind, tail_exp) = res_stack
                    .pop_back()
                    .unwrap_or(Err(Error::EmptyResultStackInExpParsing))?;
                res_stack.push_back(Ok((
                    t_ind,
                    Box::new(BranchLeadingExp {
                        branch,
                        tail_exp: Some(tail_exp),
                    }),
                )));
            }
            ParseState::ParsingBranchInGroupExpLeave => {
                let (mut t_ind, branch) = res_stack
                    .pop_back()
                    .unwrap_or(Err(Error::EmptyResultStackInExpParsing))?;
                if tokens[t_ind] != PathExpToken::OpRParen {
                    return Err(Error::RParenNotFoundInBranchEnd(t_ind));
                }
                t_ind += 1;
                let next_token = tokens.get(t_ind);
                if let Some(next_token) = next_token {
                    match next_token {
                        PathExpToken::Path(_) | PathExpToken::OpLParen => {
                            // GroupExp -> [OpLParen] Branch [OpRParen] Exp
                            state_stack.push_back(ParseState::ParsingExpInBranchExpLeave(branch));
                            state_stack.push_back(ParseState::ParsingExpEnter(t_ind));
                        }
                        // GroupExp -> [OpLParen] Branch [OpRParen]
                        _ => {
                            res_stack.push_back(Ok((
                                t_ind,
                                Box::new(BranchLeadingExp {
                                    branch,
                                    tail_exp: None,
                                }),
                            )));
                        }
                    }
                } else {
                    // GroupExp -> [OpLParen] Branch [OpRParen]
                    res_stack.push_back(Ok((
                        t_ind,
                        Box::new(BranchLeadingExp {
                            branch,
                            tail_exp: None,
                        }),
                    )));
                }
            }
            ParseState::ParsingBranchEnter(t_ind) => {
                if t_ind >= tokens.len() {
                    res_stack.push_back(Err(Error::EmptyTokensLeftInBranchParsing));
                } else {
                    state_stack.push_back(ParseState::ParsingExpInBranchLeave(t_ind));
                    state_stack.push_back(ParseState::ParsingExpEnter(t_ind));
                }
            }
            ParseState::ParsingExpInBranchLeave(t_ind) => {
                let (mut t_ind, exp) = match res_stack
                    .pop_back()
                    .unwrap_or(Err(Error::EmptyResultStackInExpParsing))
                {
                    Ok((t_ind, exp)) => (t_ind, exp), // Alter -> Exp
                    Err(Error::InvalidTokenInExpParsing(_)) => {
                        // Alter -> \epsilon
                        res_stack.push_back(Ok((
                            t_ind,
                            Box::new(ExpAlter {
                                exp: Box::new(EmptyAlter),
                            }),
                        )));
                        continue;
                    }
                    Err(other) => {
                        res_stack.push_back(Err(other));
                        continue;
                    }
                };

                if let Some(cur_token) = tokens.get(t_ind) {
                    match cur_token {
                        PathExpToken::OpOr => {
                            // Branch -> Alter [OpOr] Branch
                            t_ind += 1; // Skip OpOr
                            state_stack
                                .push_back(ParseState::ParsingBranchInBranchLeave(t_ind, exp));
                            state_stack.push_back(ParseState::ParsingBranchEnter(t_ind));
                        }
                        // Branch -> Alter
                        PathExpToken::OpRParen => res_stack.push_back(Ok((
                            t_ind,
                            Box::new(ExpLeadingBranch {
                                exp,
                                tail_branch: None,
                            }),
                        ))),
                        other => res_stack
                            .push_back(Err(Error::InvalidTokenInBranchParsing(other.clone()))),
                    }
                } else {
                    res_stack.push_back(Err(Error::InvalidEndInBranchParsing));
                }
            }
            ParseState::ParsingBranchInBranchLeave(_, exp) => {
                // Branch -> Alter [OpOr] Branch
                let (t_ind, tail_branch) = res_stack
                    .pop_back()
                    .unwrap_or(Err(Error::EmptyResultStackInExpParsing))?;
                res_stack.push_back(Ok((
                    t_ind,
                    Box::new(ExpLeadingBranch {
                        exp,
                        tail_branch: Some(tail_branch),
                    }),
                )));
            }
        }
    }

    res_stack
        .pop_back()
        .unwrap_or(Err(Error::EmptyResultStackInExpParsing))
}

fn parse_path_exp_recur(tokens: &[PathExpToken]) -> Result<(usize, Box<dyn PathProduct>), Error> {
    let mut t_ind = 0;
    let cur_token = &tokens[t_ind];
    match cur_token {
        PathExpToken::Path(s) => {
            // PathExp -> [Path] GroupExp |
            //            [Path]
            let next_token = tokens.get(t_ind + 1);
            if let Some(next_token) = next_token {
                match next_token {
                    PathExpToken::OpLParen => {
                        // PathExp -> [Path] GroupExp
                        t_ind += 1;
                        let (token_count, tail_exp) = parse_path_exp_recur(&tokens[t_ind..])?;
                        t_ind += token_count;
                        Ok((
                            t_ind,
                            Box::new(PathLeadingExp {
                                path: s.clone(),
                                tail_exp: Some(tail_exp),
                            }),
                        ))
                    }
                    PathExpToken::OpOr | PathExpToken::OpRParen => {
                        // PathExp -> [Path]
                        t_ind += 1;
                        Ok((
                            1,
                            Box::new(PathLeadingExp {
                                path: s.clone(),
                                tail_exp: None,
                            }),
                        ))
                    }
                    other => Err(Error::InvalidTokenInExpParsing(other.clone())),
                }
            } else {
                Ok((
                    1,
                    Box::new(PathLeadingExp {
                        path: s.clone(),
                        tail_exp: None,
                    }),
                ))
            }
        }
        PathExpToken::OpLParen => {
            t_ind += 1;
            let (token_count, branch) = parse_path_branch_recur(&tokens[t_ind..])?;
            t_ind += token_count;
            if tokens[t_ind] != PathExpToken::OpRParen {
                return Err(Error::RParenNotFoundInBranchEnd(t_ind));
            }
            t_ind += 1;
            let next_token = tokens.get(t_ind);
            if let Some(next_token) = next_token {
                match next_token {
                    PathExpToken::Path(_) | PathExpToken::OpLParen => {
                        // GroupExp -> [OpLParen] Branch [OpRParen] Exp
                        let (token_count, tail_exp) = parse_path_exp_recur(&tokens[t_ind..])?;
                        t_ind += token_count;
                        Ok((
                            t_ind,
                            Box::new(BranchLeadingExp {
                                branch,
                                tail_exp: Some(tail_exp),
                            }),
                        ))
                    }
                    // GroupExp -> [OpLParen] Branch [OpRParen]
                    _ => Ok((
                        t_ind,
                        Box::new(BranchLeadingExp {
                            branch,
                            tail_exp: None,
                        }),
                    )),
                }
            } else {
                // GroupExp -> [OpLParen] Branch [OpRParen]
                Ok((
                    t_ind,
                    Box::new(BranchLeadingExp {
                        branch,
                        tail_exp: None,
                    }),
                ))
            }
        }
        other => Err(Error::InvalidTokenInExpParsing(other.clone())),
    }
}

fn parse_path_branch_recur(
    tokens: &[PathExpToken],
) -> Result<(usize, Box<dyn PathProduct>), Error> {
    if tokens.is_empty() {
        return Err(Error::EmptyTokensLeftInBranchParsing);
    }

    let (token_count, exp) = match parse_path_exp_recur(tokens) {
        Ok((token_count, exp)) => (token_count, exp), // Alter -> Exp
        Err(Error::InvalidTokenInExpParsing(_)) => {
            return Ok((
                0,
                Box::new(ExpAlter {
                    exp: Box::new(EmptyAlter),
                }),
            ))
        } // Alter -> \epsilon
        Err(other) => return Err(other),
    };

    let mut t_ind = token_count;
    if let Some(cur_token) = tokens.get(t_ind) {
        match cur_token {
            PathExpToken::OpOr => {
                // Branch -> Alter [OpOr] Branch
                t_ind += 1; // Skip OpOr
                let (token_count, tail_branch) = parse_path_branch_recur(&tokens[t_ind..])?;
                t_ind += token_count;
                Ok((
                    t_ind,
                    Box::new(ExpLeadingBranch {
                        exp,
                        tail_branch: Some(tail_branch),
                    }),
                ))
            }
            // Branch -> Alter
            PathExpToken::OpRParen => Ok((
                t_ind,
                Box::new(ExpLeadingBranch {
                    exp,
                    tail_branch: None,
                }),
            )),
            other => Err(Error::InvalidTokenInBranchParsing(other.clone())),
        }
    } else {
        Err(Error::InvalidEndInBranchParsing)
    }
}
