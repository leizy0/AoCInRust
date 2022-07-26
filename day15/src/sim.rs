use std::collections::{HashMap, VecDeque};
use std::fmt::Display;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum BGElem {
    Empty,
    Rock,
}

impl BGElem {
    fn is_blocked(&self) -> bool {
        match self {
            BGElem::Empty => false,
            BGElem::Rock => true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Map {
    bg: Vec<BGElem>,
    row_count: usize,
    col_count: usize,
}

impl Map {
    fn new(row_count: usize, col_count: usize) -> Map {
        Map {
            bg: vec![BGElem::Empty; row_count * col_count],
            row_count,
            col_count,
        }
    }

    fn put(&mut self, position: &Position, elem: BGElem) {
        self.bg[position.row_major_ind(self.col_count)] = elem;
    }

    fn at(&self, position: &Position) -> BGElem {
        self.bg[position.row_major_ind(self.col_count)]
    }

    fn can_stay(&self, position: &Position) -> bool {
        !self.at(position).is_blocked()
    }

    fn is_valid(&self, position: &Position) -> bool {
        position.r < self.row_count && position.c < self.col_count
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnitRace {
    Goblin,
    Elf,
}

impl Display for UnitRace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    r: usize,
    c: usize,
}

impl Position {
    fn new(row_ind: usize, col_ind: usize) -> Position {
        Position {
            r: row_ind,
            c: col_ind,
        }
    }

    fn row_major(ind: usize, col_count: usize) -> Position {
        Position {
            r: ind / col_count,
            c: ind % col_count,
        }
    }

    fn check_board_dist(&self, other: &Position) -> usize {
        let r_dist = if self.r < other.r {
            other.r - self.r
        } else {
            self.r - other.r
        };
        let c_dist = if self.c < other.c {
            other.c - self.c
        } else {
            self.c - other.c
        };
        r_dist + c_dist
    }

    // In reading order
    fn neighbor_4(&self) -> Vec<Position> {
        let mut res = Vec::with_capacity(4);
        if self.r > 0 {
            res.push(Position::new(self.r - 1, self.c));
        }
        if self.c > 0 {
            res.push(Position::new(self.r, self.c - 1));
        }
        res.push(Position::new(self.r, self.c + 1));
        res.push(Position::new(self.r + 1, self.c));
        res
    }

    fn row_major_ind(&self, col_count: usize) -> usize {
        self.r * col_count + self.c
    }
}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Position {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.r != other.r {
            self.r.cmp(&other.r)
        } else {
            self.c.cmp(&other.c)
        }
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "position({}, {})", self.r, self.c)
    }
}

const UNIT_DEFAULT_HEALTH: i32 = 200;
const UNIT_DEFAULT_ATTACK: i32 = 3;
#[derive(Debug, Clone, Copy)]
pub struct Unit {
    id: usize,
    race: UnitRace,
    health: i32,
    attack: i32,
    position: Position,
}

impl Unit {
    pub fn new(position: Position, race: UnitRace) -> Unit {
        static ID_COUNTER: AtomicUsize = AtomicUsize::new(0);
        Unit {
            id: ID_COUNTER.fetch_add(1, Ordering::Relaxed),
            race,
            health: UNIT_DEFAULT_HEALTH,
            attack: UNIT_DEFAULT_ATTACK,
            position,
        }
    }

    pub fn goblin(position: Position) -> Unit {
        Self::new(position, UnitRace::Goblin)
    }

    pub fn elf(position: Position) -> Unit {
        Self::new(position, UnitRace::Elf)
    }

    pub fn race(&self) -> UnitRace {
        self.race
    }

    pub fn health(&self) -> i32 {
        self.health
    }

    pub fn attack(&self) -> i32 {
        self.attack
    }

    pub fn is_dead(&self) -> bool {
        self.health <= 0
    }

    pub fn plan_phase(
        &self,
        phase: ActionPhase,
        map: &Map,
        living_units: &LivingUnits,
    ) -> Option<Action> {
        match phase {
            ActionPhase::Move => self.plan_move(map, living_units),
            ActionPhase::Attack => self.plan_attack(map, living_units),
        }
    }

    pub fn plan_move(&self, map: &Map, living_units: &LivingUnits) -> Option<Action> {
        let obstacle_map = ObstacleMap::new(map, living_units, self.id);
        let move_paths = living_units
            .units_of_race(self.enemy_race())
            .iter()
            .flat_map(|u| u.neighbors_in_range(map))
            .filter(|p| !obstacle_map.is_blocked(p))
            .map(|p| self.shortest_paths_to(&p, &obstacle_map))
            .filter(|op| op.is_some())
            .map(|op| op.unwrap())
            .collect::<Vec<_>>();
        if move_paths.is_empty() {
            // No open position to move to
            return None;
        }

        move_paths.iter().min_by_key(|p| p.len()).and_then(|p| {
            if p.len() > 0 {
                Some(Action::Move {
                    id: self.id,
                    to_position: p[0],
                })
            } else {
                None
            }
        })
    }

    fn is_in_range(&self, position: Position) -> bool {
        self.position.check_board_dist(&position) == 1
    }

    pub fn plan_attack(&self, map: &Map, living_units: &LivingUnits) -> Option<Action> {
        let mut neighbor_enemies = self
            .neighbors_in_range(map)
            .iter()
            .map(|p| {
                living_units
                    .find(p)
                    .and_then(|op| if op.race != self.race { Some(op) } else { None })
            })
            .filter(|op| op.is_some())
            .map(Option::unwrap)
            .collect::<Vec<_>>();
        if neighbor_enemies.is_empty() {
            return None;
        }

        neighbor_enemies.sort_unstable_by_key(|u| u.position);
        neighbor_enemies
            .iter()
            .min_by_key(|u| u.health())
            .map(|u| Action::Attack {
                attacker_id: self.id,
                attackee_id: u.id,
            })
    }

    fn neighbors_in_range(&self, map: &Map) -> Vec<Position> {
        self.position
            .neighbor_4()
            .iter()
            .filter(|p| map.is_valid(*p))
            .copied()
            .collect()
    }

    fn enemy_race(&self) -> UnitRace {
        match self.race {
            UnitRace::Elf => UnitRace::Goblin,
            UnitRace::Goblin => UnitRace::Elf,
        }
    }

    fn shortest_paths_to(
        &self,
        des_pos: &Position,
        obstacle_map: &ObstacleMap,
    ) -> Option<Vec<Position>> {
        let mut path_mat = vec![usize::MAX; obstacle_map.row_count * obstacle_map.col_count];
        let col_count = obstacle_map.col_count;
        let mut scan_queue = VecDeque::new();
        scan_queue.push_back(self.position.row_major_ind(col_count));
        while let Some(cur_pos_ind) = scan_queue.pop_front() {
            let cur_pos = Position::row_major(cur_pos_ind, col_count);
            if cur_pos == *des_pos {
                let mut path = Vec::new();
                let mut end_pos = cur_pos;
                while end_pos != self.position {
                    path.push(end_pos);
                    end_pos =
                        Position::row_major(path_mat[end_pos.row_major_ind(col_count)], col_count);
                }

                path.reverse();
                return Some(path);
            }

            let new_candidates = cur_pos.neighbor_4();
            let new_positions = new_candidates
                .iter()
                .filter(|p| !obstacle_map.is_blocked(p)) // Open place
                .filter(|p| path_mat[p.row_major_ind(col_count)] == usize::MAX)
                .collect::<Vec<_>>(); // Not visited

            for new_pos in new_positions {
                let ind = new_pos.row_major_ind(col_count);
                path_mat[ind] = cur_pos_ind;
                scan_queue.push_back(ind);
            }
        }

        None
    }
}

impl Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Unit({}, {}, {}, {})",
            self.id, self.race, self.position, self.health
        )
    }
}

