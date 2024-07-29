use std::{collections::HashSet, error, fmt::Display};

use anyhow::Context;
use clap::Parser;

#[derive(Debug)]
pub enum Error {
    InvalidCupSeqText(String),
    InvalidCupChar(char),
    RepeatCupID(usize),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidCupSeqText(s) => {
                write!(f, "Invalid string for cup sequence({}), expect digits", s)
            }
            Error::InvalidCupChar(c) => write!(f, "Invalid character for cup: {}", c),
            Error::RepeatCupID(id) => write!(f, "Found repeat cup id: {}", id),
        }
    }
}

impl error::Error for Error {}

#[derive(Debug, Parser)]
pub struct CLIArgs {
    pub init_cups: String,
}

pub struct CupGame {
    next_ids: Vec<usize>,
    cur_cup: usize,
    cup_n: usize,
}

impl TryFrom<&str> for CupGame {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let cup_n = value.chars().count();
        if cup_n >= 10 || cup_n == 0 {
            return Err(Error::InvalidCupSeqText(value.to_string())).with_context(|| {
                format!(
                    "Game building from string only supports at most 9 digits, given {}.",
                    cup_n
                )
            });
        }

        let mut next_ids = vec![0; cup_n + 1]; // Reserve element at 0.
        let mut cup_seen = HashSet::new();
        let mut start_cup = None;
        let mut last_cup = None;
        for (ind, c) in value.chars().enumerate() {
            let cup = c
                .to_digit(10)
                .map(|n| n as usize)
                .filter(|n| *n > 0 && *n <= cup_n)
                .ok_or(Error::InvalidCupChar(c))
                .with_context(|| format!("Failed to build cup game at index {}.", ind))?;
            start_cup.get_or_insert(cup);
            if !cup_seen.insert(cup) {
                return Err(Error::RepeatCupID(cup))
                    .with_context(|| format!("Failed to build cup game at index {}.", ind));
            }

            if let Some(last_cup) = last_cup.take() {
                next_ids[last_cup] = cup;
            }
            last_cup = Some(cup);
        }
        let start_cup = start_cup.unwrap();
        next_ids[last_cup.unwrap()] = start_cup;

        Ok(CupGame {
            next_ids,
            cur_cup: start_cup,
            cup_n,
        })
    }
}

impl CupGame {
    pub fn cup_n(&self) -> usize {
        self.cup_n
    }

    pub fn next(&self, id: usize) -> Option<usize> {
        self.next_ids.get(id).copied()
    }

    pub fn one_move(&mut self) {
        const MOVE_COUNT: usize = 3;
        assert!(self.cup_n >= MOVE_COUNT + 2);

        let mut dest_cup = self.cur_cup;
        loop {
            // Find the destination cup, current cup - 1 and wrap to the highest cup if the result is below the lowest one.
            dest_cup = (dest_cup + self.cup_n - 2) % self.cup_n + 1;
            let mut move_cup = self.cur_cup;
            let mut is_dest_ok = true;
            let mut move_front = None;
            // Check if the destination cup is in the moving sequence. If it is, try the next one.
            for _ in 0..MOVE_COUNT {
                move_cup = self.next(move_cup).unwrap();
                move_front.get_or_insert(move_cup);
                if dest_cup == move_cup {
                    is_dest_ok = false;
                    break;
                }
            }
            if !is_dest_ok {
                continue;
            }

            // Move to the destination.
            let move_front = move_front.unwrap();
            let move_rear = move_cup;
            let move_rear_next = self.next(move_rear).unwrap();
            let dest_next = self.next(dest_cup).unwrap();
            self.next_ids[self.cur_cup] = move_rear_next;
            self.next_ids[dest_cup] = move_front;
            self.next_ids[move_rear] = dest_next;
            break;
        }
        self.cur_cup = self.next(self.cur_cup).unwrap();
    }
}
