use std::{
    cell::RefCell,
    collections::{BTreeSet, HashMap, HashSet, LinkedList},
    rc::Rc,
};

use crate::Direction;

use super::{Maze, MazePosition};

#[derive(Debug, Clone)]
pub enum MazeStep {
    Move(Direction),
    Warp(MazePosition),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct MazeWalkerState {
    pos: MazePosition,
    warped_pos: BTreeSet<MazePosition>,
}

#[derive(Debug, Clone)]
pub(crate) struct MazeWalker {
    state: MazeWalkerState,
    path: Vec<MazeStep>,
    visited_gates_pos: HashSet<MazePosition>,
    global_paths_for_gates:
        Rc<RefCell<HashMap<MazePosition, HashMap<MazePosition, Vec<MazeStep>>>>>,
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
    pub fn new(start_pos: &MazePosition) -> Self {
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

    pub fn pos(&self) -> &MazePosition {
        &self.state.pos
    }

    pub fn search_gates(&self, maze: &dyn Maze) -> Vec<MazePosition> {
        self.global_paths_for_gates
            .borrow_mut()
            .entry(self.state.pos.clone())
            .or_insert_with(|| self.bfs_for_gates(maze))
            .iter()
            .filter(|(pos, _)| !self.visited_gates_pos.contains(pos))
            .map(|(pos, _)| pos.clone())
            .collect()
    }

    pub fn move_and_warp(&mut self, maze: &dyn Maze, pos: &MazePosition) {
        if maze.is_gate(pos) {
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
                self.state.pos = if let Some(warp_pos) = maze.warp_pos(pos) {
                    // Do warp
                    self.state.warped_pos.insert(pos.clone());
                    self.visited_gates_pos.insert(warp_pos.clone());
                    self.path.push(MazeStep::Warp(warp_pos.clone()));
                    warp_pos
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

    fn bfs_for_gates(&self, maze: &dyn Maze) -> HashMap<MazePosition, Vec<MazeStep>> {
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
                if maze.is_gate(&next_pos) {
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
