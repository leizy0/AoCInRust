use std::{
    collections::HashMap,
    error,
    fmt::Display,
    fs::File,
    io::{self, BufRead, BufReader},
    path::{Path, PathBuf},
};

use clap::Parser;

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    InvalidRuleText(String),
    InvalidRuleID(String),
    RepeatRuleID(usize),
    UnknownRuleText(String),
    InvalidLiteralRuleText(String),
    InvalidConcatRuleText(String),
    InvalidOrRuleText(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IOError(ioe) => write!(f, "I/O error: {}", ioe),
            Error::InvalidRuleText(s) => write!(f, "Invalid rule text: {}", s),
            Error::InvalidRuleID(s) => write!(
                f,
                "Invalid text({}) for rule id, expect an unsigned number",
                s
            ),
            Error::RepeatRuleID(id) => {
                write!(f, "Rule ID {} occured again, which isn't allowed", id)
            }
            Error::UnknownRuleText(s) => write!(f, "Unknown rule text: {}", s),
            Error::InvalidLiteralRuleText(s) => write!(f, "Invalid literal rule text: {}", s),
            Error::InvalidConcatRuleText(s) => write!(f, "Invalid concatenation rule text: {}", s),
            Error::InvalidOrRuleText(s) => write!(f, "Invalid or rule text: {}", s),
        }
    }
}

impl error::Error for Error {}

#[derive(Debug, Parser)]
pub struct CLIArgs {
    pub input_path: PathBuf,
}

pub trait MMRule {
    fn check(&self, msg: &str, dict: &MMRules) -> Option<usize>;
}

struct LiteralRule {
    literal: String,
}

impl TryFrom<&str> for LiteralRule {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let quote = '"';
        let trimmed = value.trim();
        if trimmed.starts_with(quote) && trimmed.ends_with(quote) {
            Ok(Self {
                literal: trimmed[1..(trimmed.len() - 1)].to_string(),
            })
        } else {
            Err(Error::InvalidLiteralRuleText(value.to_string()))
        }
    }
}

impl MMRule for LiteralRule {
    fn check(&self, msg: &str, _dict: &MMRules) -> Option<usize> {
        if msg.starts_with(&self.literal) {
            Some(self.literal.len())
        } else {
            None
        }
    }
}

struct ConcatRule {
    inners: Vec<usize>,
}

impl TryFrom<&str> for ConcatRule {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Self {
            inners: value
                .split_whitespace()
                .map(|s| {
                    s.parse::<usize>()
                        .map_err(|_| Error::InvalidConcatRuleText(value.to_string()))
                })
                .collect::<Result<Vec<_>, Error>>()?,
        })
    }
}

impl MMRule for ConcatRule {
    fn check(&self, msg: &str, dict: &MMRules) -> Option<usize> {
        let mut cur_ind = 0;
        for inner in &self.inners {
            if let Some(ind) = dict
                .get(*inner)
                .and_then(|r| r.check(&msg[cur_ind..], dict))
            {
                cur_ind += ind;
            } else {
                return None;
            }
        }

        Some(cur_ind)
    }
}

struct OrRule {
    inners: Vec<Box<dyn MMRule>>,
}

impl TryFrom<&str> for OrRule {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Self {
            inners: value
                .split('|')
                .map(|s| {
                    ConcatRule::try_from(s)
                        .map_err(|_| Error::InvalidOrRuleText(value.to_string()))
                        .map(|cr| Box::new(cr) as Box<dyn MMRule>)
                })
                .collect::<Result<Vec<_>, Error>>()?,
        })
    }
}

impl MMRule for OrRule {
    fn check(&self, msg: &str, dict: &MMRules) -> Option<usize> {
        self.inners
            .iter()
            .filter_map(|inner| inner.check(msg, dict))
            .next()
    }
}

pub struct MMRules {
    rules: HashMap<usize, Box<dyn MMRule>>,
}

impl MMRules {
    pub fn get(&self, id: usize) -> Option<&dyn MMRule> {
        self.rules.get(&id).map(|bm| bm.as_ref())
    }

    fn new() -> Self {
        Self {
            rules: HashMap::new(),
        }
    }

    fn add(&mut self, text: &str) -> Result<(), Error> {
        let colon_ind = text
            .find(':')
            .ok_or(Error::InvalidRuleText(text.to_string()))?;
        let rule_id_text = &text[..colon_ind];
        let rule_id = rule_id_text
            .parse::<usize>()
            .map_err(|_| Error::InvalidRuleID(rule_id_text.to_string()))?;
        if self.rules.contains_key(&rule_id) {
            return Err(Error::RepeatRuleID(rule_id));
        }

        self.rules
            .insert(rule_id, parse_rule(&text[(colon_ind + 1)..])?);
        Ok(())
    }
}

pub fn read_info<P: AsRef<Path>>(path: P) -> Result<(MMRules, Vec<String>), Error> {
    let file = File::open(path).map_err(Error::IOError)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let mut rules = MMRules::new();
    while let Some(l) = lines.next() {
        let s = l.map_err(Error::IOError)?;
        if s.is_empty() {
            break;
        }

        rules.add(s.as_str())?;
    }

    let msgs = lines
        .map(|l| l.map_err(Error::IOError))
        .filter(|l| !l.as_ref().is_ok_and(|s| s.is_empty()))
        .collect::<Result<Vec<_>, Error>>()?;
    Ok((rules, msgs))
}

fn parse_rule(text: &str) -> Result<Box<dyn MMRule>, Error> {
    fn parse_rule_dyn<R>(text: &str) -> Result<Box<dyn MMRule>, Error>
    where
        R: MMRule + for<'a> TryFrom<&'a str, Error = Error> + 'static,
    {
        R::try_from(text).map(|r| Box::new(r) as Box<dyn MMRule>)
    }

    type MMRuleContorFn = fn(&str) -> Result<Box<dyn MMRule>, Error>;
    static MM_RULE_CONTORS: [MMRuleContorFn; 3] = [
        parse_rule_dyn::<LiteralRule>,
        parse_rule_dyn::<ConcatRule>,
        parse_rule_dyn::<OrRule>,
    ];

    MM_RULE_CONTORS
        .iter()
        .filter_map(|f| f(text).ok())
        .next()
        .ok_or(Error::UnknownRuleText(text.to_string()))
}
