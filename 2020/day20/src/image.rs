use std::collections::{HashMap, HashSet};

use anyhow::Context;

use crate::{
    tile::{ArrangedTile, BorderConstraint, Tile},
    Arrangement, Direction, Error, Pixel,
};

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

        let mapped_tile_refs = value.iter().map(|t| (t.id(), t)).collect::<HashMap<_, _>>();
        let mut left_tile_ids = value.iter().map(|t| t.id()).collect::<HashSet<_>>();
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
                left_tile_ids.remove(&tile.id());
                img_tiles.push(ImageTile::new(tile.id(), &arrg));
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
            tiles: value.into_iter().map(|t| (t.id(), t)).collect(),
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