pub struct LivingUnits<'a> {
    units: &'a Vec<Unit>,
    living_ids: Vec<usize>,
}

impl LivingUnits<'_> {
    pub fn new(units: &Vec<Unit>, mut living_ids: Vec<usize>) -> LivingUnits {
        living_ids.sort_unstable_by_key(|id| units[*id].position);
        LivingUnits { units, living_ids }
    }

    pub fn find(&self, position: &Position) -> Option<&Unit> {
        self.living_ids
            .binary_search_by_key(position, |id| self.units[*id].position)
            .ok()
            .map(|id_ind| &self.units[self.living_ids[id_ind]])
    }

    pub fn units_of_race(&self, race: UnitRace) -> Vec<&Unit> {
        self.living_ids
            .iter()
            .map(|id| &self.units[*id])
            .filter(|u| u.race == race)
            .collect()
    }
}

struct ObstacleMap {
    mask: Vec<bool>,
    row_count: usize,
    col_count: usize,
}

impl ObstacleMap {
    fn new(map: &Map, living_units: &LivingUnits, except_id: usize) -> ObstacleMap {
        let mask = map.bg.iter().map(|e| e.is_blocked()).collect::<Vec<_>>();

        let mut map = ObstacleMap {
            mask,
            row_count: map.row_count,
            col_count: map.col_count,
        };
        for living_id in living_units
            .living_ids
            .iter()
            .filter(|id| **id != except_id)
        {
            map.mask[living_units.units[*living_id]
                .position
                .row_major_ind(map.col_count)] = true;
        }

        map
    }

