#[macro_use]
extern crate lazy_static;
extern crate regex;

use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::Instant;

fn main() {
    let input_path = "input.txt";
    let input_file =
        File::open(input_path).expect(&format!("Failed to open input file({})", input_path));
    let input_lines: Vec<String> = BufReader::new(input_file)
        .lines()
        .map(|l| l.unwrap())
        .collect();

    let grow_rules = input_lines[2..]
        .iter()
        .fold(GrowRuleMap::new(), |mut map, x| {
            map.add_rule(x);
            map
        });
    let mut simulator = PlantSimulator::new(&input_lines[0], grow_rules).unwrap();

    let gen = 10000;
    let start_time = Instant::now();
    for i in 0..gen {
        println!("{}: generation[{}]", start_time.elapsed().as_secs(), i + 1);
        simulator.sim_one_gen();

        let ind_offset = simulator.ind_offset();
        let mut plant_ind_sum = 0i32;
        for (i, status) in simulator.cur_pots().enumerate() {
            if *status == PotStatus::Plant {
                plant_ind_sum += i as i32 + ind_offset;
            }
        }

        println!(
            "After {} generation, sum of index of all planted pots is {}",
            i + 1, plant_ind_sum
        );

        println!("Pots: [{}] @{}", simulator.pots_str(), simulator.ind_offset())
    }
}

#[derive(Copy, Clone, Hash, PartialEq, Debug)]
enum PotStatus {
    Plant,
    Empty,
}

impl Eq for PotStatus {}

impl PotStatus {
    pub fn from_char(desc: char) -> PotStatus {
        match desc {
            '#' => PotStatus::Plant,
            '.' => PotStatus::Empty,
            _ => panic!("Invalid pot status char({})", desc),
        }
    }
}

struct GrowRuleMap {
    map: HashMap<[PotStatus; GrowRuleMap::ORG_STATUS_SIZE], PotStatus>,
}

impl GrowRuleMap {
    const ORG_STATUS_SIZE: usize = 5;

    pub fn new() -> GrowRuleMap {
        GrowRuleMap {
            map: HashMap::new(),
        }
    }

    pub fn add_rule(&mut self, desc: &str) {
        lazy_static! {
            static ref RULE_PATTERN: Regex = Regex::new(r"([#\.]{5}) => ([#\.])").unwrap();
        }

        match RULE_PATTERN.captures(desc) {
            Some(caps) => {
                let mut pots = [PotStatus::Empty; 5];
                for (i, c) in caps[1].chars().enumerate() {
                    pots[i] = PotStatus::from_char(c);
                }

                let rule_res = PotStatus::from_char(caps[2].chars().next().unwrap());
                self.map.entry(pots).or_insert(rule_res);
            }
            None => println!("Failed to add rule, invalid description({})", desc),
        }
    }

    pub fn apply(&self, cur_status: &[PotStatus; 5]) -> PotStatus {
        *self.map.get(cur_status).expect(&format!(
            "Failed to find any exist rule to apply to current status({:?})",
            cur_status
        ))
    }

    pub fn apply_or(&self, cur_status: &[PotStatus; 5], def_status: PotStatus) -> PotStatus {
        match self.map.get(cur_status) {
            Some(&status) => status,
            None => def_status,
        }
    }
}

struct PlantSimulator {
    rules: GrowRuleMap,
    pots: Vec<PotStatus>,
    first_ind: i32,
}

impl PlantSimulator {
    pub fn new(init_desc: &str, rules_map: GrowRuleMap) -> Option<PlantSimulator> {
        lazy_static! {
            // initial state: ##.#.####..#####..#.....##....#.#######..#.#...........#......##...##.#...####..##.#..##.....#..####
            static ref INIT_STATUS_PATTERN: Regex = Regex::new(r"initial state: ([#\.]+)").unwrap();
        }

        assert!(
            rules_map.apply(&[
                PotStatus::Empty,
                PotStatus::Empty,
                PotStatus::Empty,
                PotStatus::Empty,
                PotStatus::Empty
            ]) != PotStatus::Plant
        );

        INIT_STATUS_PATTERN
            .captures(init_desc)
            .map(|caps| PlantSimulator {
                rules: rules_map,
                pots: caps[1].chars().map(|c| PotStatus::from_char(c)).collect(),
                first_ind: 0,
            })
    }

    pub fn sim_gen(&mut self, gen_n: u32) {
        (0..gen_n).for_each(|_| self.sim_one_gen());
    }

    pub fn sim_one_gen(&mut self) {
        let first_plant_ind = match self
            .pots
            .iter()
            .position(|&status| status == PotStatus::Plant)
        {
            Some(ind) => ind,
            None => return,
        };

        let last_plant_ind = self
            .pots
            .iter()
            .rposition(|&status| status == PotStatus::Plant)
            .unwrap();
        let start_ind = first_plant_ind as i32 - 2;
        let end_ind = last_plant_ind as i32 + 2;
        let mut next_pots = Vec::new();
        for i in start_ind..=end_ind {
            let mut cur_pots = [PotStatus::Empty; 5];
            (0..5).for_each(|offset| cur_pots[offset] = self.pot_status(i + (offset as i32) - 2));

            let next_status = self.rules.apply(&cur_pots);
            next_pots.push(next_status);
        }

        self.pots = next_pots;
        self.first_ind += start_ind;
    }

    pub fn cur_pots(&self) -> impl Iterator<Item = &PotStatus> {
        self.pots.iter()
    }

    pub fn ind_offset(&self) -> i32 {
        self.first_ind
    }

    pub fn pots_str(&self) -> String {
        let mut res = String::new();
        for status in &self.pots {
            res.push(match status {
                PotStatus::Plant => '1',
                PotStatus::Empty => '0',
            })
        }

        res
    }

    fn pot_status(&self, ind: i32) -> PotStatus {
        if ind < 0 || ind >= self.pots.len() as i32 {
            PotStatus::Empty
        } else {
            self.pots[ind as usize]
        }
    }
}
