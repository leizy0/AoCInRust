use std::{
    collections::{HashMap, LinkedList},
    error,
    fmt::Display,
    fs::File,
    io::{self, BufRead, BufReader},
    mem,
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

trait MMRule {
    fn check(&self, msg: &str, dict: &MMRules, left_rules: &mut LinkedList<usize>)
        -> Option<usize>;
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
    fn check(
        &self,
        msg: &str,
        _dict: &MMRules,
        _left_rules: &mut LinkedList<usize>,
    ) -> Option<usize> {
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
    fn check(
        &self,
        msg: &str,
        dict: &MMRules,
        left_rules: &mut LinkedList<usize>,
    ) -> Option<usize> {
        let mut cur_ind = 0;
        self.inners
            .iter()
            .rev()
            .for_each(|id| left_rules.push_front(*id));
        while let Some(id) = left_rules.pop_front() {
            if let Some(len) = dict
                .get(id)
                .and_then(|r| r.check(&msg[cur_ind..], dict, left_rules))
            {
                cur_ind += len;
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
    fn check(
        &self,
        msg: &str,
        dict: &MMRules,
        left_rules: &mut LinkedList<usize>,
    ) -> Option<usize> {
        for inner in &self.inners {
            let mut this_left_rules = left_rules.clone();
            if let Some(len) = inner.check(msg, dict, &mut this_left_rules) {
                if len >= msg.len() {
                    debug_assert!(len == msg.len());
                    mem::swap(left_rules, &mut this_left_rules);
                    return Some(len);
                }
            }
        }

        None
    }
}

struct MMRules {
    rules: HashMap<usize, Box<dyn MMRule>>,
}

impl MMRules {
    fn new() -> Self {
        Self {
            rules: HashMap::new(),
        }
    }

    fn get(&self, id: usize) -> Option<&dyn MMRule> {
        self.rules.get(&id).map(|bm| bm.as_ref())
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

pub struct MMChecker {
    rules: MMRules,
    start_id: usize,
}

impl MMChecker {
    fn new(rules: MMRules, start_id: usize) -> Self {
        debug_assert!(
            rules.get(start_id).is_some(),
            "Can't find start rule in given rules."
        );
        Self { rules, start_id }
    }

    pub fn check(&self, msg: &str) -> bool {
        let mut left_rules = LinkedList::new();
        self.rules
            .get(self.start_id)
            .unwrap()
            .check(msg, &self.rules, &mut left_rules)
            .is_some_and(|len| len == msg.len())
    }
}

pub fn read_info<P: AsRef<Path>>(
    path: P,
    start_rule_id: usize,
) -> Result<(MMChecker, Vec<String>), Error> {
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
    let checker = MMChecker::new(rules, start_rule_id);

    let msgs = lines
        .map(|l| l.map_err(Error::IOError))
        .filter(|l| !l.as_ref().is_ok_and(|s| s.is_empty()))
        .collect::<Result<Vec<_>, Error>>()?;
    Ok((checker, msgs))
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
