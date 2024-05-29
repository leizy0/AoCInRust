use std::{
    collections::HashMap,
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
    RepeatedPropInPassport(String), // Name of repeated property.
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IOError(ioe) => write!(f, "I/O error: {}", ioe),
            Error::RepeatedPropInPassport(name) => {
                write!(f, "Found repeated property({}) in passport text.", name)
            }
        }
    }
}

impl error::Error for Error {}

#[derive(Parser)]
pub struct CliArgs {
    pub input_path: PathBuf,
}

pub struct Passport {
    props: HashMap<String, String>,
}

impl Passport {
    pub fn new() -> Self {
        Self {
            props: HashMap::new(),
        }
    }

    pub fn contains_prop(&self, p_name: &str) -> bool {
        self.props.contains_key(p_name)
    }

    pub fn prop(&self, name: &str) -> Option<&str> {
        self.props.get(name).map(|sr| sr.as_str())
    }

    fn add_props(&mut self, text: &str) -> Result<(), Error> {
        static PROP_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\w+):([^\s]+)").unwrap());

        for caps in PROP_PATTERN.captures_iter(text) {
            if self.contains_prop(&caps[1]) {
                return Err(Error::RepeatedPropInPassport(caps[1].to_string()));
            } else {
                self.props.insert(caps[1].to_string(), caps[2].to_string());
            }
        }

        Ok(())
    }
}

pub trait PropValidator {
    fn name(&self) -> &'static str;
    fn validate(&self, s: &str) -> bool;
}

pub struct BirthYearVad {}
impl PropValidator for BirthYearVad {
    fn name(&self) -> &'static str {
        "byr"
    }

    fn validate(&self, s: &str) -> bool {
        static PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\d{4}$").unwrap());

        if PATTERN.is_match(s) {
            if let Ok(year) = s.parse::<usize>() {
                return year >= 1920 && year <= 2002;
            }
        }

        false
    }
}

pub struct IssueYearVad {}
impl PropValidator for IssueYearVad {
    fn name(&self) -> &'static str {
        "iyr"
    }

    fn validate(&self, s: &str) -> bool {
        static PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\d{4}$").unwrap());

        if PATTERN.is_match(s) {
            if let Ok(year) = s.parse::<usize>() {
                return year >= 2010 && year <= 2020;
            }
        }

        false
    }
}

pub struct ExpirYearVad {}
impl PropValidator for ExpirYearVad {
    fn name(&self) -> &'static str {
        "eyr"
    }

    fn validate(&self, s: &str) -> bool {
        static PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\d{4}$").unwrap());

        if PATTERN.is_match(s) {
            if let Ok(year) = s.parse::<usize>() {
                return year >= 2020 && year <= 2030;
            }
        }

        false
    }
}

pub struct HeightVad {}
impl PropValidator for HeightVad {
    fn name(&self) -> &'static str {
        "hgt"
    }

    fn validate(&self, s: &str) -> bool {
        static PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\d+)(cm|in)$").unwrap());

        if let Some(caps) = PATTERN.captures(s) {
            if let Ok(value) = caps[1].parse::<usize>() {
                return if &caps[2] == "cm" {
                    value >= 150 && value <= 193
                } else {
                    // "in"
                    value >= 59 && value <= 76
                };
            }
        }

        false
    }
}

pub struct HairColorVad {}
impl PropValidator for HairColorVad {
    fn name(&self) -> &'static str {
        "hcl"
    }

    fn validate(&self, s: &str) -> bool {
        static PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"^#[0-9a-f]{6}$").unwrap());

        PATTERN.is_match(s)
    }
}

pub struct EyeColorVad {}
impl PropValidator for EyeColorVad {
    fn name(&self) -> &'static str {
        "ecl"
    }

    fn validate(&self, s: &str) -> bool {
        static PATTERN: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"^amb|blu|brn|gry|grn|hzl|oth$").unwrap());

        PATTERN.is_match(s)
    }
}

pub struct PassportIDVad {}
impl PropValidator for PassportIDVad {
    fn name(&self) -> &'static str {
        "pid"
    }

    fn validate(&self, s: &str) -> bool {
        static PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\d{9}$").unwrap());

        PATTERN.is_match(s)
    }
}

pub fn read_pp<P: AsRef<Path>>(path: P) -> Result<Vec<Passport>, Error> {
    let file = File::open(path).map_err(Error::IOError)?;
    let reader = BufReader::new(file);
    let mut passports = Vec::new();
    let mut cur_pp = None;

    for line in reader.lines().map(|l| l.map_err(Error::IOError)) {
        let s = line?;
        if s.is_empty() {
            if let Some(pp) = cur_pp.take() {
                passports.push(pp);
            }
        } else {
            cur_pp
                .get_or_insert_with(|| Passport::new())
                .add_props(&s)?;
        }
    }

    // Remeber to push the last one!
    if let Some(pp) = cur_pp.take() {
        passports.push(pp);
    }

    Ok(passports)
}
