use std::{
    collections::{HashMap, HashSet, LinkedList},
    error,
    fmt::Display,
    ops::{Index, Range},
};

use crate::int_code::io::{InputPort, OutputPort};

#[derive(Debug)]
pub enum Error {
    IntCodeError(crate::Error),
    InconsistentMapRow(usize, usize),
    InvalidScaffoldMapValue(i64),
    MultipleRobotInMap(usize, usize),
    RobotNotFoundInMap,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IntCodeError(e) => write!(f, "{}", e.to_string()),
            Error::InconsistentMapRow(this_col_n, last_col_n) => write!(f, "Found inconsistent row in scaffold map, this row has {} columns, the last one has {} columns", this_col_n, last_col_n),
            Error::InvalidScaffoldMapValue(v) => write!(f, "Invalid value({}) in scaffold map", v),
            Error::MultipleRobotInMap(last_ind, ind) => write!(f, "Found multiple robots(last one at {}, this one at {}) in given scaffold map", last_ind, ind),
            Error::RobotNotFoundInMap => write!(f, "Can't find robot in scaffold map, expect one"),
        }
    }
}

impl error::Error for Error {}

#[derive(Debug, PartialEq, Eq)]
enum TileType {
    Scaffold,
    Empty,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RobotCommand {
    TurnLeft,
    TurnRight,
    Forward(usize),
}

impl Display for RobotCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RobotCommand::TurnLeft => write!(f, "L"),
            RobotCommand::TurnRight => write!(f, "R"),
            RobotCommand::Forward(n) => write!(f, "F({})", n),
        }
    }
}

impl RobotCommand {
    pub fn text(&self) -> String {
        match self {
            RobotCommand::TurnLeft => "L".to_string(),
            RobotCommand::TurnRight => "R".to_string(),
            RobotCommand::Forward(n) => format!("{}", n),
        }
    }

    pub fn text_len(&self) -> usize {
        match self {
            RobotCommand::TurnLeft => 1,
            RobotCommand::TurnRight => 1,
            RobotCommand::Forward(n) => digits_n(*n),
        }
    }
}

fn digits_n(n: usize) -> usize {
    if n == 0 {
        return 1;
    }

    let mut digits_n = 0;
    let mut product = 1usize;
    while n >= product {
        product *= 10;
        digits_n += 1;
    }

    digits_n
}

pub fn text_len(path: &[RobotCommand]) -> usize {
    if path.is_empty() {
        return 0;
    }

    let com_texts_len: usize = path.iter().map(|c| c.text_len()).sum();
    com_texts_len + path.len() - 1
}

#[derive(Debug, Clone)]
pub struct RobotPath(Vec<RobotCommand>);

impl Display for RobotPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        if !self.0.is_empty() {
            for i in 0..(self.0.len() - 1) {
                write!(f, "{}, ", self.0[i])?;
            }
            write!(f, "{}", self.0[self.0.len() - 1])?;
        }
        write!(f, "]")
    }
}

impl Index<Range<usize>> for RobotPath {
    type Output = [RobotCommand];

    fn index(&self, index: Range<usize>) -> &Self::Output {
        &self.path()[index]
    }
}

impl RobotPath {
    pub fn new(path: Vec<RobotCommand>) -> Self {
        Self(path)
    }

    pub fn path(&self) -> &[RobotCommand] {
        &self.0
    }

    pub fn text(&self) -> String {
        if self.0.is_empty() {
            return String::new();
        }

        let com_texts = self.0.iter().map(|c| c.text()).collect::<Vec<_>>();
        com_texts.join(",")
    }

    pub fn text_len(&self) -> usize {
        text_len(&self.path())
    }
}

#[derive(Debug)]
pub struct ZipRobotPath {
    path: Vec<usize>,
    subpaths: Vec<RobotPath>,
}