    fn is_blocked(&self, position: &Position) -> bool {
        position.r >= self.row_count
            || position.c >= self.col_count
            || self.mask[position.row_major_ind(self.col_count)]
    }
}

pub struct SimResult {
    pub winner: UnitRace,
    pub round_count: u32,
    pub units: Vec<Unit>,
}

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    UnknownChar {
        position: Position,
        c: char,
    },
    MoveOutOfRange {
        move_unit: Unit,
        to_position: Position,
    },
    MoveCanNotStay {
        move_unit: Unit,
        to_position: Position,
    },
    AttackNotInRange {
        attack_unit: Unit,
        attacked_unit: Unit,
    },
    AttackAlly {
        attack_unit: Unit,
        attacked_unit: Unit,
    },
    GhostAttacking {
        attack_unit: Unit,
        attacked_unit: Unit,
    },
    AttackDeadBody {
        attack_unit: Unit,
        attacked_unit: Unit,
    },
    NoIdInLiveUnits {
        dead_id: usize,
    },
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IOError(ioe) => write!(f, "{}", ioe),
            Error::UnknownChar { position, c } => write!(
                f,
                "Unknown char({}) found at row#{}, column#{}",
                c, position.r, position.c
            ),
            Error::MoveOutOfRange {
                move_unit,
                to_position,
            } => write!(
                f,
                "{:?} try move to {}, out of range!",
                move_unit, to_position
            ),
            Error::MoveCanNotStay {
                move_unit,
                to_position,
            } => write!(f, "{:?} try move to {}, blocked!", move_unit, to_position),
            Error::AttackNotInRange {
                attack_unit,
                attacked_unit,
            } => write!(
                f,
                "{:?} try attack {:?}, out of range!",
                attack_unit, attacked_unit
            ),
            Error::AttackAlly {
                attack_unit,
                attacked_unit,
            } => write!(
                f,
                "{:?} try attack {:?}, this is ally!",
                attack_unit, attacked_unit
            ),
            Error::GhostAttacking {
                attack_unit,
                attacked_unit,
            } => write!(
                f,
                "{:?} try attack {:?}, ghost is attacking!",
                attack_unit, attacked_unit
            ),
            Error::AttackDeadBody {
                attack_unit,
                attacked_unit,
            } => write!(
                f,
                "{:?} try attack {:?}, it's dead!",
                attack_unit, attacked_unit
            ),
            Error::NoIdInLiveUnits { dead_id } => {
                write!(f, "ID({}) not found in living units", dead_id)
            }
        }
    }
}

