use std::fmt::Display;

pub mod image;

#[derive(Debug)]
pub enum Error {
    IOError(std::io::Error),
    InvalidCharAsPixel(char, u32),
    NoImageInFile,
    DigitCountNotMatchImageDimension(usize, u32, u32),
    PixelValueExceedLimit(u32, u32),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IOError(ioe) => write!(f, "I/O Error: {}", ioe),
            Error::InvalidCharAsPixel(c, pixel_radix) => write!(
                f,
                "Invalid character({}) found, because pixel radix is {}",
                c, pixel_radix
            ),
            Error::NoImageInFile => write!(f, "No image in given file"),
            Error::DigitCountNotMatchImageDimension(count, width, height) => write!(
                f,
                "Given {} digits, but image is {} x {} which can't divide the count exactly",
                count, width, height
            ),
            Error::PixelValueExceedLimit(v, l) => write!(
                f,
                "Given pixel value {} that exceeds pixel value limit {}",
                v, l
            ),
        }
    }
}