impl ZipRobotPath {
    pub fn new<F: Fn(&[RobotCommand]) -> bool>(src_path: &RobotPath, is_valid: F) -> Self {
        let mut path_ranges = LinkedList::new();
        path_ranges.push_back(0..src_path.path().len());
        let subpath_ranges = Self::min_subpaths(src_path, path_ranges, &is_valid, 0, None);
        let path = Self::path_from_sub_paths(&src_path, &subpath_ranges);
        let subpaths = subpath_ranges
            .iter()
            .map(|sp_range| {
                RobotPath(
                    src_path.path()[sp_range.clone()]
                        .iter()
                        .cloned()
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>();

        Self { path, subpaths }
    }

    pub fn path(&self) -> &[usize] {
        &self.path
    }

    pub fn sub_paths(&self) -> &[RobotPath] {
        &self.subpaths
    }

    fn min_subpaths<F: Fn(&[RobotCommand]) -> bool>(
        path: &RobotPath,
        mut path_ranges: LinkedList<Range<usize>>,
        is_valid: &F,
        level: usize,
        min_subpath_n: Option<usize>,
    ) -> LinkedList<Range<usize>> {
        let mut subpath_ranges = LinkedList::new();
        // Clone first nonempty range
        while let Some(first_path_range) = path_ranges.front().cloned() {
            if first_path_range.is_empty() {
                path_ranges.pop_front();
                continue;
            }

            if path_ranges.len() == 1 && is_valid(&path[first_path_range.clone()]) {
                // If only one valid subpath is left, return that one.
                subpath_ranges.push_back(first_path_range);
            } else {
                // Cut off remained recursion branches, if number of subpaths has exceeds known minimium number of subpaths.
                if min_subpath_n.is_some_and(|n| level + 1 >= n) {
                    let mut command_map = HashMap::new();
                    path_ranges.iter().flat_map(|r| r.clone()).for_each(|ind| {
                        command_map.entry(path.path()[ind].clone()).or_insert(ind);
                    });
                    subpath_ranges.extend(command_map.iter().map(|(_, v)| *v..(*v + 1)));
                    break;
                }

                let mut min_subpath_ranges = LinkedList::new();
                let mut min_subpath_n = usize::MAX;
                for i in 1..=first_path_range.len() {
                    let first_subpath_range = first_path_range.start..(first_path_range.start + i);
                    if !is_valid(&path[first_subpath_range.clone()]) {
                        break;
                    }

                    let mut this_subpath_ranges = LinkedList::new();
                    this_subpath_ranges.push_back(first_subpath_range.clone());

                    // Recursion, find minimium subpaths in path ranges which have this_subpath removed.
                    let left_path_ranges =
                        Self::remove_subpath_range(path, &first_subpath_range, &path_ranges);
                    let mut left_min_subpath_ranges = Self::min_subpaths(
                        path,
                        left_path_ranges,
                        is_valid,
                        level + 1,
                        Some(min_subpath_n),
                    );

                    let this_subpath_n = left_min_subpath_ranges.len() + 1;
                    if this_subpath_n < min_subpath_n {
                        // Found less subpaths to construct original path.
                        this_subpath_ranges.append(&mut left_min_subpath_ranges);
                        min_subpath_ranges = this_subpath_ranges;
                        min_subpath_n = this_subpath_n;
                    }
                }

                subpath_ranges = min_subpath_ranges;
            }
            break;
        }

        subpath_ranges
    }

    fn remove_subpath_range(
        path: &RobotPath,
        remove_range: &Range<usize>,
        path_ranges: &LinkedList<Range<usize>>,
    ) -> LinkedList<Range<usize>> {
        let mut left_path_ranges = LinkedList::new();
        for range in path_ranges.iter() {
            let mut last_unequal_ind = range.start;
            let mut remove_range_ind = remove_range.start;
            for i in range.clone() {
                if path.path()[i] == path.path()[remove_range_ind] {
                    remove_range_ind += 1;
                    if !remove_range.contains(&remove_range_ind) {
                        // Found a repeat subrange, reset index in remove_range, and insert left range.
                        remove_range_ind = remove_range.start;

                        let last_left_range = last_unequal_ind..(i - (remove_range.len() - 1));
                        if !last_left_range.is_empty() {
                            left_path_ranges.push_back(last_left_range);
                        }
                        last_unequal_ind = i + 1;
                    }
                } else {
                    remove_range_ind = remove_range.start;
                }
            }

            if last_unequal_ind < range.end {
                // Insert final left range.
                left_path_ranges.push_back(last_unequal_ind..range.end);
            }
        }

        left_path_ranges
    }

    fn path_from_sub_paths(path: &RobotPath, path_ranges: &LinkedList<Range<usize>>) -> Vec<usize> {
        let mut inds = Vec::new();
        let mut path_ind = 0;
        let path = path.path();
        while path_ind < path.len() {
            let mut sub_path_found = None;
            // Check each subpath in order.
            for (subpath_ind, range) in path_ranges.iter().enumerate() {
                if path[range.clone()]
                    .iter()
                    .enumerate()
                    .all(|(ind, command)| *command == path[path_ind + ind])
                {
                    // Found one matched subpath, record its index.
                    sub_path_found = Some(subpath_ind);
                    path_ind += range.len();
                    break;
                }
            }

            inds.push(sub_path_found.expect(&format!(
                "Failed to find a subpath match elements in path at {}.",
                path_ind
            )));
        }

        inds
    }
}

pub struct ZipPathPilot {
    input_buf: Vec<char>,
    input_ind: usize,
    dust_n: usize,
    need_video: bool,
}

impl ZipPathPilot {
    pub fn new(path: &ZipRobotPath, need_video: bool) -> Self {
        let mut input_buf = String::new();
        // Push main move function.
        let move_fn_codes = ["A", "B", "C"];
        let main_move_fn_str = path
            .path()
            .iter()
            .map(|ind| {
                assert!(*ind < 3, "Only support at most 3 move functions.");
                move_fn_codes[*ind]
            })
            .collect::<Vec<_>>()
            .join(",");
        input_buf.push_str(&main_move_fn_str);
        input_buf.push('\n');

        // Push sub move functions.
        for move_fn in path.sub_paths() {
            input_buf.push_str(&move_fn.text());
            input_buf.push('\n');
        }

        // Push answer for video feed
        input_buf.push(if need_video { 'y' } else { 'n' });
        input_buf.push('\n');
        let input_buf = input_buf.chars().collect::<Vec<_>>();

        Self {
            input_buf,
            input_ind: 0,
            dust_n: 0,
            need_video,
        }
    }

    pub fn dust_n(&self) -> usize {
        self.dust_n
    }
}

impl InputPort for ZipPathPilot {
    fn get(&mut self) -> Option<i64> {
        let ind = self.input_ind;
        self.input_ind += 1;
        self.input_buf.get(ind).map(|c| u32::from(*c).into())
    }

    fn reg_proc(&mut self, _proc_id: usize) {}
}

impl OutputPort for ZipPathPilot {
    fn put(&mut self, value: i64) -> Result<(), crate::Error> {
        if !self.need_video || (self.need_video && value >= 0) {
            // Playing video isn't supported yet.
            self.dust_n = usize::try_from(value).expect(&format!("Invalid dust amount({})", value));
        }

        Ok(())
    }

    fn wait_proc_id(&self) -> Option<usize> {
        None
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash)]
pub struct Position(usize, usize); // (x, y)

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}

impl Position {
    pub fn x(&self) -> usize {
        self.0
    }

    pub fn y(&self) -> usize {
        self.1
    }

    pub fn next_along(&self, dir: Direction) -> Option<Self> {
        match dir {
            Direction::North if self.1 > 0 => Some(Position(self.0, self.1 - 1)),
            Direction::South => Some(Position(self.0, self.1 + 1)),
            Direction::West if self.0 > 0 => Some(Position(self.0 - 1, self.1)),
            Direction::East => Some(Position(self.0 + 1, self.1)),
            _ => None,
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

impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::North => write!(f, "North"),
            Direction::South => write!(f, "South"),
            Direction::West => write!(f, "West"),
            Direction::East => write!(f, "East"),
        }
    }
}

impl Direction {
    pub fn left(&self) -> Self {
        match self {
            Direction::North => Direction::West,
            Direction::South => Direction::East,
            Direction::West => Direction::South,
            Direction::East => Direction::North,
        }
    }

    pub fn right(&self) -> Self {
        self.left().reverse()
    }

    pub fn reverse(&self) -> Self {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
            Direction::East => Direction::West,
        }
    }
}

#[derive(Debug, Clone)]
struct VaccumRobot {
    pos: Position,
    dir: Direction,
    passed_scaffolds: HashSet<Position>,
    path: RobotPath,
}

impl Display for VaccumRobot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "VaccumRobot({}, {})", self.pos, self.dir)
    }
}

