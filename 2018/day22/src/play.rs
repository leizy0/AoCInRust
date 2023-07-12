use std::{
    cmp::{Ordering, Reverse},
    collections::{BinaryHeap, HashMap},
    fmt::Display,
};

use once_cell::sync::Lazy;

use crate::{
    map::{CaveBlock, CaveMap},
    Error, Position,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tool {
    ClimbingGear,
    Torch,
    None,
}

static CAVE_TOOL_VALID_TABLE: Lazy<HashMap<(CaveBlock, Tool), bool>> = Lazy::new(|| {
    HashMap::from([
        ((CaveBlock::Rocky, Tool::ClimbingGear), true),
        ((CaveBlock::Rocky, Tool::Torch), true),
        ((CaveBlock::Rocky, Tool::None), false),
        ((CaveBlock::Wet, Tool::ClimbingGear), true),
        ((CaveBlock::Wet, Tool::Torch), false),
        ((CaveBlock::Wet, Tool::None), true),
        ((CaveBlock::Narrow, Tool::ClimbingGear), false),
        ((CaveBlock::Narrow, Tool::Torch), true),
        ((CaveBlock::Narrow, Tool::None), true),
    ])
});

impl Tool {
    pub fn is_valid_in(self, block: CaveBlock) -> bool {
        CAVE_TOOL_VALID_TABLE[&(block, self)]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Player {
    pos: Position,
    equip: Tool,
}

impl Display for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(pos: {}, equip: {:?})", self.pos, self.equip)
    }
}

impl Player {
    pub fn new(pos: &Position, tool: Tool) -> Player {
        Player {
            pos: *pos,
            equip: tool,
        }
    }

    pub fn next_actions(&self, map: &CaveMap) -> Vec<Action> {
        let mut actions = [Tool::ClimbingGear, Tool::Torch, Tool::None]
            .iter()
            .filter(|t| **t != self.equip && t.is_valid_in(map.at(&self.pos)))
            .map(|t| Action::Switch {
                from: self.equip,
                to: *t,
            })
            .collect::<Vec<_>>();

        actions.extend(
            [
                self.pos.up(),
                Some(self.pos.down()),
                self.pos.left(),
                Some(self.pos.right()),
            ]
            .iter()
            .filter_map(|op| op.filter(|p| self.equip.is_valid_in(map.at(&p))))
            .map(|p| Action::Move {
                from: self.pos,
                to: p,
            }),
        );
        actions
    }

    pub fn perform(&self, action: &Action) -> Player {
        match action {
            Action::Move { from, to } => {
                assert!(*from == self.pos);
                Player {
                    pos: *to,
                    equip: self.equip,
                }
            }
            Action::Switch { from, to } => {
                assert!(*from == self.equip);
                Player {
                    pos: self.pos,
                    equip: *to,
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Move { from: Position, to: Position },
    Switch { from: Tool, to: Tool },
}

impl Action {
    pub fn cost(&self) -> usize {
        match self {
            Action::Move { .. } => 1,
            Action::Switch { .. } => 7,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ActionRecord {
    from_player: Player,
    action: Action,
}

#[derive(Debug, Eq)]
struct State {
    cost: usize,
    player: Player,
    record: ActionRecord,
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.cost.cmp(&other.cost)
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

pub fn fastest_plan_to(
    init_player: &Player,
    map: &CaveMap,
    end_player: &Player,
) -> Result<Vec<Action>, Error> {
    let mut state_heap = BinaryHeap::from([Reverse(State {
        cost: 0,
        player: *init_player,
        record: ActionRecord {
            from_player: *init_player,
            action: Action::Move {
                from: init_player.pos,
                to: init_player.pos,
            },
        },
    })]);
    let mut record_map = HashMap::new();

    while let Some(Reverse(state)) = state_heap.pop() {
        if record_map.contains_key(&state.player) {
            continue;
        }

        record_map.insert(state.player, state.record);
        if state.player == *end_player {
            break;
        }

        for action in state.player.next_actions(map) {
            let next_player = state.player.perform(&action);
            if !record_map.contains_key(&next_player) {
                let record = ActionRecord {
                    from_player: state.player,
                    action,
                };
                state_heap.push(Reverse(State {
                    cost: state.cost + action.cost(),
                    player: next_player,
                    record,
                }));
            }
        }
    }

    if !record_map.contains_key(end_player) {
        return Err(Error::UnreachableTarget(*end_player));
    }

    let mut action_plan = Vec::new();
    let mut cur_player = end_player;
    while cur_player != init_player {
        let record = &record_map[cur_player];
        action_plan.push(record.action);
        cur_player = &record.from_player;
    }

    action_plan.reverse();
    Ok(action_plan)
}
