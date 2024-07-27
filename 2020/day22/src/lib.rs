use std::{
    collections::{HashSet, VecDeque},
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
    InvalidPlayerHeader(String),
    InvalidPlayerID(String),
    InvalidCardText(String),
    FoundSameCard(usize),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidPlayerHeader(s) => write!(f, "Invalid player header: {}", s),
            Error::InvalidPlayerID(s) => write!(f, "Invalid player ID: {}", s),
            Error::InvalidCardText(s) => write!(f, "Invalid card: {}", s),
            Error::FoundSameCard(c) => write!(f, "Found unexpected same card: {}", c),
        }
    }
}

impl error::Error for Error {}

#[derive(Debug, Parser)]
pub struct CLIArgs {
    pub input_path: PathBuf,
}

pub struct Player {
    id: usize,
    cards: Vec<usize>,
}

impl TryFrom<&str> for Player {
    type Error = Error;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        static PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"Player\s+(\d+):").unwrap());

        PATTERN
            .captures(value)
            .ok_or(Error::InvalidPlayerHeader(value.to_string()))
            .and_then(|caps| {
                caps[1]
                    .parse::<usize>()
                    .map(|id| Player::new(id))
                    .map_err(|_| Error::InvalidPlayerID(caps[1].to_string()))
            })
    }
}

impl Player {
    pub fn id(&self) -> usize {
        self.id
    }

    fn new(id: usize) -> Self {
        Self {
            id,
            cards: Vec::new(),
        }
    }

    fn add_card(&mut self, text: &str) -> Result<(), Error> {
        Ok(self.cards.push(
            text.parse::<usize>()
                .map_err(|_| Error::InvalidCardText(text.to_string()))?,
        ))
    }
}

pub struct CombatRes {
    pub turns_n: usize,
    pub winner: usize,
    pub winner_cards: VecDeque<usize>,
}

pub fn read_players<P: AsRef<Path>>(path: P) -> Result<Vec<Player>> {
    let file = File::open(path.as_ref())
        .with_context(|| format!("Failed to open given path: {}.", path.as_ref().display()))?;
    let reader = BufReader::new(file);
    let mut players = Vec::<Player>::new();
    let mut cur_player = None;
    for (l_ind, l) in reader.lines().enumerate() {
        let s = l.with_context(|| {
            format!(
                "Failed to read line #{} from given file: {}.",
                l_ind + 1,
                path.as_ref().display()
            )
        })?;
        if s.is_empty() {
            if let Some(player) = cur_player.take() {
                players.push(player);
            }

            continue;
        }

        if let Some(player) = cur_player.as_mut() {
            player.add_card(&s).with_context(|| {
                format!("Failed to add line #{}({}) as card to player.", l_ind, s)
            })?;
        } else {
            cur_player = Some(Player::try_from(s.as_str()).with_context(|| {
                format!("Failed to add line #{}({}) as card to player.", l_ind, s)
            })?);
        }
    }

    if let Some(player) = cur_player.take() {
        players.push(player);
    }

    Ok(players)
}

pub fn combat1(player0: &Player, player1: &Player) -> Result<CombatRes, Error> {
    let mut player0_cards = player0.cards.iter().copied().collect::<VecDeque<_>>();
    let mut player1_cards = player1.cards.iter().copied().collect::<VecDeque<_>>();
    let mut turns_n = 0;
    while !player0_cards.is_empty() && !player1_cards.is_empty() {
        let card0 = player0_cards.pop_front().unwrap();
        let card1 = player1_cards.pop_front().unwrap();
        if card0 > card1 {
            player0_cards.push_back(card0);
            player0_cards.push_back(card1);
        } else if card0 < card1 {
            player1_cards.push_back(card1);
            player1_cards.push_back(card0);
        } else {
            return Err(Error::FoundSameCard(card0));
        }

        turns_n += 1;
    }

    let (winner, winner_cards) = if player0_cards.is_empty() {
        (1, player1_cards)
    } else {
        (0, player0_cards)
    };

    Ok(CombatRes {
        turns_n,
        winner,
        winner_cards,
    })
}

pub fn combat2(player0: &Player, player1: &Player) -> Result<CombatRes, Error> {
    combat2_recur(
        player0.cards.iter(),
        player0.cards.len(),
        player1.cards.iter(),
        player1.cards.len(),
    )
}

fn combat2_recur<'a, I: Iterator<Item = &'a usize>>(
    cards0: I,
    cards0_n: usize,
    cards1: I,
    cards1_n: usize,
) -> Result<CombatRes, Error> {
    let mut player0_cards = cards0.take(cards0_n).copied().collect::<VecDeque<_>>();
    let mut player1_cards = cards1.take(cards1_n).copied().collect::<VecDeque<_>>();
    let mut turns_n = 0;
    let mut decks_set = HashSet::new();
    while !player0_cards.is_empty() && !player1_cards.is_empty() {
        if !decks_set.insert((player0_cards.clone(), player1_cards.clone())) {
            // Run into the infinite loop, player0 wins.
            return Ok(CombatRes {
                turns_n,
                winner: 0,
                winner_cards: player0_cards,
            });
        }

        let card0 = player0_cards.pop_front().unwrap();
        let card1 = player1_cards.pop_front().unwrap();
        let round_winner = if card0 <= player0_cards.len() && card1 <= player1_cards.len() {
            // Recursive game.
            let CombatRes {
                turns_n: recur_turns_n,
                winner,
                ..
            } = combat2_recur(player0_cards.iter(), card0, player1_cards.iter(), card1)?;
            turns_n += recur_turns_n;
            winner
        } else {
            // Normal game.
            if card0 > card1 {
                0
            } else if card1 > card0 {
                1
            } else {
                return Err(Error::FoundSameCard(card0));
            }
        };

        if round_winner == 0 {
            // Player 0 wins this round.
            player0_cards.push_back(card0);
            player0_cards.push_back(card1);
        } else {
            // Player 1 wins this round.
            player1_cards.push_back(card1);
            player1_cards.push_back(card0);
        }

        turns_n += 1;
    }

    let (winner, winner_cards) = if player0_cards.is_empty() {
        (1, player1_cards)
    } else {
        (0, player0_cards)
    };

    Ok(CombatRes {
        turns_n,
        winner,
        winner_cards,
    })
}