impl VaccumRobot {
    pub fn new(pos: &Position, dir: Direction) -> Self {
        let mut passed_scaffolds = HashSet::new();
        passed_scaffolds.insert(*pos);
        Self {
            pos: *pos,
            dir,
            passed_scaffolds,
            path: RobotPath(Vec::new()),
        }
    }

    pub fn move_neighbors<F: Fn(&Position, bool) -> bool>(
        &self,
        neighbors: &mut LinkedList<Position>,
        filter: F,
    ) {
        neighbors.extend(
            [self.dir, self.dir.left(), self.dir.right()]
                .iter()
                .flat_map(|dir| self.pos.next_along(*dir).into_iter())
                .filter(|pos| filter(pos, self.passed_scaffolds.contains(pos))),
        );
    }

    pub fn move_to_neighbor(&mut self, pos: &Position) {
        let forward_pos = self.pos.next_along(self.dir);
        let left_pos = self.pos.next_along(self.dir.left());
        let right_pos = self.pos.next_along(self.dir.right());
        if forward_pos.is_some_and(|p| p == *pos) {
            // Move forward
            self.forward();
        } else if left_pos.is_some_and(|p| p == *pos) {
            // Turn left
            self.turn_left();
            self.forward();
        } else if right_pos.is_some_and(|p| p == *pos) {
            // Turn right
            self.turn_right();
            self.forward();
        } else {
            panic!(
                "Given neighbor position({}) isn't this robot({})'s neighbor.",
                pos, self.pos
            );
        }
    }

