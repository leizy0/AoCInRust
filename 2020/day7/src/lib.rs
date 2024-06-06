use std::{
    collections::{HashMap, HashSet, LinkedList},
    error,
    fmt::Display,
    fs::File,
    io::{self, BufRead, BufReader},
    path::{Path, PathBuf},
};

use clap::Parser;
use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    InvalidBagRuleText(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IOError(ioe) => write!(f, "I/O error: {}", ioe),
            Error::InvalidBagRuleText(s) => write!(f, "Invalid bag rule({})", s),
        }
    }
}

impl error::Error for Error {}

#[derive(Debug, Parser)]
pub struct CliArgs {
    pub input_path: PathBuf,
}

pub struct BagRules {
    bag_qualifiers: HashMap<String, usize>, // <bag qualifier, bag index>
    contain_rules: Vec<LinkedList<(usize, usize)>>, // vec[index of bag contains] = list of (Index of bag be contained, count of contained bag).
    contained_rules: Vec<LinkedList<(usize, usize)>>, // vec[Index of bag be contained] = list of (index of bag contains, count of contained bag).
}

impl BagRules {
    pub fn contained_kinds_n(&self, qualifier: &str) -> Option<usize> {
        // BFS for bag nodes connected with given bag node in contained graph(self.contained_rules).
        self.bag_qualifiers.get(qualifier).map(|contained_ind| {
            let mut contain_inds = HashSet::new();
            let mut search_inds = LinkedList::new();
            search_inds.push_back(contained_ind);
            while let Some(ind) = search_inds.pop_front() {
                if contain_inds.contains(&ind) {
                    continue;
                }

                if ind != contained_ind {
                    contain_inds.insert(ind);
                }

                for (search_ind, _) in &self.contained_rules[*ind] {
                    search_inds.push_back(search_ind);
                }
            }

            contain_inds.len()
        })
    }

    fn new() -> Self {
        Self {
            bag_qualifiers: HashMap::new(),
            contain_rules: Vec::new(),
            contained_rules: Vec::new(),
        }
    }

    fn add_rule(&mut self, text: &str) -> Result<(), Error> {
        const CONTAINED_BAG_PATTERN_TEXT: &'static str = r"(\d+)\s+((\w+\s+)+\w+)\s+bags?";
        static CONTAIN_RULE_PATTERN: Lazy<Regex> = Lazy::new(|| {
            Regex::new(&format!(
                r"(.+)\s+bags\s+contain\s+({},?\s?)+\.",
                CONTAINED_BAG_PATTERN_TEXT
            ))
            .unwrap()
        });
        static EMPTY_RULE_PATTERN: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"(.+)\s+bags contain no other bags.").unwrap());
        static CONTAINED_BAG_PATTERN: Lazy<Regex> =
            Lazy::new(|| Regex::new(&CONTAINED_BAG_PATTERN_TEXT).unwrap());

        if let Some(caps) = CONTAIN_RULE_PATTERN.captures(text) {
            let contain_ind = self.get_or_add_bag(&caps[1]);
            for caps in CONTAINED_BAG_PATTERN.captures_iter(text) {
                let count = caps[1].parse::<usize>().unwrap();
                let contained_ind = self.get_or_add_bag(&caps[2]);
                self.contain_rules[contain_ind].push_back((contained_ind, count));
                self.contained_rules[contained_ind].push_back((contain_ind, count));
            }
        } else if let Some(caps) = EMPTY_RULE_PATTERN.captures(text) {
            self.get_or_add_bag(&caps[1]);
        } else {
            return Err(Error::InvalidBagRuleText(text.to_string()));
        }

        Ok(())
    }

    fn get_or_add_bag(&mut self, qualifier: &str) -> usize {
        *self
            .bag_qualifiers
            .entry(qualifier.to_string())
            .or_insert_with(|| {
                let ind = self.contain_rules.len();
                self.contain_rules.push(LinkedList::new());
                self.contained_rules.push(LinkedList::new());

                ind
            })
    }
}

pub fn read_br<P: AsRef<Path>>(path: P) -> Result<BagRules, Error> {
    let file = File::open(path).map_err(Error::IOError)?;
    let reader = BufReader::new(file);
    let mut rules = BagRules::new();
    for l in reader.lines() {
        let s = l.map_err(Error::IOError)?;
        rules.add_rule(&s)?;
    }

    Ok(rules)
}
