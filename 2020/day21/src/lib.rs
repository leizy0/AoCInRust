use std::{
    collections::{HashMap, HashSet, VecDeque},
    error,
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use clap::Parser;
use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Debug)]
pub enum Error {
    InvalidFoodText(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidFoodText(s) => write!(f, "Invalid text for food: {}", s),
        }
    }
}

impl error::Error for Error {}

#[derive(Debug, Parser)]
pub struct CLIArgs {
    pub input_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Food {
    ingrd_ids: HashSet<usize>,
    allrg_ids: HashSet<usize>,
}

impl Food {
    pub fn ingrd_ids(&self) -> impl Iterator<Item = &usize> {
        self.ingrd_ids.iter()
    }

    pub fn allrg_ids(&self) -> impl Iterator<Item = &usize> {
        self.allrg_ids.iter()
    }
}

pub struct FoodList {
    ingrd_names: Vec<String>,
    allrg_names: Vec<String>,
    foods: Vec<Food>,
}

impl FoodList {
    pub fn iter(&self) -> FoodIdIter {
        FoodIdIter::new(&self)
    }
}

pub struct FoodIdIter<'a> {
    foods: &'a FoodList,
    ind: usize,
}

impl<'a> Iterator for FoodIdIter<'a> {
    type Item = &'a Food;

    fn next(&mut self) -> Option<Self::Item> {
        let cur_ind = self.ind;
        if cur_ind < self.foods.foods.len() {
            self.ind += 1;
        }

        self.foods.foods.get(cur_ind)
    }
}

impl<'a> FoodIdIter<'a> {
    pub fn new(foods: &'a FoodList) -> Self {
        Self { foods, ind: 0 }
    }
}

struct FoodListBuilder {
    ingrd_names_map: HashMap<String, usize>,
    allrg_names_map: HashMap<String, usize>,
    foods: Vec<Food>,
}

impl FoodListBuilder {
    pub fn new() -> Self {
        Self {
            ingrd_names_map: HashMap::new(),
            allrg_names_map: HashMap::new(),
            foods: Vec::new(),
        }
    }

    pub fn add_food(&mut self, text: &str) -> Result<(), Error> {
        static FOOD_PATTERN: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"(?<ingrd_names>(?:\w+)(?:\s+\w+)*)\s+\(contains\s+(?<allrg_names>(?:\w+)(?:,\s+\w+)*)\)").unwrap()
        });
        fn map_to_ids_with_dict<'a>(
            names: impl Iterator<Item = &'a str>,
            dict: &mut HashMap<String, usize>,
        ) -> HashSet<usize> {
            let mut ids = HashSet::new();
            for name in names {
                if !dict.contains_key(name) {
                    let id = dict.len();
                    dict.insert(name.to_string(), id);
                }

                ids.insert(dict[name]);
            }

            ids
        }

        if let Some(caps) = FOOD_PATTERN.captures(text) {
            let info = Food {
                ingrd_ids: map_to_ids_with_dict(
                    caps["ingrd_names"].split_whitespace(),
                    &mut self.ingrd_names_map,
                ),
                allrg_ids: map_to_ids_with_dict(
                    caps["allrg_names"].split(',').map(|s| s.trim()),
                    &mut self.allrg_names_map,
                ),
            };
            self.foods.push(info);

            Ok(())
        } else {
            Err(Error::InvalidFoodText(text.to_string()))
        }
    }

    pub fn build(self) -> FoodList {
        fn dict_to_names(dict: HashMap<String, usize>) -> Vec<String> {
            let mut names = vec![String::new(); dict.len()];
            for (name, id) in dict {
                names[id] = name;
            }

            names
        }

        FoodList {
            ingrd_names: dict_to_names(self.ingrd_names_map),
            allrg_names: dict_to_names(self.allrg_names_map),
            foods: self.foods,
        }
    }
}

pub fn read_foods<P: AsRef<Path> + Copy>(path: P) -> Result<FoodList> {
    let file = File::open(path)
        .with_context(|| format!("Failed to open given file({}).", path.as_ref().display()))?;
    let reader = BufReader::new(file);
    let mut builder = FoodListBuilder::new();
    let mut lines = reader.lines();
    while let Some(l) = lines.next() {
        let s = l.with_context(|| {
            format!(
                "Failed to read line of given file({}).",
                path.as_ref().display()
            )
        })?;
        builder
            .add_food(&s)
            .with_context(|| format!("Failed to add food from string({})", s))?;
    }

    Ok(builder.build())
}

pub fn find_ingrd_allg_map(foods: &FoodList) -> HashMap<usize, usize> {
    let mut map = HashMap::new();
    let mut infos = foods.foods.iter().cloned().collect::<VecDeque<_>>();
    let mut unchange_acc_count = 0;
    while let Some(mut info) = infos.pop_front() {
        for (ingrd_id, allrg_id) in &map {
            if info.ingrd_ids.remove(ingrd_id) {
                info.allrg_ids.remove(allrg_id);
            }
        }

        if info.ingrd_ids.len() == 1 && info.allrg_ids.len() == 1 {
            // One to one map.
            map.entry(info.ingrd_ids.iter().copied().next().unwrap())
                .or_insert_with(|| info.ingrd_ids.iter().copied().next().unwrap());
        } else if !info.allrg_ids.is_empty() {
            let cur_info_len = infos.len();
            if unchange_acc_count >= cur_info_len {
                // No more change happened, can't find any more map information.
                break;
            }

            let mut has_changed = false;
            for other_ind in 0..cur_info_len {
                let allrg_intersec = info
                    .allrg_ids
                    .intersection(&infos[other_ind].allrg_ids)
                    .copied()
                    .collect::<HashSet<_>>();
                if !allrg_intersec.is_empty() {
                    let allrg_intersec_n = allrg_intersec.len();
                    let ingrd_intersec = info
                        .ingrd_ids
                        .intersection(&infos[other_ind].ingrd_ids)
                        .copied()
                        .collect::<HashSet<_>>();
                    let ingrd_intersec_n = ingrd_intersec.len();
                    if ingrd_intersec_n == allrg_intersec_n {
                        // n to n map.
                        if ingrd_intersec_n == 1 && allrg_intersec_n == 1 {
                            // One to one map.
                            let ingrd_id = ingrd_intersec.iter().copied().next().unwrap();
                            let allgr_id = allrg_intersec.iter().copied().next().unwrap();
                            map.entry(ingrd_id).or_insert(allgr_id);
                        }

                        ingrd_intersec.iter().for_each(|ingrd_id| {
                            info.ingrd_ids.remove(ingrd_id);
                        });
                        allrg_intersec.iter().for_each(|allrg_id| {
                            info.allrg_ids.remove(allrg_id);
                        });
                        has_changed = true;
                    } else {
                        // m to n map, m >= n.
                        assert!(ingrd_intersec_n >= allrg_intersec_n);
                        let new_info = Food {
                            ingrd_ids: ingrd_intersec,
                            allrg_ids: allrg_intersec,
                        };
                        if new_info != info && new_info != infos[other_ind] {
                            infos.push_back(new_info);
                            has_changed = true;
                        }
                    }
                }
            }

            if !info.allrg_ids.is_empty() {
                // Has allergy information left.
                infos.push_back(info);
            }

            if has_changed {
                unchange_acc_count = 0;
            } else {
                unchange_acc_count += 1;
            }
        }
    }

    map
}
