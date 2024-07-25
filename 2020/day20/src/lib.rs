use std::{
    error,
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader},
    ops::Neg,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use clap::Parser;
use mask::{ImageMask, ImageMaskBuilder};
use once_cell::sync::Lazy;
use tile::{Tile, TileBuilder};

mod image;
mod mask;
mod tile;

pub use image::SatelliteImage;
pub use mask::ArrangedImageMask;

#[derive(Debug)]
pub enum Error {
    InvalidTileHeader(String),
    InvalidTileIDText(String),
    InconsistentColNum(usize, usize), // (number of columns in current row, expected column numbers in ealier rows).
    InvalidPixelChar(char),
    TileNotFound,
    InvalidDirDiscriminant(u8),
    InconsistentTileSize,
    TilesAreNotSquare,
    NoMaskPath,
    InvalidMaskChar(char),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidTileHeader(s) => write!(f, "Invalid tile header: {}", s),
            Error::InvalidTileIDText(s) => write!(f, "Invalid tile id: {}", s),
            Error::InconsistentColNum(this_cols_n, expect_cols_n) => write!(
                f,
                "Found inconsistent column number({}), expect {} columns according to earlier rows",
                this_cols_n, expect_cols_n
            ),
            Error::InvalidPixelChar(c) => write!(f, "Invalid character for pixel: {}", c),
            Error::TileNotFound => write!(f, "Failed to find tile fits given constraints."),
            Error::InvalidDirDiscriminant(dis) => write!(
                f,
                "Invalid discriminant({}) for Direction, expect [0, 3]",
                dis
            ),
            Error::InconsistentTileSize => write!(f, "Given tiles have inconsistent size."),
            Error::TilesAreNotSquare => write!(f, "Given tiles are not square, expect squares which can keep size when flip and rotate."),
            Error::NoMaskPath => write!(f, "No mask path found, expect a mask path to load image mask"),
            Error::InvalidMaskChar(c) => write!(f, "Invalid character for image mask: {}", c),
        }
    }
}
impl error::Error for Error {}

#[derive(Debug, Parser)]
pub struct CLIArgs {
    pub tiles_path: PathBuf,
    pub mask_path: Option<PathBuf>,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pixel {
    Black = 0,
    White = 1,
}

impl TryFrom<char> for Pixel {
    type Error = Error;

    fn try_from(value: char) -> std::result::Result<Self, Self::Error> {
        match value {
            '.' => Ok(Pixel::Black),
            '#' => Ok(Pixel::White),
            other => Err(Error::InvalidPixelChar(other)),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up = 0,
    Right = 1,
    Down = 2,
    Left = 3,
}

impl TryFrom<u8> for Direction {
    type Error = Error;

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        Ok(match value {
            0 => Direction::Up,
            1 => Direction::Right,
            2 => Direction::Down,
            3 => Direction::Left,
            other => return Err(Error::InvalidDirDiscriminant(other)),
        })
    }
}

impl Neg for Direction {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Direction::Up => Direction::Down,
            Direction::Right => Direction::Left,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
        }
    }
}

impl Direction {
    pub fn all_dirs() -> &'static [Direction] {
        static ALL_DIRECTIONS: [Direction; 4] = [
            Direction::Up,
            Direction::Right,
            Direction::Down,
            Direction::Left,
        ];

        &ALL_DIRECTIONS
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
enum Rotation {
    Clockwise0 = 0,
    Clockwise90 = 1,
    Clockwise180 = 2,
    Clockwise270 = 3,
}

impl Neg for Rotation {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Rotation::Clockwise0 => Rotation::Clockwise0,
            Rotation::Clockwise90 => Rotation::Clockwise270,
            Rotation::Clockwise180 => Rotation::Clockwise180,
            Rotation::Clockwise270 => Rotation::Clockwise90,
        }
    }
}

impl Rotation {
    pub fn all_rots() -> &'static [Rotation] {
        static ALL_ROTATIONS: [Rotation; 4] = [
            Rotation::Clockwise0,
            Rotation::Clockwise90,
            Rotation::Clockwise180,
            Rotation::Clockwise270,
        ];

