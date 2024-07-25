use std::ops::Shr;

use once_cell::sync::Lazy;
use regex::Regex;

use crate::{Arrangement, Direction, Error, Pixel};

pub enum BorderConstraint<'a> {
    Unique(Direction, &'a [Tile]),
    Equal(Direction, TileBorder),
}

impl<'a> BorderConstraint<'a> {
    pub fn check<'b>(&self, check_tile: &ArrangedTile<'b>) -> bool {
        match self {
            BorderConstraint::Unique(check_dir, tiles) => {
                let unique_border = check_tile.border(*check_dir);
                tiles.iter().filter(|t| t.id != check_tile.id()).all(|t| {
                    Direction::all_dirs().iter().all(|dir| {
                        let test_border = t.border(*dir);
                        unique_border != test_border && unique_border != test_border.flip()
                    })
                })
            }
            BorderConstraint::Equal(check_dir, check_border) => {
                check_tile.border(*check_dir) == *check_border
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct TileBorder {
    value: usize,
    len: usize,
}

impl PartialEq for TileBorder {
    fn eq(&self, other: &Self) -> bool {
        if self.len != other.len {
            false
        } else {
            // Because the clockwise order, the border value bits must be flipped to check equality.
            self.flip().value == other.value
        }
    }
}
impl Eq for TileBorder {}

impl<'a, Iter: Iterator<Item = &'a Pixel>> From<Iter> for TileBorder {
    fn from(iter: Iter) -> Self {
        let (value, len) = iter.fold((0, 0), |(v, l), p| (v << 1 | *p as usize, l + 1));

        Self { value, len }
    }
}

impl TileBorder {
    pub fn flip(&self) -> TileBorder {
        Self {
            value: self
                .value
                .reverse_bits()
                .shr(usize::BITS as usize - self.len),
            len: self.len,
        }
    }
}

pub struct Tile {
    id: usize,
    pixels: Vec<Pixel>,
    rows_n: usize,
    cols_n: usize,
}

impl Tile {
    pub fn id(&self) -> usize {
        self.id
    }

    pub fn size(&self) -> (usize, usize) {
        (self.rows_n, self.cols_n)
    }

    fn border(&self, dir: Direction) -> TileBorder {
        // Collect border pixels according to clockwise order.
        match dir {
            Direction::Up => (0..self.cols_n).filter_map(|c| self.pixel(0, c)).into(),
            Direction::Right => (0..self.rows_n)
                .filter_map(|r| self.pixel(r, self.cols_n - 1))
                .into(),
            Direction::Down => (0..self.cols_n)
                .rev()
                .filter_map(|c| self.pixel(self.rows_n - 1, c))
                .into(),
            Direction::Left => (0..self.rows_n)
                .rev()
                .filter_map(|r| self.pixel(r, 0))
                .into(),
        }
    }

    pub fn pixel(&self, r: usize, c: usize) -> Option<&Pixel> {
        if r >= self.rows_n || c >= self.cols_n {
            None
        } else {
            self.pixels.get(r * self.cols_n + c)
        }
    }

    pub fn arrg_to_fit(&self, constraints: &[BorderConstraint]) -> Option<Arrangement> {
        Arrangement::all_arrgs()
            .iter()
            .filter(|arrg| {
                constraints
                    .iter()
                    .all(|c| c.check(&ArrangedTile::new(self, arrg)))
            })
            .next()
            .cloned()
    }
}

pub struct ArrangedTile<'a> {
    tile: &'a Tile,
    arrg: Arrangement,
    arrged_rows_n: usize,
    arrged_cols_n: usize,
}

impl<'a> ArrangedTile<'a> {
    pub fn new(tile: &'a Tile, arrg: &Arrangement) -> Self {
        let (org_rows_n, org_cols_n) = tile.size();
        let (arrged_rows_n, arrged_cols_n) = arrg.map_size(org_rows_n, org_cols_n);
        Self {
            tile,
            arrg: arrg.clone(),
            arrged_rows_n,
            arrged_cols_n,
        }
    }

    pub fn id(&self) -> usize {
        self.tile.id()
    }

    pub fn border(&self, mut dir: Direction) -> TileBorder {
        let flip = &self.arrg.flip;
        // Reverse map direction.
        dir = (-self.arrg.rotation).map_dir((-*flip).map_dir(dir));
        let border = self.tile.border(dir);
        if flip.flip_border() {
            border.flip()
        } else {
            border
        }
    }

    pub fn pixel(&self, r: usize, c: usize) -> Option<&'a Pixel> {
        self.arrg
            .rev_map_pos(r, c, self.arrged_rows_n, self.arrged_cols_n)
            .and_then(|(rev_r, rev_c)| self.tile.pixel(rev_r, rev_c))
    }
}

pub struct TileBuilder {
    id: usize,
    pixels: Vec<Pixel>,
    rows_n: usize,
    cols_n: Option<usize>,
}

impl TryFrom<&str> for TileBuilder {
    type Error = Error;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        static PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"Tile\s+(\d+):").unwrap());

        PATTERN
            .captures(value)
            .ok_or(Error::InvalidTileHeader(value.to_string()))
            .and_then(|caps| {
                caps[1]
                    .parse::<usize>()
                    .map_err(|_| Error::InvalidTileIDText(caps[1].to_string()))
                    .map(|id| TileBuilder::new(id))
            })
    }
}

impl TileBuilder {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            pixels: Vec::new(),
            rows_n: 0,
            cols_n: None,
        }
    }

    pub fn build(self) -> Tile {
        Tile {
            id: self.id,
            pixels: self.pixels,
            rows_n: self.rows_n,
            cols_n: self.cols_n.unwrap_or(0),
        }
    }

    pub fn add_row(&mut self, text: &str) -> Result<(), Error> {
        let this_col_n = text.chars().count();
        if *self.cols_n.get_or_insert(this_col_n) != this_col_n {
            return Err(Error::InconsistentColNum(this_col_n, self.cols_n.unwrap()));
        }

        for c in text.chars() {
            self.pixels.push(Pixel::try_from(c)?);
        }
        self.rows_n += 1;

        Ok(())
    }
}