    pub fn passed_count(&self) -> usize {
        self.passed_scaffolds.len()
    }

    pub fn path(&self) -> RobotPath {
        self.path.clone()
    }

    fn forward(&mut self) {
        self.pos = self
            .pos
            .next_along(self.dir)
            .expect(&format!("{} can't forward any more.", self));
        self.passed_scaffolds.insert(self.pos);
        if let Some(command) = self.path.0.pop() {
            match command {
                // Combine two forward command.
                RobotCommand::Forward(n) => self.path.0.push(RobotCommand::Forward(n + 1)),
                c => {
                    self.path.0.push(c);
                    self.path.0.push(RobotCommand::Forward(1));
                }
            }
        } else {
            self.path.0.push(RobotCommand::Forward(1));
        }
    }

    fn turn_left(&mut self) {
        self.dir = self.dir.left();
        self.path.0.push(RobotCommand::TurnLeft);
    }

    fn turn_right(&mut self) {
        self.dir = self.dir.right();
        self.path.0.push(RobotCommand::TurnRight);
    }
}

pub struct ScaffoldMap {
    map: Vec<TileType>,
    row_n: usize,
    col_n: usize,
    robot_ind: usize,
    robot_dir: Direction,
    scaffold_n: usize,
}

impl Display for ScaffoldMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.row_n {
            for x in 0..self.col_n {
                if self.pos_to_ind(&Position(x, y)) == self.robot_ind {
                    write!(
                        f,
                        "{}",
                        match self.robot_dir {
                            Direction::North => "^",
                            Direction::South => "v",
                            Direction::West => "<",
                            Direction::East => ">",
                        }
                    )?;
                    continue;
                }

                write!(
                    f,
                    "{}",
                    match self.tile(&Position(x, y)).unwrap() {
                        TileType::Scaffold => "#",
                        TileType::Empty => ".",
                    }
                )?;
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}

impl ScaffoldMap {
    pub fn try_from_ints<I: Iterator<Item = i64>>(iter: I) -> Result<Self, Error> {
        let mut map = Vec::new();
        let mut row_n = 0;
        let mut col_n = 0;
        let mut robot_ind = None;
        let mut robot_dir = None;
        let mut scaffold_n = 0;
        for (ind, v) in iter.enumerate() {
            match u32::try_from(v)
                .map_err(|_| Error::InvalidScaffoldMapValue(v))
                .and_then(|u| char::from_u32(u).ok_or(Error::InvalidScaffoldMapValue(v)))?
            {
                '#' => {
                    map.push(TileType::Scaffold);
                    scaffold_n += 1;
                }
                dir @ ('^' | 'v' | '<' | '>') => {
                    let dir = match dir {
                        '^' => Direction::North,
                        'v' => Direction::South,
                        '<' => Direction::West,
                        '>' => Direction::East,
                        _ => unreachable!(),
                    };
                    // Subtract the number of new line.
                    let ind_in_map = ind - row_n;
                    if *robot_ind.get_or_insert(ind_in_map) != ind_in_map
                        || *robot_dir.get_or_insert(dir) != dir
                    {
                        return Err(Error::MultipleRobotInMap(robot_ind.unwrap(), ind_in_map));
                    }
                    map.push(TileType::Scaffold);
                    scaffold_n += 1;
                }
                '.' => map.push(TileType::Empty),
                'X' => {
                    // Subtract the number of new line.
                    let ind_in_map = ind - row_n;
                    if *robot_ind.get_or_insert(ind_in_map) != ind_in_map {
                        return Err(Error::MultipleRobotInMap(robot_ind.unwrap(), ind_in_map));
                    }
                    map.push(TileType::Empty)
                }
                '\n' => {
                    if col_n == 0 {
                        col_n = ind;
                    }

                    let this_col_n = ind - (col_n + 1) * row_n;
                    if this_col_n != 0 {
                        // Ignore empty line.
                        row_n += 1;
                        if this_col_n != col_n {
                            return Err(Error::InconsistentMapRow(this_col_n, col_n));
                        }
                    }
                }
                _ => return Err(Error::InvalidScaffoldMapValue(v)),
            };
        }

        if robot_dir.is_none() || robot_ind.is_none() {
            return Err(Error::RobotNotFoundInMap);
        }

        Ok(ScaffoldMap {
            map,
            row_n,
            col_n,
            robot_ind: robot_ind.unwrap(),
            robot_dir: robot_dir.unwrap(),
            scaffold_n,
        })
    }

