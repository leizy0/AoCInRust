use std::{
    cell::RefCell,
    error,
    fmt::Display,
    fs::File,
    io::{self, BufRead, BufReader},
    ops::{Index, IndexMut},
    path::Path,
};

use rayon::iter::{
    IndexedParallelIterator, IntoParallelRefMutIterator, ParallelBridge, ParallelIterator,
};

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    EmptyInput(String),
    InvalidInputChar(char),
    WrongSignalLen(usize, usize),
    ParitalNotInclude(usize, usize, usize),
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
            Error::ParitalNotInclude(given_ind, offset, total_len) => write!(
                f,
                "Given index({}) is out of expected range([{}, {}))",
                given_ind, offset, total_len
            ),
        }
    }
}

impl error::Error for Error {}

struct OffsetWndIndexIter {
    offset: usize,
    wnd_width: usize,
    cycle_width: usize,
    end: usize,
    wnd_ind: usize,
    cycle_ind: usize,
}

impl OffsetWndIndexIter {
    pub fn new(offset: usize, wnd_width: usize, cycle_width: usize, end: usize) -> Self {
        Self {
            offset,
            wnd_width,
            cycle_width,
            end,
            wnd_ind: 0,
            cycle_ind: 0,
        }
    }
}

impl Iterator for OffsetWndIndexIter {
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
        Self {
            pat_ind,
            signal_len,
        }
    }

    pub fn one_ind_iter(&self) -> impl Iterator<Item = usize> {
        OffsetWndIndexIter::new(
            self.pat_ind,
            self.pat_ind + 1,
            (self.pat_ind + 1) * 4,
            self.signal_len,
        )
    }

    pub fn neg_one_ind_iter(&self) -> impl Iterator<Item = usize> {
        OffsetWndIndexIter::new(
            self.pat_ind + (self.pat_ind + 1) * 2,
            self.pat_ind + 1,
            (self.pat_ind + 1) * 4,
            self.signal_len,
        )
    }
}

pub trait Signal: Index<usize, Output = u32> + IndexMut<usize, Output = u32> {
    fn len(&self) -> usize;
}

impl Signal for [u32] {
    fn len(&self) -> usize {
        self.len()
    }
}

impl Signal for Vec<u32> {
    fn len(&self) -> usize {
        self.len()
    }
}

pub struct RepeatSignal {
    signal_tem: Vec<u32>,
    total_len: usize,
}

impl RepeatSignal {
    pub fn new(signal_tem: &[u32], repeat_count: usize) -> Self {
        Self {
            signal_tem: Vec::from(signal_tem),
            total_len: repeat_count * signal_tem.len(),
        }
    }
}

impl Index<usize> for RepeatSignal {
    type Output = u32;

    fn index(&self, index: usize) -> &Self::Output {
        if index >= self.total_len {
            panic!("Index({}) exceeds signal length({})", index, self.total_len);
        }
        &self.signal_tem[index % self.signal_tem.len()]
    }
}

impl IndexMut<usize> for RepeatSignal {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index >= self.total_len {
            panic!("Index({}) exceeds signal length({})", index, self.total_len);
        }

        let tem_len = self.signal_tem.len();
        &mut self.signal_tem[index % tem_len]
    }
}

impl Signal for RepeatSignal {
    fn len(&self) -> usize {
        self.total_len
    }
}

pub struct FullFFT {
    outputs: RefCell<Vec<u32>>,
}

impl FullFFT {
    pub fn new(signal_len: usize) -> Self {
        Self {
            outputs: RefCell::new(vec![0; signal_len]),
        }
    }

