use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    fs::File,
    io::{self, BufRead, BufReader},
    ops::Range,
    path::{Path, PathBuf},
    slice,
};

use clap::Parser;
use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    NoOwnTicket,
    InvalidFieldRule(String),
    InvalidTicketField(String),
    NullMappedTicketField(usize), // Field index
    NoMapOnTickets,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IOError(ioe) => write!(f, "I/O error: {}", ioe),
            Error::NoOwnTicket => write!(f, "No own ticket found"),
            Error::InvalidFieldRule(s) => write!(f, "Invalid filed rule: {}", s),
            Error::InvalidTicketField(s) => write!(f, "Invalid ticket filed: {}", s),
            Error::NullMappedTicketField(f_ind) => write!(f, "Value in field(#{}) can't be mapped(violates all known rules)", f_ind),
            Error::NoMapOnTickets => write!(f, "There's no map from ticket field to field rule that makes all tickets have an united field order according to given rules."),
        }
    }
}

#[derive(Debug, Parser)]
pub struct CLIArgs {
    pub input_path: PathBuf,
}

#[derive(Debug)]
pub struct FieldRule {
    name: String,
    ranges: Vec<Range<usize>>,
}

impl TryFrom<&str> for FieldRule {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        const RANGE_PATTERN_STR: &'static str = r"(\d+)-(\d+)";
        static RANGE_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(&RANGE_PATTERN_STR).unwrap());
        static RULE_PATTERN: Lazy<Regex> = Lazy::new(|| {
            Regex::new(&format!(
                r"(?<name>\w+(\s\w+)*):\s+(?<ranges>{0}(\s+or\s+{0})*)",
                RANGE_PATTERN_STR
            ))
            .unwrap()
        });

        RULE_PATTERN
            .captures(value)
            .ok_or(Error::InvalidFieldRule(value.to_string()))
            .map(|caps| {
                let name = caps["name"].to_string();
                let ranges = RANGE_PATTERN
                    .captures_iter(&caps["ranges"])
                    .map(|caps| {
                        caps[1].parse::<usize>().unwrap()..(caps[2].parse::<usize>().unwrap() + 1)
                    })
                    .collect();

                Self { name, ranges }
            })
    }
}

impl FieldRule {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn contains(&self, n: usize) -> bool {
        self.ranges.iter().any(|r| r.contains(&n))
    }
}

pub struct Ticket {
    fields: Vec<usize>,
}

impl TryFrom<&str> for Ticket {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Self {
            fields: value
                .split(',')
                .map(|s| {
                    s.parse::<usize>()
                        .map_err(|_| Error::InvalidTicketField(s.to_string()))
                })
                .collect::<Result<Vec<_>, Error>>()?,
        })
    }
}

impl<'a> IntoIterator for &'a Ticket {
    type Item = &'a usize;

    type IntoIter = slice::Iter<'a, usize>;

    fn into_iter(self) -> Self::IntoIter {
        self.fields.iter()
    }
}

pub fn read_ticket_info<P: AsRef<Path>>(
    path: P,
) -> Result<(Vec<FieldRule>, Ticket, Vec<Ticket>), Error> {
    let file = File::open(path).map_err(Error::IOError)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    // Field rules.
    let mut field_rules = Vec::new();
    while let Some(l) = lines.next() {
        let s = l.map_err(Error::IOError)?;
        if s.is_empty() {
            break;
        }

        field_rules.push(FieldRule::try_from(s.as_str())?);
    }

    // Your ticket.
    // Skip line: "your ticket:".
    lines.next();
    let own_ticket = lines
        .next()
        .ok_or(Error::NoOwnTicket)
        .and_then(|l| l.map_err(Error::IOError))
        .and_then(|s| Ticket::try_from(s.as_str()))?;

    // Other tickets
    let other_tickets = lines
        .skip(2)
        .map(|l| {
            l.map_err(Error::IOError)
                .and_then(|s| Ticket::try_from(s.as_str()))
        })
        .collect::<Result<Vec<_>, Error>>()?;
    Ok((field_rules, own_ticket, other_tickets))
}

pub fn map_field_with_tickets(
    rules: &[FieldRule],
    tickets: &[Ticket],
) -> Result<HashMap<String, usize>, Error> {
    let fields_n = rules.len();
    let mut prob_rule_inds_of_fields = vec![(0..fields_n).collect::<HashSet<usize>>(); fields_n];
    let mut temp_r_inds = Vec::<usize>::with_capacity(rules.len());
    for ticket in tickets {
        for (f_ind, field) in ticket.into_iter().enumerate() {
            let prob_rule_inds = &mut prob_rule_inds_of_fields[f_ind];
            temp_r_inds.clear();
            temp_r_inds.extend(prob_rule_inds.iter());
            for r_ind in temp_r_inds.iter().copied() {
                if !rules[r_ind].contains(*field) {
                    // Remove index of violated rule.
                    prob_rule_inds.remove(&r_ind);
                    if prob_rule_inds.is_empty() {
                        // One field doesn't have any rule that it always obey.
                        return Err(Error::NullMappedTicketField(f_ind));
                    }
                }
            }
        }
    }

    let mut field_rule_inds = vec![0; prob_rule_inds_of_fields.len()];
    if !unique_perm_in_cartessian(
        &prob_rule_inds_of_fields,
        &mut field_rule_inds,
        &mut HashSet::new(),
        0,
    ) {
        return Err(Error::NoMapOnTickets);
    }

    Ok(field_rule_inds
        .iter()
        .enumerate()
        .map(|(f_ind, r_ind)| (rules[*r_ind].name().to_string(), f_ind))
        .collect())
}

fn unique_perm_in_cartessian(
    num_sets: &Vec<HashSet<usize>>,
    perm: &mut Vec<usize>,
    used_nums: &mut HashSet<usize>,
    set_ind: usize,
) -> bool {
    if set_ind >= num_sets.len() {
        return true;
    }

    for n in num_sets[set_ind].iter() {
        if used_nums.insert(*n) {
            perm[set_ind] = *n;
            if unique_perm_in_cartessian(num_sets, perm, used_nums, set_ind + 1) {
                return true;
            }

            used_nums.remove(n);
        }
    }

    false
}
