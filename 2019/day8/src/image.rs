use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use crate::Error;

pub struct Image {
    layers: Vec<Layer>,
    width: u32,
    height: u32,
    pixel_radix: u32,
}

impl Image {
    fn from_digits(
        digits: &[u32],
        width: u32,
        height: u32,
        pixel_radix: u32,
    ) -> Result<Self, Error> {
        let layer_digit_count = width * height;
        if digits.len() % (layer_digit_count as usize) != 0 {
            Err(Error::DigitCountNotMatchImageDimension(
                digits.len(),
                width,
                height,
            ))
        } else {
            let layers = digits
                .chunks(layer_digit_count as usize)
                .map(|w| Layer::from_digits(w, width, height, pixel_radix))
                .collect::<Result<Vec<_>, Error>>()?;
            Ok(Self {
                layers,
                width,
                height,
                pixel_radix,
            })
        }
    }

    pub fn layers(&self) -> &[Layer] {
        &self.layers
    }
}

pub struct Layer {
    pixels: Vec<u32>,
    width: u32,
    height: u32,
    pixel_radix: u32,
    digit_counts: Vec<u32>,
}

impl Layer {
    fn from_digits(
        digits: &[u32],
        width: u32,
        height: u32,
        pixel_radix: u32,
    ) -> Result<Self, Error> {
        let expect_digit_count = width * height;
        if digits.len() != expect_digit_count as usize {
            Err(Error::DigitCountNotMatchImageDimension(
                digits.len(),
                width,
                height,
            ))
        } else {
            let mut digit_counts = vec![0u32; pixel_radix as usize];
            for &d in digits {
                if d >= pixel_radix {
                    return Err(Error::PixelValueExceedLimit(d, pixel_radix));
                } else {
                    digit_counts[d as usize] += 1;
                }
            }

            Ok(Self {
                pixels: Vec::from(digits),
                width,
                height,
                pixel_radix,
                digit_counts,
            })
        }
    }

    pub fn digit_count(&self, d: u32) -> Option<u32> {
        self.digit_counts.get(d as usize).copied()
    }
}

pub fn read_image<P>(path: P, width: u32, height: u32, pixel_radix: u32) -> Result<Image, Error>
where
    P: AsRef<Path>,
{
    let file = File::open(path).map_err(Error::IOError)?;
    let reader = BufReader::new(file);
    reader
        .lines()
        .map(|lr| {
            lr.map_err(Error::IOError).and_then(|l| {
                let digits = l
                    .chars()
                    .map(|c| {
                        c.to_digit(pixel_radix)
                            .ok_or_else(|| Error::InvalidCharAsPixel(c, pixel_radix))
                    })
                    .collect::<Result<Vec<_>, Error>>()?;
                Image::from_digits(&digits, width, height, pixel_radix)
            })
        })
        .next()
        .unwrap_or(Err(Error::NoImageInFile))
}
