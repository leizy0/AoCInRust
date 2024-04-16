use std::{fmt::Display, ops::Range};

use crate::{Direction, Error, Position};

pub mod walker;

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
pub(super) struct MazeRow {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MazePosition {
    pos_in_level: Position,
    level: usize,
}

impl Display for MazePosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.pos_in_level(), self.level())
    }
}

impl MazePosition {
    pub fn new(pos: &Position, level: usize) -> Self {
        Self {
            pos_in_level: pos.clone(),
            level,
        }
    }

    pub fn pos_in_level(&self) -> &Position {
        &self.pos_in_level
    }

    pub fn level(&self) -> usize {
        self.level
    }

    pub fn move_along(&self, dir: Direction) -> Option<Self> {
        self.pos_in_level()
            .move_along(dir)
            .map(|p| MazePosition::new(&p, self.level()))
    }
}

pub trait Maze {
    fn is_gate(&self, pos: &MazePosition) -> bool;
    fn can_pass(&self, pos: &MazePosition) -> bool;
    fn warp_pos(&self, pos: &MazePosition) -> Option<MazePosition>;
    fn start_pos(&self) -> &MazePosition;
    fn stop_pos(&self) -> &MazePosition;
}

#[derive(Debug)]
pub struct PlainMaze {
    map: MazeMap,
    start_pos: MazePosition,
    stop_pos: MazePosition,
}

impl Display for PlainMaze {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in &self.map {
            writeln!(f, "{}", row)?;
        }

        Ok(())
    }
}

impl Maze for PlainMaze {
    fn is_gate(&self, pos: &MazePosition) -> bool {
        if let Some(TileType::Gate(_, _)) = self.tile(pos) {
            true
        } else {
            false
        }
    }

    fn can_pass(&self, pos: &MazePosition) -> bool {
        self.tile(pos).is_some_and(|tt| tt.can_pass())
    }

    fn warp_pos(&self, pos: &MazePosition) -> Option<MazePosition> {
        if let Some(TileType::Gate(_, warp_pos_op)) = self.tile(pos) {
            // Gate in plain maze will warp to position in the same level.
            warp_pos_op
                .as_ref()
                .map(|warp_pos| MazePosition::new(warp_pos, pos.level()))
        } else {
            None
        }
    }

    fn start_pos(&self) -> &MazePosition {
        &self.start_pos
    }

    fn stop_pos(&self) -> &MazePosition {
        &self.stop_pos
    }
}

impl PlainMaze {
    pub(super) fn new(
        map: MazeMap,
        start_pos: &Position,
        start_name: &str,
        stop_pos: &Position,
        stop_name: &str,
    ) -> Self {
        let start_pos = MazePosition::new(start_pos, 0);
        let stop_pos = MazePosition::new(stop_pos, 0);
        let mut maze = Self {
            map,
            start_pos: start_pos.clone(),
            stop_pos: stop_pos.clone(),
        };
        assert!(maze
            .tile(&start_pos)
            .is_some_and(|tt| *tt == TileType::Empty));
        assert!(maze
            .tile(&stop_pos)
            .is_some_and(|tt| *tt == TileType::Empty));
        *maze.tile_mut(&start_pos).unwrap() = TileType::Gate(start_name.to_string(), None);
        *maze.tile_mut(&stop_pos).unwrap() = TileType::Gate(stop_name.to_string(), None);

        maze
    }

    pub fn tile(&self, pos: &MazePosition) -> Option<&TileType> {
        // Ignore level in plain maze.
        let pos = pos.pos_in_level();
        self.map.get(pos.r()).and_then(|row| row.tile(pos.c()))
    }

    pub fn tile_mut(&mut self, pos: &MazePosition) -> Option<&mut TileType> {
        // Ignore level in plain maze.
        let pos = pos.pos_in_level();
        self.map
            .get_mut(pos.r())
            .and_then(|row| row.tile_mut(pos.c()))
    }
}

pub struct RecursiveMaze {
    maze: PlainMaze,
}

impl Display for RecursiveMaze {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (r_ind, row) in self.maze.map.iter().enumerate() {
            let row_str = row
                .to_string()
                .chars()
                .enumerate()
                .map(|(c_ind, c)| {
                    let pos = MazePosition::new(&Position::new(c_ind, r_ind), 0);
                    if pos.pos_in_level() == self.start_pos().pos_in_level() {
                        'A'
                    } else if pos.pos_in_level() == self.stop_pos().pos_in_level() {
                        'Z'
                    } else if self.is_gate(&pos) {
                        assert!(c == 'O');
                        if self.is_at_inner_bound(&pos) {
                            'g'
                        } else if self.is_at_outer_bound(&pos) {
                            'G'
                        } else {
                            c
                        }
                    } else if c == 'O' && !self.can_pass(&pos) {
                        'X'
                    } else {
                        c
                    }
                })
                .collect::<String>();