        &ALL_ROTATIONS
    }

    pub fn map_dir(&self, dir: Direction) -> Direction {
        Direction::try_from((*self as u8 + dir as u8) % 4).unwrap()
    }

    pub fn map_size(&self, rows_n: usize, cols_n: usize) -> (usize, usize) {
        match self {
            Rotation::Clockwise0 | Rotation::Clockwise180 => (rows_n, cols_n),
            Rotation::Clockwise90 | Rotation::Clockwise270 => (cols_n, rows_n),
        }
    }

    pub fn map_pos(
        &self,
        r: usize,
        c: usize,
        rows_n: usize,
        cols_n: usize,
    ) -> Option<(usize, usize)> {
        if r >= rows_n || c >= cols_n {
            None
        } else {
            Some(match self {
                Rotation::Clockwise0 => (r, c),
                Rotation::Clockwise90 => (c, rows_n - 1 - r),
                Rotation::Clockwise180 => (rows_n - 1 - r, cols_n - 1 - c),
                Rotation::Clockwise270 => (cols_n - 1 - c, r),
            })
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Flip {
    NoFlip,
    UpDown,
    LeftRight,
    Both,
}

impl Neg for Flip {
    type Output = Self;

    fn neg(self) -> Self::Output {
        self
    }
}

impl Flip {
    pub fn all_flips() -> &'static [Flip] {
        static ALL_FLIPS: [Flip; 4] = [Flip::NoFlip, Flip::UpDown, Flip::LeftRight, Flip::Both];

        &ALL_FLIPS
    }

    pub fn flip_border(&self) -> bool {
        match self {
            Flip::UpDown | Flip::LeftRight => true,
            _ => false,
        }
    }

    pub fn map_pos(
        &self,
        r: usize,
        c: usize,
        rows_n: usize,
        cols_n: usize,
    ) -> Option<(usize, usize)> {
        if r >= rows_n || c >= cols_n {
            None
        } else {
            Some(match self {
                Flip::NoFlip => (r, c),
                Flip::UpDown => (rows_n - 1 - r, c),
                Flip::LeftRight => (r, cols_n - 1 - c),
                Flip::Both => (rows_n - 1 - r, cols_n - 1 - c),
            })
        }
    }

    pub fn map_dir(&self, dir: Direction) -> Direction {
        match self {
            Flip::UpDown if dir == Direction::Up || dir == Direction::Down => -dir,
            Flip::LeftRight if dir == Direction::Left || dir == Direction::Right => -dir,
            Flip::Both => -dir,
            _ => dir,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Arrangement {
    rotation: Rotation, // Apply firstly.
    flip: Flip,         // Apply secondly.
}

impl Arrangement {
    pub fn all_arrgs() -> &'static [Arrangement] {
        static ALL_ARRANGEMENTS: Lazy<Vec<Arrangement>> = Lazy::new(|| {
            Rotation::all_rots()
                .iter()
                .flat_map(|rot| {
                    Flip::all_flips().iter().map(|flip| Arrangement {
                        rotation: *rot,
                        flip: *flip,
                    })
                })
                .collect()
        });

        &ALL_ARRANGEMENTS
    }

    pub fn map_size(&self, rows_n: usize, cols_n: usize) -> (usize, usize) {
        self.rotation.map_size(rows_n, cols_n)
    }

    pub fn map_pos(
        &self,
        r: usize,
        c: usize,
        rows_n: usize,
        cols_n: usize,
    ) -> Option<(usize, usize)> {
        self.rotation
            .map_pos(r, c, rows_n, cols_n)
            .and_then(|(rot_r, rot_c)| {
                let (rot_rows_n, rot_cols_n) = self.rotation.map_size(rows_n, cols_n);
                self.flip.map_pos(rot_r, rot_c, rot_rows_n, rot_cols_n)
            })
    }

    fn rev_map_pos(
        &self,
        r: usize,
        c: usize,
        rows_n: usize,
        cols_n: usize,
    ) -> Option<(usize, usize)> {
        (-self.flip)
            .map_pos(r, c, rows_n, cols_n)
            .and_then(|(flip_r, flip_c)| (-self.rotation).map_pos(flip_r, flip_c, rows_n, cols_n))
    }
}

pub fn read_tiles<P: AsRef<Path>>(path: P) -> Result<Vec<Tile>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut numbered_lines = reader.lines().enumerate();
    let mut tiles = Vec::new();
    let mut builder: Option<TileBuilder> = None;
    while let Some((ind, l)) = numbered_lines.next() {
        let s = l.with_context(|| format!("Failed to read line {}.", ind + 1))?;
        if s.is_empty() {
            if let Some(builder) = builder.take() {
                tiles.push(builder.build());
            }
            continue;
        }

        if let Some(builder) = builder.as_mut() {
            builder
                .add_row(s.as_str())
                .context("Failed to add a new row to the building tile.")?;
        } else {
            builder = Some(
                TileBuilder::try_from(s.as_str())
                    .context("Failed to construct a new tile builder from id line.")?,
            );
        }
    }
    if let Some(builder) = builder.take() {
        tiles.push(builder.build());
    }

    Ok(tiles)
}

pub fn read_mask<P: AsRef<Path>>(path: P) -> Result<ImageMask> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut builder = ImageMaskBuilder::new();
    for l in reader.lines() {
        let s = l.context("Can't read rows of image mask.")?;
        builder
            .add_row(s.as_str())
            .context("Failed to read a row of the mask.")?;
    }

    Ok(builder.build())
}
