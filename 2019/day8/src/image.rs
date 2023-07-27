use std::{
    fmt::Display,
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

    pub fn merge(&self) -> Layer {
        // pixel value is 0..pixel_radix, each value is one color except pixel_radix - 1 which is transparent.
        let mut layer = Layer::new_transparent(self.width, self.height, self.pixel_radix);
        for (ind, p) in layer.pixels_mut().iter_mut().enumerate() {
            *p = self
                .layers
                .iter()
                .map(|l| l.pixels()[ind])
                .find(|&p| p != self.pixel_radix - 1)
                .unwrap_or(*p)
        }

        layer
    }
}

pub struct Layer {
    pixels: Vec<u32>,
    width: u32,
    height: u32,
    pixel_radix: u32,
    digit_counts: Vec<u32>,
}

impl Display for Layer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in self.pixels.chunks(self.width as usize) {
            writeln!(
                f,
                "{}",
                row.iter()
                    .map(|&p| char::from_digit(p, self.pixel_radix).unwrap())
                    .collect::<String>()
            )?;
        }

        Ok(())
    }
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
            let digit_counts = Self::count_pixels(digits, pixel_radix)?;
            Ok(Self {
                pixels: Vec::from(digits),
                width,
                height,
                pixel_radix,
                digit_counts,
            })
        }
    }

    fn new_transparent(width: u32, height: u32, pixel_radix: u32) -> Layer {
        let pixels = vec![pixel_radix - 1; (width * height) as usize];
        let digit_counts = Self::count_pixels(&pixels, pixel_radix).unwrap();
        Self {
            pixels,
            width,
            height,
            pixel_radix,
            digit_counts,
        }
    }

    pub fn digit_count(&self, d: u32) -> Option<u32> {
        self.digit_counts.get(d as usize).copied()
    }

    fn pixels(&self) -> &[u32] {
        &self.pixels
    }

    fn pixels_mut(&mut self) -> &mut [u32] {
        &mut self.pixels
    }

    fn count_pixels(pixels: &[u32], pixel_radix: u32) -> Result<Vec<u32>, Error> {
        let mut digit_counts = vec![0u32; pixel_radix as usize];
        for &d in pixels {
            if d >= pixel_radix {
                return Err(Error::PixelValueExceedLimit(d, pixel_radix));
            } else {
                digit_counts[d as usize] += 1;
            }
        }

        Ok(digit_counts)
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