            writeln!(f, "{}", row_str)?;
        }

        Ok(())
    }
}

impl Maze for RecursiveMaze {
    fn is_gate(&self, pos: &MazePosition) -> bool {
        if pos.level() == 0 {
            // If at top level, all outer gates except the start and stop gates are actually walls.
            self.start_pos() == pos
                || self.stop_pos() == pos
                || (self.is_at_inner_bound(pos) && self.maze.is_gate(pos))
        } else {
            // If position isn't at the top level, start and stop gate(both don't have warp positions) are actually walls.
            self.maze.is_gate(pos)
                && pos.pos_in_level() != self.start_pos().pos_in_level()
                && pos.pos_in_level() != self.stop_pos().pos_in_level()
        }
    }

    fn can_pass(&self, pos: &MazePosition) -> bool {
        if pos.level() == 0 {
            // If at top level, all outer gates except the start and stop gates are actually walls.
            let is_outer_warp_gate = pos != self.start_pos()
                && pos != self.stop_pos()
                && self.maze.is_gate(pos)
                && self.is_at_outer_bound(pos);
            self.maze.can_pass(pos) && !is_outer_warp_gate
        } else {
            // If position isn't at the top level, start and stop gate(both don't have warp positions) are actually walls.
            self.maze.can_pass(pos)
                && pos.pos_in_level() != self.start_pos().pos_in_level()
                && pos.pos_in_level() != self.stop_pos().pos_in_level()
        }
    }

    fn warp_pos(&self, pos: &MazePosition) -> Option<MazePosition> {
        if let Some(warp_pos) = self.maze.warp_pos(pos) {
            if pos.level() == 0 {
                if self.is_at_inner_bound(pos) {
                    Some(MazePosition::new(&warp_pos.pos_in_level(), pos.level() + 1))
                } else {
                    // If at outer bound, it's actually a wall, not a gate
                    None
                }
            } else {
                let warp_level = if self.is_at_outer_bound(pos) {
                    // Warp out.
                    pos.level() - 1
                } else if self.is_at_inner_bound(pos) {
                    // Warp in.
                    pos.level() + 1
                } else {
                    panic!(
                        "Found gate({}) is at neither inner nor outer bound of maze.",
                        pos
                    );
                };

                Some(MazePosition::new(&warp_pos.pos_in_level(), warp_level))
            }
        } else {
            None
        }
    }

    fn start_pos(&self) -> &MazePosition {
        self.maze.start_pos()
    }

    fn stop_pos(&self) -> &MazePosition {
        self.maze.stop_pos()
    }
}

impl RecursiveMaze {
    pub fn new(maze: PlainMaze) -> Self {
        Self { maze }
    }

    fn is_at_outer_bound(&self, pos: &MazePosition) -> bool {
        // Ignore level.
        let pos = pos.pos_in_level();
        // Top row.
        pos.r() == 0 ||
        // Bottom row.
        pos.r() == self.maze.map.len() - 1 ||
        // First or last tile in one row.
        self.maze.map.get(pos.r()).is_some_and(|row| {
            // Check the first tile position, skip any leading hole range
            row.ranges.iter().filter(|range| range.start_ind.is_some()).next().is_some_and(|r| r.range.start == pos.c()) ||
            // Check the last position of the last range(assume must be a tile range, that is, there is no any hole range at the end of any row).
            row.ranges.iter().next_back().is_some_and(|r| r.range.end - 1 == pos.c())
        })
    }

    fn is_at_inner_bound(&self, pos: &MazePosition) -> bool {
        // Ignore level.
        let pos = pos.pos_in_level();
        // Not the top row.
        pos.r() != 0 &&
        // Not the bottom row.
        pos.r() != self.maze.map.len() - 1 && (
            // Exactly at the boundary of any inner hole.
            self.maze.map.get(pos.r()).is_some_and(|row| {
                // Skip the first range, then check every inner hole if the position is at its boundary
                row.ranges.iter().skip(1).filter(|range| range.start_ind.is_none()).any(|r| (pos.c() == r.range.start - 1) || (pos.c() == r.range.end))
            }) ||
            // Is inside of any inner hole at the last or the next row.
            [pos.r() - 1, pos.r() + 1].iter().flat_map(|r_ind| self.maze.map.get(*r_ind)).any(|row| {
                // Skip the first range, then check every inner hole if the position is in this range.
                row.ranges.iter().skip(1).filter(|range| range.start_ind.is_none()).any(|r| r.range.contains(&pos.c()))
            })
        )
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
pub(super) struct MazeMapBuilder {
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