pub fn load_settings<P: AsRef<Path> + Display>(path: P) -> Result<(Map, Vec<Unit>), Error> {
    let input_file = File::open(path).map_err(|ioe| Error::IOError(ioe))?;
    let reader = BufReader::new(input_file);
    let lines = reader
        .lines()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|ioe| Error::IOError(ioe))?;
    let row_count = lines.len();
    let col_count = lines[0].chars().count();
    let mut map = Map::new(row_count, col_count);
    let mut units = Vec::new();
    for (row_ind, line) in lines.iter().enumerate() {
        for (col_ind, c) in line.chars().enumerate() {
            let position = Position::new(row_ind, col_ind);
            match c {
                '#' => map.put(&position, BGElem::Rock),
                '.' => map.put(&position, BGElem::Empty),
                'G' => units.push(Unit::goblin(position)),
                'E' => units.push(Unit::elf(position)),
                other => return Err(Error::UnknownChar { position, c: other }),
            }
        }
    }

    Ok((map, units))
}

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Move {
        id: usize,
        to_position: Position,
    },
    Attack {
        attacker_id: usize,
        attackee_id: usize,
    },
}

pub enum ActionPhase {
    Move,
    Attack,
}

#[derive(Debug, Clone)]
pub enum Cheat {
    SetElfAttack { attack: i32 },
}

#[derive(Debug, Clone)]
pub struct Simulator {
    map: Map,
    units: Vec<Unit>,
    living_unit_ids: Vec<usize>,
    cheats: Vec<Cheat>,
}

impl Simulator {
    pub fn new(map: Map, units: Vec<Unit>) -> Simulator {
        Simulator {
            map,
            living_unit_ids: (0..units.len()).collect::<Vec<_>>(),
            units,
            cheats: Vec::new(),
        }
    }

    pub fn add_cheat(&mut self, cheat: Cheat) {
        self.cheats.push(cheat);
    }

    pub fn simulate(self) -> Result<SimResult, Error> {
        self.simulate_with_cond(one_race_all_dead)
    }

    pub fn simulate_with_cond<F: Fn(&[Unit]) -> Option<UnitRace>>(
        mut self,
        end_cond: F,
    ) -> Result<SimResult, Error> {
        self.apply_cheats();
        let mut round_ind = 1u32;
        loop {
            println!("Round#{}", round_ind);
            self.log_map();
            self.living_unit_ids.sort_unstable_by(|l_id, r_id| {
                self.units[*l_id].position.cmp(&self.units[*r_id].position)
            });
            let left_unit_count = self.living_unit_ids.len();
            for (ind, id) in self.living_unit_ids.clone().into_iter().enumerate() {
                if self.units[id].is_dead() {
                    continue;
                }

                for phase in [ActionPhase::Move, ActionPhase::Attack] {
                    let living_units = LivingUnits::new(&self.units, self.living_unit_ids.clone());
                    if let Some(action) = self.units[id].plan_phase(phase, &self.map, &living_units)
                    {
                        self.log_action(&action);
                        self.execute(action)?;
                    }
                }

                let is_last = ind == left_unit_count - 1;
                if let Some(winner) = end_cond(&self.units) {
                    return Ok(SimResult {
                        winner,
                        round_count: if is_last { round_ind } else { round_ind - 1 },
                        units: self.units,
                    });
                }
            }

            round_ind += 1;
            println!();
        }
    }

    fn apply_cheats(&mut self) {
        for cheat in &self.cheats {
            match cheat {
                Cheat::SetElfAttack { attack } => {
                    self.units
                        .iter_mut()
                        .filter(|u| u.race() == UnitRace::Elf)
                        .for_each(|u| u.attack = *attack);
                }
            }
        }
    }

