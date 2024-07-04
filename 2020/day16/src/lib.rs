use std::{
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
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IOError(ioe) => write!(f, "I/O error: {}", ioe),
            Error::NoOwnTicket => write!(f, "No own ticket found"),
            Error::InvalidFieldRule(s) => write!(f, "Invalid filed rule: {}", s),
            Error::InvalidTicketField(s) => write!(f, "Invalid ticket filed: {}", s),
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
                        (caps[1].parse::<usize>().unwrap()..(caps[2].parse::<usize>().unwrap() + 1))
                    })
                    .collect();

                Self { name, ranges }
            })
    }
}

impl FieldRule {
    pub fn contain(&self, n: usize) -> bool {
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