    pub fn intersections(&self) -> Vec<Position> {
        let mut res = Vec::new();
        for y in 0..self.row_n {
            for x in 0..self.col_n {
                let pos = Position(x, y);
                if self.is_intersection(&pos) {
                    res.push(pos);
                }
            }
        }

        res
    }

    pub fn one_touch_paths(&self) -> Vec<RobotPath> {
        let mut paths = Vec::new();
        let init_robot = VaccumRobot::new(&self.ind_to_pos(self.robot_ind), self.robot_dir);
        let mut robots = LinkedList::new();
        robots.push_back(init_robot);
        let mut neighbors = LinkedList::new();

        while let Some(mut robot) = robots.pop_front() {
            debug_assert!(neighbors.is_empty());
            loop {
                // Get tiles this robot can move to, which have not been passed earlier and are scaffold, or it's an intersection.
                robot.move_neighbors(&mut neighbors, |pos, is_passed| {
                    self.is_intersection(pos) || (self.is_scaffold(pos) && !is_passed)
                });
                if let Some(pos) = neighbors.pop_front() {
                    // Neighbors after the first are left to cloned robot exploring in later loop.
                    while let Some(pos) = neighbors.pop_front() {
                        // At intersection, split and explore each path.
                        let mut new_robot = robot.clone();
                        new_robot.move_to_neighbor(&pos);
                        robots.push_back(new_robot);
                    }

                    // This robot move to the first movable tile, other tiles are left for robots cloned above.
                    robot.move_to_neighbor(&pos);
                } else {
                    // This robot can't move any more, start next one.
                    if robot.passed_count() >= self.scaffold_n {
                        // This robot completes the whole map, record its path.
                        paths.push(robot.path());
                    }
                    break;
                }
            }
        }

        paths
    }

    fn is_valid_pos(&self, pos: &Position) -> bool {
        pos.x() < self.col_n && pos.y() < self.row_n
    }

    fn tile(&self, pos: &Position) -> Option<&TileType> {
        if !self.is_valid_pos(pos) {
            return None;
        }

        self.map.get(self.pos_to_ind(pos))
    }

    fn is_scaffold(&self, pos: &Position) -> bool {
        self.tile(pos).is_some_and(|tt| *tt == TileType::Scaffold)
    }

    fn is_intersection(&self, pos: &Position) -> bool {
        self.is_scaffold(pos)
            && ([
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East,
            ]
            .iter()
            .flat_map(|dir| pos.next_along(*dir))
            .filter(|pos| self.is_scaffold(pos))
            .count()
                == 4)
    }

    fn pos_to_ind(&self, pos: &Position) -> usize {
        assert!(pos.x() < self.col_n && pos.y() < self.row_n);
        pos.y() * self.col_n + pos.x()
    }

    fn ind_to_pos(&self, ind: usize) -> Position {
        assert!(ind < self.map.len());
        Position(ind % self.col_n, ind / self.col_n)
    }
}
