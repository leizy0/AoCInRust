use std::{
    collections::{HashMap, HashSet},
    error,
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader},
    ops::{Neg, Shr},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use clap::Parser;
use once_cell::sync::Lazy;
use regex::Regex;

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
enum Direction {
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

enum BorderConstraint<'a> {
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
struct TileBorder {
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

    fn size(&self) -> (usize, usize) {
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

    fn arrg_to_fit(&self, constraints: &[BorderConstraint]) -> Option<Arrangement> {
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

struct ArrangedTile<'a> {
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

struct TileBuilder {
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

pub struct ImageTile {
    id: usize,
    arrg: Arrangement,
}

impl ImageTile {
    pub fn id(&self) -> usize {
        self.id
    }

    fn new(id: usize, arrg: &Arrangement) -> Self {
        Self {
            id,
            arrg: arrg.clone(),
        }
    }
}

pub struct SatelliteImage {
    tiles: HashMap<usize, Tile>,
    img_tiles: Vec<ImageTile>,
    img_tile_rows_n: usize,
    img_tile_cols_n: usize,
    tile_pixel_rows_n: usize,
    tile_pixel_cols_n: usize,
    pixel_rows_n: usize,
    pixel_cols_n: usize,
}

impl TryFrom<Vec<Tile>> for SatelliteImage {
    type Error = anyhow::Error;

    fn try_from(value: Vec<Tile>) -> std::result::Result<Self, Self::Error> {
        // Check if all tiles are in the same size.
        if !value.is_empty() && !value.iter().skip(1).all(|t| t.size() == value[0].size()) {
            return Err(Error::InconsistentTileSize)
                .context("Can't construct image from tiles with inconsistent size.");
        }

        // Check if the same size forms a square.
        if !value.is_empty() {
            let (rows_n, cols_n) = value[0].size();
            if rows_n != cols_n {
                return Err(Error::TilesAreNotSquare)
                    .context("Can't construct image from tiles which aren't square.");
            }
        }

        let mapped_tile_refs = value.iter().map(|t| (t.id, t)).collect::<HashMap<_, _>>();
        let mut left_tile_ids = value.iter().map(|t| t.id).collect::<HashSet<_>>();
        let mut img_tiles = Vec::new();
        let mut row_ind = 0;
        let mut col_ind = 0;
        let mut img_tile_cols_n = None;
        // Start at top left corner, moves from left to right and up to down to find a proper tile fitting in the current location.
        loop {
            if left_tile_ids.is_empty() {
                if let Some(cols_n) = img_tile_cols_n.as_ref() {
                    if *cols_n != col_ind {
                        return Err(Error::InconsistentColNum(col_ind, *cols_n))
                            .context("Can't construct image from given tiles.");
                    } else {
                        // loop ends at the end of the last row, so add the line feed.
                        row_ind += 1;
                    }
                }
                break;
            }

            let up_constraint = if row_ind == 0 {
                // Top row.
                BorderConstraint::Unique(Direction::Up, &value)
            } else {
                // Lower rows.
                let upper_tile_ind = img_tile_cols_n.unwrap() * (row_ind - 1) + col_ind;
                let upper_image_tile: &ImageTile = &img_tiles[upper_tile_ind];
                let upper_tile = mapped_tile_refs[&upper_image_tile.id];
                BorderConstraint::Equal(
                    Direction::Up,
                    ArrangedTile::new(upper_tile, &upper_image_tile.arrg).border(Direction::Down),
                )
            };
            let left_constraint = if col_ind == 0 {
                // Left column.
                BorderConstraint::Unique(Direction::Left, &value)
            } else {
                // Right columns.
                let left_tile_ind = img_tile_cols_n.unwrap_or(0) * row_ind + col_ind - 1;
                let left_image_tile: &ImageTile = &img_tiles[left_tile_ind];
                let left_tile = mapped_tile_refs[&left_image_tile.id];
                BorderConstraint::Equal(
                    Direction::Left,
                    ArrangedTile::new(left_tile, &left_image_tile.arrg).border(Direction::Right),
                )
            };
            let tile_constraints = [up_constraint, left_constraint];

            if let Some((arrg, tile)) = left_tile_ids
                .iter()
                .map(|id| mapped_tile_refs[id])
                .filter_map(|t| t.arrg_to_fit(&tile_constraints).map(|arrg| (arrg, t)))
                .next()
            {
                left_tile_ids.remove(&tile.id);
                img_tiles.push(ImageTile::new(tile.id, &arrg));
                col_ind += 1;
            } else {
                // Not found, at the end of the current row.
                if let Some(cols_n) = img_tile_cols_n.as_ref() {
                    if *cols_n != col_ind {
                        return Err(Error::InconsistentColNum(col_ind, *cols_n))
                            .context("Can't construct image from given tiles.");
                    }

                    row_ind += 1;
                    col_ind = 0;
                } else {
                    img_tile_cols_n = Some(col_ind);
                }
            }
        }

        let img_tile_rows_n = row_ind;
        let img_tile_cols_n = img_tile_cols_n.unwrap_or(0);
        let (tile_pixel_rows_n, tile_pixel_cols_n) = value
            .get(0)
            .map(|t| t.size())
            .and_then(|(r, c)| {
                r.checked_sub(2)
                    .and_then(|pr| c.checked_sub(2).map(|pc| (pr, pc)))
            })
            .unwrap_or((0, 0));
        let pixel_rows_n = img_tile_rows_n * tile_pixel_rows_n;
        let pixel_cols_n = img_tile_cols_n * tile_pixel_cols_n;

        Ok(Self {
            tiles: value.into_iter().map(|t| (t.id, t)).collect(),
            img_tiles,
            img_tile_rows_n,
            img_tile_cols_n,
            tile_pixel_rows_n,
            tile_pixel_cols_n,
            pixel_rows_n,
            pixel_cols_n,
        })
    }
}

impl SatelliteImage {
    pub fn tile_size(&self) -> (usize, usize) {
        (self.img_tile_rows_n, self.img_tile_cols_n)
    }

    pub fn tile(&self, r: usize, c: usize) -> Option<&ImageTile> {
        if r >= self.img_tile_rows_n || c >= self.img_tile_cols_n {
            None
        } else {
            self.img_tiles.get(r * self.img_tile_cols_n + c)
        }
    }

    pub fn pixel_size(&self) -> (usize, usize) {
        (self.pixel_rows_n, self.pixel_cols_n)
    }

    pub fn pixel(&self, r: usize, c: usize) -> Option<&Pixel> {
        let img_tile_r_ind = r / self.tile_pixel_rows_n;
        let img_tile_c_ind = c / self.tile_pixel_cols_n;
        let tile_pixel_r_ind = r % self.tile_pixel_rows_n + 1; // +1 for skipping the border
        let tile_pixel_c_ind = c % self.tile_pixel_cols_n + 1; // +1 for skipping the border
        self.img_tiles
            .get(img_tile_r_ind * self.img_tile_cols_n + img_tile_c_ind)
            .and_then(|img_t| {
                self.tiles.get(&img_t.id).and_then(|t| {
                    ArrangedTile::new(t, &img_t.arrg).pixel(tile_pixel_r_ind, tile_pixel_c_ind)
                })
            })
    }
}

pub struct ImageMask {
    masked_pixels_pos: Vec<(usize, usize)>,
    rows_n: usize,
    cols_n: usize,
}

impl ImageMask {
    pub fn size(&self) -> (usize, usize) {
        (self.rows_n, self.cols_n)
    }
}

pub struct ArrangedImageMask {
    masked_pixels_pos: Vec<(usize, usize)>,
    rows_n: usize,
    cols_n: usize,
}

impl ArrangedImageMask {
    pub fn new(mask: &ImageMask, arrg: &Arrangement) -> Self {
        let (org_rows_n, org_cols_n) = mask.size();
        let (rows_n, cols_n) = arrg.map_size(org_rows_n, org_cols_n);
        let masked_pixels_pos = mask
            .masked_pixels_pos
            .iter()
            .map(|(r, c)| arrg.map_pos(*r, *c, org_rows_n, org_cols_n).unwrap())
            .collect::<Vec<_>>();
        Self {
            masked_pixels_pos,
            rows_n,
            cols_n,
        }
    }

    pub fn masked_pixels_pos(&self, image: &SatelliteImage) -> HashSet<(usize, usize)> {
        let mut masked_pos = HashSet::new();
        let (pixel_rows_n, pixel_cols_n) = image.pixel_size();
        for mask_r in 0..=(pixel_rows_n - self.rows_n) {
            for mask_c in 0..=(pixel_cols_n - self.cols_n) {
                if self.masked_pixels_pos.iter().all(|(r, c)| {
                    image
                        .pixel(*r + mask_r, *c + mask_c)
                        .is_some_and(|p| *p == Pixel::White)
                }) {
                    masked_pos.extend(
                        self.masked_pixels_pos
                            .iter()
                            .map(|(r, c)| (*r + mask_r, *c + mask_c)),
                    );
                }
            }
        }

        masked_pos
    }
}

struct ImageMaskBuilder {
    white_pixels_pos: Vec<(usize, usize)>,
    rows_n: usize,
    cols_n: Option<usize>,
}

impl ImageMaskBuilder {
    pub fn new() -> Self {
        Self {
            white_pixels_pos: Vec::new(),
            rows_n: 0,
            cols_n: None,
        }
    }

    pub fn add_row(&mut self, text: &str) -> Result<(), Error> {
        let this_cols_n = text.chars().count();
        if *self.cols_n.get_or_insert(this_cols_n) != this_cols_n {
            return Err(Error::InconsistentColNum(this_cols_n, self.cols_n.unwrap()));
        }

        for (c_ind, c) in text.chars().enumerate() {
            match c {
                '#' => self.white_pixels_pos.push((self.rows_n, c_ind)),
                ' ' => (),
                other => return Err(Error::InvalidMaskChar(other)),
            }
        }
        self.rows_n += 1;

        Ok(())
    }

    pub fn build(self) -> ImageMask {
        ImageMask {
            masked_pixels_pos: self.white_pixels_pos,
            rows_n: self.rows_n,
            cols_n: self.cols_n.unwrap_or(0),
        }
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
