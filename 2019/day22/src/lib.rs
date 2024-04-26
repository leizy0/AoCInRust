use std::{
    cell::RefCell, collections::HashMap, env::args, error, fmt::Display, fs::File, io::{self, BufRead, BufReader}, path::Path
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

pub struct Deck {
    cards_n: usize,
    inc_map_to_cache: RefCell<HashMap<u32, Vec<Option<usize>>>>,
}

impl Deck {
    pub fn new(cards_n: usize) -> Self {
        Self {
            cards_n,
            inc_map_to_cache: RefCell::new(HashMap::new()),
        }
    }

    pub fn shuffle_map_from(&self, techs: &[ShuffleTech], mut origin_ind: usize) -> usize {
        for tech in techs.iter() {
            origin_ind = tech.map_from(origin_ind, self.cards_n);
        }

        origin_ind
    }

    pub fn shuffle_map_to(&self, techs: &[ShuffleTech], mut target_ind: usize) -> usize {
        for tech in techs.iter().rev() {
            match tech {
                ShuffleTech::Increment(u) => {
                    let inc_u = usize::try_from(*u).unwrap();
                    let ind_rem = target_ind % inc_u;
                    let start_ind = *self.inc_map_to_cache.borrow_mut().entry(*u).or_insert_with(|| vec![None; inc_u])[ind_rem].get_or_insert_with(|| tech.map_to(ind_rem, self.cards_n));
                    target_ind = target_ind / inc_u + start_ind;
                },
                other => target_ind = other.map_to(target_ind, self.cards_n),
            }
        }

        target_ind
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
