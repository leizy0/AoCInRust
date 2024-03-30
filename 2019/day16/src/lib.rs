use std::{
    cell::RefCell, error, fmt::Display, fs::File, io::{self, BufRead, BufReader}, path::Path, sync::{Arc, RwLock}
};

use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelBridge, ParallelIterator};

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


struct OffsetWndIter {
    offset: usize,
    wnd_width: usize,
    cycle_width: usize,
    end: usize,
    wnd_ind: usize,
    cycle_ind: usize,
}

impl OffsetWndIter {
    pub fn new(offset: usize, wnd_width: usize, cycle_width: usize, end: usize) -> Self {
        Self { offset, wnd_width, cycle_width, end, wnd_ind: 0, cycle_ind: 0 }
    }
}

impl Iterator for OffsetWndIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let ind = self.offset + self.cycle_width * self.cycle_ind + self.wnd_ind;
        self.wnd_ind = if self.wnd_ind + 1 < self.wnd_width {
            self.wnd_ind + 1
        } else {
            self.cycle_ind += 1;
            0
        };

        if ind < self.end {
            Some(ind)
        } else {
            None
        }
    }
}


struct Pattern {
    pat_ind: usize,
    signal_len: usize,
}

impl Pattern {
    pub fn new(pat_ind: usize, signal_len: usize) -> Self {
        Self { pat_ind, signal_len }
    }

    pub fn one_ind_iter(&self) -> impl Iterator<Item = usize> {
        OffsetWndIter::new(self.pat_ind, self.pat_ind + 1, (self.pat_ind + 1) * 4, self.signal_len)
    }

    pub fn neg_one_ind_iter(&self) -> impl Iterator<Item = usize> {
        OffsetWndIter::new(self.pat_ind + (self.pat_ind + 1) * 2, self.pat_ind + 1, (self.pat_ind + 1) * 4, self.signal_len)
    }
}



pub struct FFT {
    // outputs: Arc<RwLock<Vec<u32>>>,
    outputs: RefCell<Vec<u32>>,
}

impl FFT {
    pub fn new(signal_len: usize) -> Self {
        Self {
            // outputs: Arc::new(RwLock::new(vec![0; signal_len])),
            outputs: RefCell::new(vec![0; signal_len]),
        }
    }

    pub fn process_n(&self, signal: &mut [u32], phase_count: usize) -> Result<(), Error> {
        let signal_len = signal.len();
        // let outputs_len = self.outputs.read().unwrap().len();
        let outputs_len = self.outputs.borrow().len();
        if signal_len != outputs_len {
            return Err(Error::WrongSignalLen(signal_len, outputs_len));
        }

        // Process one phase.
        fn process(inputs: &[u32], outputs: &mut [u32]) {
            outputs.par_iter_mut().enumerate().for_each(|(ind, d)| {
                let pattern = Pattern::new(ind, inputs.len());
                let one_sum: i32 = pattern.one_ind_iter().par_bridge().map(|ind| inputs[ind] as i32).sum();
                let negative_one_sum: i32 = pattern.neg_one_ind_iter().par_bridge().map(|ind| inputs[ind] as i32).sum();
    
                *d = u32::try_from((one_sum - negative_one_sum).abs() % 10).unwrap();
            });
        }

        println!("Process begins.");
        for p_ind in 0..phase_count {
            if p_ind % 2 == 0 {
                // Even, signal -> self.outputs
                process(&signal, &mut self.outputs.borrow_mut());
            } else {
                // Odd, self.outputs -> signal
                process(&self.outputs.borrow(), signal);
            };

            println!("Phase {} completed.", p_ind);
        }

        if phase_count % 2 == 1 {
            // Odd, result is in self.outputs, copy back to given signal
            signal.copy_from_slice(&self.outputs.borrow());
        }
        println!("Process ends.");

        Ok(())
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
