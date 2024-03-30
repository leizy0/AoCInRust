use std::{
    cell::RefCell,
    error,
    fmt::Display,
    fs::File,
    io::{self, BufRead, BufReader},
    iter,
    path::Path,
};

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    EmptyInput(String),
    InvalidInputChar(char),
    WrongSignalLen(usize, usize),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IOError(ioe) => write!(f, "I/O error: {}", ioe),
            Error::EmptyInput(path) => write!(f, "No inupt data in given file({})", path),
            Error::InvalidInputChar(c) => write!(
                f,
                "Invalid character({}) found in input, only digits are expected",
                c
            ),
            Error::WrongSignalLen(given_len, expected_len) => write!(
                f,
                "Given signal in wrong length({}), expected {}",
                given_len, expected_len
            ),
        }
    }
}

impl error::Error for Error {}

pub struct FFT {
    patterns: Vec<Vec<i32>>,
    outputs: RefCell<Vec<u32>>,
}

impl FFT {
    pub fn new(signal_len: usize) -> Self {
        Self {
            patterns: Self::gen_patterns(signal_len),
            outputs: RefCell::new(vec![0; signal_len]),
        }
    }

    pub fn process_n(&self, signal: &mut [u32], phase_count: usize) -> Result<(), Error> {
        let signal_len = signal.len();
        if signal_len != self.patterns.len() {
            return Err(Error::WrongSignalLen(signal_len, self.patterns.len()));
        }

        unsafe {
            let signal_ptr = signal.as_mut_ptr();
            let outputs_ptr = self.outputs.borrow_mut().as_mut_ptr();

            for p_ind in 0..phase_count {
                let (inputs, outputs) = if p_ind % 2 == 0 {
                    (signal_ptr, outputs_ptr)
                } else {
                    (outputs_ptr, signal_ptr)
                };

                for i in 0..signal_len {
                    let mut sum = 0;
                    let pattern_len = self.patterns[i].len();
                    for j in 0..signal_len {
                        sum += (*inputs.add(j) as i32) * self.patterns[i][(j + 1) % pattern_len];
                    }

                    *outputs.add(i) = u32::try_from(sum.abs() % 10).unwrap();
                }
            }
        }

        if phase_count % 2 == 1 {
            signal.copy_from_slice(&self.outputs.borrow());
        }

        Ok(())
    }

    fn gen_patterns(signal_len: usize) -> Vec<Vec<i32>> {
        (1..=signal_len)
            .map(|i| {
                iter::repeat(0)
                    .take(i)
                    .chain(iter::repeat(1).take(i))
                    .chain(iter::repeat(0).take(i))
                    .chain(iter::repeat(-1).take(i))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    }
}

pub fn read_signal<P: AsRef<Path>>(path: P) -> Result<Vec<u32>, Error> {
    let path_str = path
        .as_ref()
        .to_str()
        .map_or(String::new(), |s| s.to_string());
    let input_file = File::open(path).map_err(Error::IOError)?;
    let reader = BufReader::new(input_file);

    // Split the first line in file to digits, and propogate error in processing.
    reader
        .lines()
        .next()
        .ok_or(Error::EmptyInput(path_str))
        .and_then(|l| {
            l.map_err(Error::IOError).and_then(|s| {
                s.chars()
                    .map(|c| c.to_digit(10).ok_or(Error::InvalidInputChar(c)))
                    .collect::<Result<Vec<_>, _>>()
            })
        })
}