    pub fn process_n(&self, signal: &mut [u32], phase_count: usize) -> Result<(), Error> {
        let signal_len = signal.len();
        let outputs_len = self.outputs.borrow().len();
        if signal_len != outputs_len {
            return Err(Error::WrongSignalLen(signal_len, outputs_len));
        }

        println!("Process begins.");
        for p_ind in 0..phase_count {
            if p_ind % 2 == 0 {
                // Even, signal -> self.outputs
                fft_phase(0, &signal, &mut self.outputs.borrow_mut());
            } else {
                // Odd, self.outputs -> signal
                fft_phase(0, &self.outputs.borrow(), signal);
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

pub struct PartialFFT {
    start_offset: usize,
    last_phase_signal: RefCell<Vec<u32>>,
    last_phase_ind: RefCell<u32>,
    phase_count: u32,
}

impl PartialFFT {
    pub fn new(start_offset: usize, init_signal: &dyn Signal, phase_count: u32) -> Self {
        assert!(start_offset <= init_signal.len());
        let init_partial_signal = (start_offset..init_signal.len())
            .map(|ind| init_signal[ind])
            .collect::<Vec<_>>();

        Self {
            start_offset,
            last_phase_signal: RefCell::new(init_partial_signal),
            last_phase_ind: RefCell::new(0),
            phase_count,
        }
    }

    pub fn nth_ele(&self, ind: usize) -> Result<u32, Error> {
        if *self.last_phase_ind.borrow() != self.phase_count - 1 {
            self.proc_to_last_phase();
        }

        self.next_nth_elem(ind)
    }

    fn next_nth_elem(&self, ind: usize) -> Result<u32, Error> {
        let total_len = self.start_offset + self.last_phase_signal.borrow().len();
        if ind < self.start_offset || ind >= total_len {
            return Err(Error::ParitalNotInclude(ind, self.start_offset, total_len));
        }

        Ok(fft_elem(
            self.start_offset,
            ind - self.start_offset,
            &*self.last_phase_signal.borrow(),
        ))
    }

    fn proc_to_last_phase(&self) {
        let mut partial_signal = self.last_phase_signal.borrow().clone();
        while *self.last_phase_ind.borrow() < self.phase_count - 1 {
            if *self.last_phase_ind.borrow() % 2 == 0 {
                // Even, self.last_phase_signal -> partial_signal
                fft_phase(
                    self.start_offset,
                    &self.last_phase_signal.borrow(),
                    &mut partial_signal,
                );
            } else {
                // Odd, partial_signal -> self.last_phase_signal
                fft_phase(
                    self.start_offset,
                    &partial_signal,
                    &mut self.last_phase_signal.borrow_mut(),
                );
            };

            println!("Partial phase {} completed.", *self.last_phase_ind.borrow());
            *self.last_phase_ind.borrow_mut() += 1;
        }

        if *self.last_phase_ind.borrow() % 2 == 1 {
            // Transfer final result to vector in self(self.last_phase_signal)
            self.last_phase_signal
                .borrow_mut()
                .copy_from_slice(&partial_signal);
        }
    }
}

// Process one phase.
fn fft_phase(offset: usize, inputs: &[u32], outputs: &mut [u32]) {
    assert!(inputs.len() == outputs.len(), "FFT inputs and outputs must be the same size. Given(inputs' size = {}, outputs' size = {}).", inputs.len(), outputs.len());
    // Optimize for latter half in signal
    if offset + 1 >= inputs.len() {
        // Compute latter half, just accumulate backward.
        for i in (0..inputs.len()).rev() {
            let sum = inputs[i] + outputs.get(i + 1).copied().unwrap_or(0);
            outputs[i] = sum % 10;
        }
        return;
    }

    // Compute by definition, but only process with elements after offset.
    outputs
        .par_iter_mut()
        .enumerate()
        .for_each(|(ind, d)| *d = fft_elem(offset, ind, inputs));
}

fn fft_elem(offset: usize, input_ind: usize, inputs: &[u32]) -> u32 {
    let signal_len = offset + inputs.len();
    let pattern = Pattern::new(input_ind + offset, signal_len);
    let one_sum: i32 = pattern
        .one_ind_iter()
        .par_bridge()
        .map(|signal_ind| inputs[signal_ind - offset] as i32)
        .sum();
    let negative_one_sum: i32 = pattern
        .neg_one_ind_iter()
        .par_bridge()
        .map(|signal_ind| inputs[signal_ind - offset] as i32)
        .sum();

    u32::try_from((one_sum - negative_one_sum).abs() % 10).unwrap()
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
