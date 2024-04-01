use std::{error, fmt::Display};

#[derive(Debug)]
pub enum Error {
    IntCodeError(crate::Error),
    InconsistentMapRow(usize, usize),
    InvalidScaffoldMapValue(i64),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IntCodeError(e) => write!(f, "{}", e.to_string()),
            Error::InconsistentMapRow(this_col_n, last_col_n) => write!(f, "Found inconsistent row in scaffold map, this row has {} columns, the last one has {} columns", this_col_n, last_col_n),
            Error::InvalidScaffoldMapValue(v) => write!(f, "Invalid value({}) in scaffold map", v),
        }
    }
}

impl error::Error for Error {}

#[derive(Debug, PartialEq, Eq)]
enum TileType {
    Scaffold,
    Empty,
}

pub struct ScaffoldMap {
    map: Vec<TileType>,
    row_n: usize,
    col_n: usize,
}

impl ScaffoldMap {
    pub fn try_from_ints<I: Iterator<Item = i64>>(iter: I) -> Result<Self, Error> {
        let mut map = Vec::new();
        let mut row_n = 0;
        let mut col_n = 0;
        for (ind, v) in iter.enumerate() {
            match u32::try_from(v)
                .map_err(|_| Error::InvalidScaffoldMapValue(v))
                .and_then(|u| char::from_u32(u).ok_or(Error::InvalidScaffoldMapValue(v)))?
            {
                '#' | '^' | 'v' | '<' | '>' => map.push(TileType::Scaffold),
                '.' | 'X' => map.push(TileType::Empty),
                '\n' => {
                    if col_n == 0 {
                        col_n = ind;
                    }

                    let this_col_n = ind - (col_n + 1) * row_n;
                    if this_col_n != 0 {
                        // Ignore empty line.
                        row_n += 1;
                        if this_col_n != col_n {
                            return Err(Error::InconsistentMapRow(this_col_n, col_n));
                        }
                    }
                }
                _ => return Err(Error::InvalidScaffoldMapValue(v)),
            };
        }

        Ok(ScaffoldMap { map, row_n, col_n })
    }

    pub fn intersections(&self) -> Vec<(usize, usize)> {
        let mut res = Vec::new();
        for y in 1..(self.row_n - 1) {
            for x in 1..(self.col_n - 1) {
                if [(x, y), (x, y - 1), (x - 1, y), (x, y + 1), (x + 1, y)]
                    .iter()
                    .all(|(x, y)| self.map[self.pos_to_ind(*x, *y)] == TileType::Scaffold)
                {
                    res.push((x, y));
                }
            }
        }

        res
    }

    pub fn pos_to_ind(&self, x: usize, y: usize) -> usize {
        assert!(x < self.col_n && y < self.row_n);
        y * self.col_n + x
    }
}
