use std::{
    collections::HashMap,
    error,
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use clap::Parser;

#[derive(Debug)]
pub enum Error {
    InvalidRuleText(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidRuleText(s) => write!(f, "Invalid rule text({}) for printer.", s),
        }
    }
}

impl error::Error for Error {}

#[derive(Debug, Parser)]
pub struct CLIArgs {
    pub input_path: PathBuf,
}

#[derive(Debug)]
pub struct PrinterRules {
    rules: HashMap<usize, Vec<usize>>,
}

impl PrinterRules {
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
        }
    }

    pub fn add_rule(&mut self, rule_text: &str) -> Result<(), Error> {
        let vert_line_ind = rule_text
            .find('|')
            .ok_or(Error::InvalidRuleText(rule_text.to_string()))?;
        let left_n = rule_text[..vert_line_ind]
            .parse::<usize>()
            .map_err(|_| Error::InvalidRuleText(rule_text.to_string()))?;
        let right_n = rule_text[(vert_line_ind + 1)..]
            .parse::<usize>()
            .map_err(|_| Error::InvalidRuleText(rule_text.to_string()))?;
        self.rules
            .entry(left_n)
            .or_insert_with(|| Vec::new())
            .push(right_n);

        Ok(())
    }

    pub fn is_valid(&self, update: &[usize]) -> bool {
        let page_n = update.len();
        for ind in (0..page_n).rev() {
            if let Some(after_pages_n) = self.rules.get(&update[ind]) {
                if update[..ind].iter().any(|n| after_pages_n.contains(n)) {
                    return false;
                }
            }
        }

        true
    }

    pub fn correct(&self, update: &[usize]) -> Vec<usize> {
        let mut corrected = Vec::from(update);
        let page_n = corrected.len();
        for ind in (0..page_n).rev() {
            while let Some(after_pages_n) = self.rules.get(&corrected[ind]) {
                let mut swap_ind = None;
                for (check_ind, n) in corrected[..ind].iter().enumerate() {
                    if after_pages_n.contains(n) {
                        swap_ind = Some(check_ind);
                        break;
                    }
                }

                if let Some(swap_ind) = swap_ind {
                    corrected.swap(swap_ind, ind);
                } else {
                    break;
                }
            }
        }

        corrected
    }
}

pub fn read_printer_settings<P: AsRef<Path>>(path: P) -> Result<(PrinterRules, Vec<Vec<usize>>)> {
    let file = File::open(&path)
        .with_context(|| format!("Failed to open given file({}).", path.as_ref().display()))?;
    let reader = BufReader::new(file);
    let mut rules = PrinterRules::new();
    let mut lines = reader.lines();
    let mut line_ind = 0;
    while let Some(line) = lines.next() {
        let line = line.with_context(|| {
            format!(
                "Failed to read line {} from given file({}).",
                line_ind + 1,
                path.as_ref().display()
            )
        })?;
        line_ind += 1;
        if line.is_empty() {
            break;
        }

        rules
            .add_rule(line.as_str())
            .with_context(|| format!("Failed to add printer rule text({}).", line))?;
    }

    let mut updates = Vec::new();
    while let Some(line) = lines.next() {
        let line = line.with_context(|| {
            format!(
                "Failed to read line {} from given file({}).",
                line_ind + 1,
                path.as_ref().display()
            )
        })?;
        line_ind += 1;
        let update = line
            .split(',')
            .map(|s| {
                s.parse::<usize>().with_context(|| {
                    format!("Failed to read page number from string({}) in update.", s)
                })
            })
            .collect::<Result<Vec<_>>>()?;
        updates.push(update);
    }

    Ok((rules, updates))
}