    fn execute(&mut self, action: Action) -> Result<(), Error> {
        match action {
            Action::Move { id, to_position } => {
                let move_unit = &mut self.units[id];
                if !move_unit.is_in_range(to_position) {
                    return Err(Error::MoveOutOfRange {
                        move_unit: *move_unit,
                        to_position,
                    });
                } else if !self.map.can_stay(&to_position) {
                    return Err(Error::MoveCanNotStay {
                        move_unit: *move_unit,
                        to_position,
                    });
                }

                move_unit.position = to_position;
            }
            Action::Attack {
                attacker_id,
                attackee_id,
            } => {
                if !self.units[attacker_id].is_in_range(self.units[attackee_id].position) {
                    return Err(Error::AttackNotInRange {
                        attack_unit: self.units[attacker_id],
                        attacked_unit: self.units[attackee_id],
                    });
                } else if self.units[attacker_id].race == self.units[attackee_id].race {
                    return Err(Error::AttackAlly {
                        attack_unit: self.units[attacker_id],
                        attacked_unit: self.units[attackee_id],
                    });
                } else if self.units[attackee_id].health <= 0 {
                    return Err(Error::GhostAttacking {
                        attack_unit: self.units[attacker_id],
                        attacked_unit: self.units[attackee_id],
                    });
                } else if self.units[attacker_id].health <= 0 {
                    return Err(Error::AttackDeadBody {
                        attack_unit: self.units[attacker_id],
                        attacked_unit: self.units[attackee_id],
                    });
                }

                self.units[attackee_id].health -= self.units[attacker_id].attack;
                if self.units[attackee_id].is_dead() {
                    let dead_id = self.units[attackee_id].id;
                    let dead_id_ind = self
                        .living_unit_ids
                        .iter()
                        .position(|id| *id == dead_id)
                        .ok_or(Error::NoIdInLiveUnits { dead_id })?;
                    self.living_unit_ids.remove(dead_id_ind);
                }
            }
        }

        Ok(())
    }

    fn log_map(&self) {
        let row_count = self.map.row_count;
        let col_count = self.map.col_count;
        let mut out_buffer: Vec<Vec<char>> = Vec::with_capacity(row_count + 2);
        for row_ind in 0..row_count {
            let row_beg = Position::new(row_ind, 0).row_major_ind(col_count);
            let row_end = Position::new(row_ind, col_count).row_major_ind(col_count);
            let mut row_buffer = self.map.bg[row_beg..row_end]
                .iter()
                .map(|e| match e {
                    BGElem::Empty => '.',
                    BGElem::Rock => '#',
                })
                .collect::<Vec<_>>();
            if row_ind != row_count - 1 {
                row_buffer.push('\n');
            }

            out_buffer.push(row_buffer);
        }

        for living_id in &self.living_unit_ids {
            let living_unit = &self.units[*living_id];
            out_buffer[living_unit.position.r][living_unit.position.c] = match living_unit.race {
                UnitRace::Elf => 'E',
                UnitRace::Goblin => 'G',
            };
        }

        println!("{}", out_buffer.into_iter().flatten().collect::<String>());
    }

    fn log_action(&self, action: &Action) {
        match action {
            Action::Move { id, to_position } => {
                println!("{} move to {}", self.units[*id], to_position)
            }
            Action::Attack {
                attacker_id,
                attackee_id,
            } => {
                let attacker = &self.units[*attacker_id];
                let attackee = &self.units[*attackee_id];
                println!("{} attack {})", attacker, attackee)
            }
        }
    }
}

fn one_race_all_dead(units: &[Unit]) -> Option<UnitRace> {
    let mut race_map = HashMap::new();
    for unit in units {
        let race_entry = race_map.entry(unit.race).or_insert(Vec::new());
        race_entry.push(unit);
    }

    for (race, units) in race_map {
        if units.into_iter().all(|u| u.is_dead()) {
            return Some(match race {
                UnitRace::Elf => UnitRace::Goblin,
                UnitRace::Goblin => UnitRace::Elf,
            });
        }
    }

    None
}
