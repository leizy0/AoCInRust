use std::collections::HashSet;

use crate::{image::SatelliteImage, Arrangement, Error, Pixel};

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

pub struct ImageMaskBuilder {
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
