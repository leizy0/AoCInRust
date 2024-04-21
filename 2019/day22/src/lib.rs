use std::{
    collections::VecDeque,
    error,
    fmt::Display,
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};

use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    WrongNumberOfArgs(usize, usize),
    InvalidCut(String),
    InvalidIncrement(String),
    InvalidTechStr(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IOError(ioe) => write!(f, "I/O Error: {}", ioe),
            Error::WrongNumberOfArgs(args_n, expect_n) => write!(
                f,
                "Wrong numbers of arguments, given {}, expect {}",
                args_n, expect_n
            ),
            Error::InvalidCut(s) => write!(f, "Invalid cut description({})", s),
            Error::InvalidIncrement(s) => write!(f, "Invalid increment description({})", s),
            Error::InvalidTechStr(s) => write!(f, "Invalid shuffle technology description({})", s),
        }
    }
}

impl error::Error for Error {}

pub enum ShuffleTech {
    NewStack,
    Cut(i32),
    Increment(u32),
}

impl TryFrom<&str> for ShuffleTech {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        type TechConFn = fn(&str) -> Option<Result<ShuffleTech, Error>>;
        static TECH_CONSTRUCTORS: Lazy<Vec<TechConFn>> = Lazy::new(|| {
            vec![
                ShuffleTech::try_into_new_stack as TechConFn,
                ShuffleTech::try_into_cut as TechConFn,
                ShuffleTech::try_into_inc as TechConFn,
            ]
        });

        TECH_CONSTRUCTORS
            .iter()
            .flat_map(|c| c(value))
            .next()
            .unwrap_or(Err(Error::InvalidTechStr(value.to_string())))
    }
}

impl ShuffleTech {
    fn try_into_new_stack(s: &str) -> Option<Result<Self, Error>> {
        if s == "deal into new stack" {
            Some(Ok(Self::NewStack))
        } else {
            None
        }
    }

    fn try_into_cut(s: &str) -> Option<Result<Self, Error>> {
        static PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"cut (-?\d+)").unwrap());
        PATTERN.captures(s).map(|caps| {
            caps[1]
                .parse::<i32>()
                .map_err(|_| Error::InvalidCut(s.to_string()))
                .map(|n| Self::Cut(n))
        })
    }

    fn try_into_inc(s: &str) -> Option<Result<Self, Error>> {
        static PATTERN: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"deal with increment (\d+)").unwrap());
        PATTERN.captures(s).map(|caps| {
            caps[1]
                .parse::<u32>()
                .map_err(|_| Error::InvalidIncrement(s.to_string()))
                .map(|n| Self::Increment(n))
        })
    }
}

pub struct Deck {
    cards: VecDeque<u32>,
}

impl Deck {
    pub fn new(n: u32) -> Self {
        Self {
            cards: (0..n).collect::<VecDeque<_>>(),
        }
    }

    pub fn shuffle(&mut self, tech: &ShuffleTech) {
        match tech {
            ShuffleTech::NewStack => self.reverse(),
            ShuffleTech::Cut(i) => {
                if *i < 0 {
                    self.cards.rotate_right(usize::try_from(i.abs()).unwrap());
                } else if *i > 0 {
                    self.cards.rotate_left(usize::try_from(*i).unwrap());
                }
            }
            ShuffleTech::Increment(u) => self.increment(*u),
        }
    }

    pub fn find(&self, n: u32) -> Option<usize> {
        self.cards.iter().position(|c| *c == n)
    }

    fn reverse(&mut self) {
        let len = self.cards.len();
        for i in 0..((len + 1) / 2) {
            self.cards.swap(i, len - 1 - i);
        }
    }

    fn increment(&mut self, n: u32) {
        let cards_n = self.cards.len();
        let mut new_cards = VecDeque::new();
        new_cards.resize(cards_n, 0);

        let mut ind = 0;
        let mut new_ind = 0;
        while ind < cards_n {
            new_cards[new_ind] = self.cards[ind];
            new_ind = (new_ind + n as usize) % cards_n;
            ind += 1;
        }

        self.cards = new_cards;
    }
}

pub fn read_shuffle<P: AsRef<Path>>(path: P) -> Result<Vec<ShuffleTech>, Error> {
    let input_file = File::open(path).map_err(Error::IOError)?;
    let reader = BufReader::new(input_file);
    reader
        .lines()
        .map(|l| {
            l.map_err(Error::IOError)
                .and_then(|s| ShuffleTech::try_from(s.as_str()))
        })
        .collect()
}
