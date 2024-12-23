use std::{
    collections::{HashMap, HashSet},
    error,
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use clap::Parser;

#[derive(Debug)]
enum Error {
    InvalidLinkText(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidLinkText(s) => {
                write!(f, "Invalid text({}) for a link, expect a hyphen in it.", s)
            }
        }
    }
}

impl error::Error for Error {}

#[derive(Debug, Parser)]
pub struct CLIArgs {
    pub input_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Computer {
    name: String,
}

impl Computer {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug)]
pub struct ComputerLinkGraph {
    link_map: HashMap<Computer, HashSet<Computer>>,
}

impl ComputerLinkGraph {
    pub fn new(links: &[(Computer, Computer)]) -> Self {
        let mut link_map = HashMap::new();
        for (left, right) in links {
            if left != right {
                link_map
                    .entry(left.clone())
                    .or_insert_with(|| HashSet::new())
                    .insert(right.clone());
                link_map
                    .entry(right.clone())
                    .or_insert_with(|| HashSet::new())
                    .insert(left.clone());
            }
        }

        Self { link_map }
    }

    pub fn all_3_groups(&self) -> HashSet<[Computer; 3]> {
        let mut groups = HashSet::new();
        for (computer, neighbors) in &self.link_map {
            for neighbor in neighbors {
                for dist_2_neighbor in self.link_map[neighbor]
                    .iter()
                    .filter(|dist_2_neighbor| *dist_2_neighbor != computer)
                {
                    if self.link_map[dist_2_neighbor].contains(computer) {
                        let mut group =
                            [computer.clone(), neighbor.clone(), dist_2_neighbor.clone()];
                        group.sort_unstable();
                        groups.insert(group);
                    }
                }
            }
        }

        groups
    }

    pub fn max_all_connected_groups(&self) -> HashSet<Vec<Computer>> {
        let mut groups = HashMap::new();
        for (computer, neighbors) in self.link_map.iter() {
            for neighbor in neighbors {
                let mut link_group = Vec::from([computer.clone(), neighbor.clone()]);
                link_group.sort_unstable();
                if !groups.contains_key(&link_group) {
                    let candidates = neighbors
                        .intersection(&self.link_map[neighbor])
                        .cloned()
                        .collect::<Vec<_>>();
                    groups.insert(link_group, candidates);
                }
            }
        }

        loop {
            let mut next_groups = HashMap::new();
            for (group, candidates) in groups.iter() {
                for computer in candidates {
                    let neighbors = &self.link_map[computer];
                    if !group.binary_search(computer).is_ok()
                        && group.iter().all(|member| neighbors.contains(member))
                    {
                        let mut new_group = group.clone();
                        new_group.push(computer.clone());
                        new_group.sort_unstable();
                        let new_candidates = candidates
                            .iter()
                            .filter(|computer| neighbors.contains(computer))
                            .cloned()
                            .collect();
                        next_groups.insert(new_group, new_candidates);
                    }
                }
            }

            if next_groups.len() > 0 {
                groups = next_groups;
            } else {
                break;
            }
        }

        groups.into_iter().map(|(group, _)| group).collect()
    }
}

pub fn read_links<P: AsRef<Path>>(path: P) -> Result<Vec<(Computer, Computer)>> {
    let file = File::open(&path)
        .with_context(|| format!("Failed to open given file({}).", path.as_ref().display()))?;
    let reader = BufReader::new(file);
    let mut links = Vec::new();
    for (ind, line) in reader.lines().enumerate() {
        let line = line.with_context(|| {
            format!(
                "Failed to read line {} in given file({}).",
                ind + 1,
                path.as_ref().display()
            )
        })?;
        let hyphen_ind = line.find('-').ok_or(Error::InvalidLinkText(line.clone()))?;
        links.push((
            Computer::new(&line[..hyphen_ind]),
            Computer::new(&line[(hyphen_ind + 1)..].to_string()),
        ));
    }

    Ok(links)
}
