use std::fmt::Display;

use int_enum::IntEnum;

pub enum Error {
    IncompleteTile(usize),
    InvalidTileId(i64),
    InvalidTilePos(i64, i64),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IncompleteTile(len) => write!(f, "Incomplete tile info(expects three integers) when parsing {}(th) tile", len + 1),
            Error::InvalidTileId(id) => write!(f, "Invalid tile id({}) encountered", id),
            Error::InvalidTilePos(x, y) => write!(f, "Invalid tile position({}, {}) encountered, expects a pair of non-negative integers", x, y),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy, IntEnum, PartialEq, Eq)]
pub enum TileId {
    #[default]
    Empty = 0,
    Wall = 1,
    Block = 2,
    HorizontalPaddle = 3,
    Ball = 4,
}

pub struct Tile {
    x: u32,
    y: u32,
    id: TileId,
}

impl Tile {
    fn from_info(x: i64, y: i64, id: i64) -> Result<Self, Error> {
        let x = u32::try_from(x).map_err(|_| Error::InvalidTilePos(x, y))?;
        let y = u32::try_from(y).map_err(|_| Error::InvalidTilePos(x as i64, y))?;
        let id = TileId::from_int(id as u8).map_err(|_| Error::InvalidTileId(id))?;

        Ok(Self { x, y, id })
    }
}

pub struct Screen {
    tiles: Vec<Tile>,
}

impl Screen {
    pub fn from_ints<I: Iterator<Item = i64>>(mut tiles_info: I) -> Result<Self, Error> {
        let mut tiles = Vec::new();
        loop {
            if let Some(x) = tiles_info.next() {
                let y = tiles_info
                    .next()
                    .ok_or(Error::IncompleteTile(tiles.len()))?;
                let tile_id = tiles_info
                    .next()
                    .ok_or(Error::IncompleteTile(tiles.len()))?;
                tiles.push(Tile::from_info(x, y, tile_id)?);
            } else {
                break;
            }
        }

        Ok(Self { tiles })
    }

    pub fn count_id(&self, id: TileId) -> usize {
        self.tiles.iter().filter(|t| t.id == id).count()
    }
}
