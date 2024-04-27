use std::{
    cell::RefCell, env::args, error, fmt::Display, fs::File, io::{self, BufRead, BufReader}, iter, path::Path
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
    pub fn map_from(&self, ind: usize, cards_n: usize) -> usize {
        assert!(ind < cards_n);
        match self {
            ShuffleTech::NewStack => cards_n - 1 - ind,
            ShuffleTech::Cut(i) => Self::cut_map_from(*i, ind, cards_n),
            ShuffleTech::Increment(u) => Self::inc_map_from(*u, ind, cards_n),
        }
    }

    pub fn map_to(&self, ind: usize, cards_n: usize) -> usize {
        assert!(ind < cards_n);
        match self {
            ShuffleTech::NewStack => cards_n - 1 - ind,
            ShuffleTech::Cut(i) => Self::cut_map_from(-i, ind, cards_n),
            ShuffleTech::Increment(u) => Self::inc_map_to(*u, ind, cards_n),
        }
    }

    fn cut_map_from(cut_i: i32, ind: usize, cards_n: usize) -> usize {
        let i_abs = usize::try_from(cut_i.abs()).unwrap() % cards_n;
        if cut_i > 0 {
            if ind < i_abs {
                // Be cut, and appended to the end.
                ind + cards_n - i_abs
            } else {
                // not be cut, move forward.
                ind - i_abs
            }
        } else if cut_i < 0 {
            if ind < cards_n - i_abs {
                // not be cut, move backward.
                ind + i_abs
            } else {
                // Be cut, and inserted into start.
                ind - (cards_n - i_abs)
            }
        } else {
            ind
        }
    }

    fn inc_map_from(inc_u: u32, ind: usize, cards_n: usize) -> usize {
        ind * usize::try_from(inc_u).unwrap() % cards_n
    }

    fn inc_map_to(inc_u: u32, ind: usize, cards_n: usize) -> usize {
        let inc_u = usize::try_from(inc_u).unwrap();
        let cards_quo = cards_n / inc_u;
        let cards_rem = cards_n % inc_u;
        let target_rem = if ind % inc_u == 0 {
            0
        } else {
            inc_u - ind % inc_u
        };

        let (gcd, (_, mut c_factor)) = Self::pulverize(isize::try_from(inc_u).unwrap(), isize::try_from(cards_rem).unwrap());
        let lcm_c_factor = isize::try_from(inc_u / gcd).unwrap();
        c_factor *= isize::try_from(target_rem / gcd).unwrap();
        let wrap_count = if c_factor < 0 {
            usize::try_from(c_factor % lcm_c_factor + lcm_c_factor)
        } else {
            usize::try_from(c_factor % lcm_c_factor)
        }.unwrap();

        let origin_ind = (wrap_count * cards_rem + ind) / inc_u + wrap_count * cards_quo;
        debug_assert!(Self::inc_map_from(inc_u as u32, origin_ind, cards_n) == ind);

        origin_ind
    }

    fn pulverize(n: isize, d: isize) -> (usize, (isize, isize)) {
        let mut n = n.abs();
        let mut d = d.abs();
        let mut n_factor = (1, 0);
        let mut d_factor = (0, 1);

        while d != 0 {
            let quo = n / d;
            let rem = n % d;
            let new_d_factor = (n_factor.0 - quo * d_factor.0, n_factor.1 - quo * d_factor.1);
            n_factor = d_factor;
            d_factor = new_d_factor;
            n = d;
            d = rem;
        }
        
        (usize::try_from(n).unwrap(), n_factor)
    }

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

#[derive(Debug)]
pub struct ShuffleDeck {
    cards_n: usize,
    ind_factor: isize,
    constant: isize,
}

impl ShuffleDeck {
    pub fn new<'a, I: IntoIterator<Item = &'a ShuffleTech>>(iter: I, cards_n: usize) -> Self {
        let init = Self { cards_n, ind_factor: 1, constant: 0 };
        iter.into_iter().fold(init, |mut res, st| {
            res.combine(st);
            res
        })
    }

    pub fn ind_factor(&self) -> isize {
        self.ind_factor
    }

    pub fn constant(&self) -> isize {
        self.constant
    }

    pub fn combine(&mut self, st: &ShuffleTech) {
        match st {
            ShuffleTech::NewStack => {
                self.ind_factor = -self.ind_factor;
                self.constant = -self.constant - 1;
            },
            ShuffleTech::Cut(c) => self.constant -= isize::try_from(*c).unwrap(),
            ShuffleTech::Increment(i) => {
                let i = isize::try_from(*i).unwrap();
                let cards_n = isize::try_from(self.cards_n).unwrap();
                self.ind_factor *= i;
                self.ind_factor %= cards_n;
                self.constant *= i;
                self.constant %= cards_n;
            },
        }
    }

    pub fn map_from(&self, ind: usize) -> usize {
        assert!(ind < self.cards_n);
        let ind = i128::try_from(ind).unwrap();
        let cards_n = i128::try_from(self.cards_n).unwrap();
        let ind_factor = i128::try_from(self.ind_factor).unwrap();
        let constant = i128::try_from(self.constant).unwrap();
        let mut ind_rem = (ind * ind_factor + constant) % cards_n;
        if ind_rem < 0 {
            ind_rem += cards_n;
        }

        usize::try_from(ind_rem).unwrap()
    }

    pub fn map_to(&self, target_ind: usize) -> usize {
        assert!(target_ind < self.cards_n);
        let target_ind = isize::try_from(target_ind).unwrap();
        let cards_n = isize::try_from(self.cards_n).unwrap();
        let target_rem = (target_ind - self.constant) % cards_n;
        let (gcd, (_, i_factor)) = Self::pulverize(cards_n, self.ind_factor);
        let gcd = isize::try_from(gcd).unwrap();
        let mut i_factor = i128::try_from(i_factor).unwrap();
        i_factor *= i128::try_from(target_rem / gcd).unwrap();
        let lcm_i_factor = (cards_n / gcd).abs();
        let origin_ind = if i_factor < 0 {
            usize::try_from(isize::try_from(i_factor % i128::try_from(lcm_i_factor).unwrap()).unwrap() + lcm_i_factor)
        } else {
            usize::try_from(i_factor % i128::try_from(lcm_i_factor).unwrap())
        }.unwrap();
        
        debug_assert!(self.map_from(origin_ind) == usize::try_from(target_ind).unwrap());

        origin_ind
    }

    fn pulverize(mut n: isize, mut d: isize) -> (isize, (isize, isize)) {
        let mut n_factor = (1, 0);
        let mut d_factor = (0, 1);

        while d != 0 {
            let quo = n / d;
            let rem = n % d;
            let new_d_factor = (n_factor.0 - quo * d_factor.0, n_factor.1 - quo * d_factor.1);
            n_factor = d_factor;
            d_factor = new_d_factor;
            n = d;
            d = rem;
        }
        
        (n, n_factor)
    }
}

pub struct CachedShuffleDeck {
    cards_n: usize,
    decks: Vec<ShuffleDeck>,
    total_deck: ShuffleDeck,
    map_to_cache: RefCell<Vec<Vec<Option<usize>>>>,
}

impl CachedShuffleDeck {
    pub fn new<'a, I: IntoIterator<Item = &'a ShuffleTech>>(iter: I, cards_n: usize) -> Self {
        const FACTOR_LIMIT: isize = 1000000;
        let mut cur_deck = ShuffleDeck::new(iter::empty(), cards_n);
        let mut total_deck = ShuffleDeck::new(iter::empty(), cards_n);
        let mut decks = Vec::new();
        let mut map_to_cache = Vec::new();
        for tech in iter {
            cur_deck.combine(tech);
            total_deck.combine(tech);
            let cur_factor_abs = cur_deck.ind_factor().abs();
            if cur_factor_abs > FACTOR_LIMIT {
                decks.push(cur_deck);
                map_to_cache.push(vec![None; usize::try_from(cur_factor_abs).unwrap()]);
                cur_deck = ShuffleDeck::new(iter::empty(), cards_n);
            }
        }
        map_to_cache.push(vec![None; usize::try_from(cur_deck.ind_factor().abs()).unwrap()]);
        decks.push(cur_deck);

        Self { cards_n, decks, total_deck, map_to_cache: RefCell::new(map_to_cache) }
    }

    pub fn map_from(&self, ind: usize) -> usize {
        self.total_deck.map_from(ind)
    }

    pub fn map_to(&self, target_ind: usize) -> usize {
        let mut cur_ind = target_ind;
        for (d_ind, deck) in self.decks.iter().enumerate().rev() {
            let cache_ind = usize::try_from(isize::try_from(cur_ind).unwrap() % deck.ind_factor()).unwrap();
            let mapped_start_ind = *self.map_to_cache.borrow_mut()[d_ind][usize::try_from(cache_ind).unwrap()].get_or_insert_with(|| deck.map_to(cache_ind));
            let ind_offset = cur_ind / usize::try_from(deck.ind_factor().abs()).unwrap();
            let prev_ind = if deck.ind_factor() > 0 {
                if ind_offset + mapped_start_ind > self.cards_n {
                    ind_offset + mapped_start_ind - self.cards_n
                } else {
                    ind_offset + mapped_start_ind
                }
            } else {
                if mapped_start_ind < ind_offset {
                    self.cards_n - ind_offset + mapped_start_ind
                } else {
                    mapped_start_ind - ind_offset
                }
            };
            debug_assert!(deck.map_from(prev_ind) == cur_ind);
            cur_ind = prev_ind;
        }
        debug_assert!(self.map_from(cur_ind) == target_ind);

        cur_ind
    }
}

pub fn check_args() -> Result<String, Error> {
    let args_n = args().len();
    let expect_n = 2;
    if args_n != expect_n {
        eprintln!("Wrong number of arguments, expect one.");
        println!("Usage: {} INPUT_FILE_PATH", args().next().unwrap());
        Err(Error::WrongNumberOfArgs(args_n, expect_n))
    } else {
        Ok(args().skip(1).next().unwrap().to_string())
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
