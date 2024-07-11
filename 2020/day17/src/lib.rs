use std::{
    error,
    fmt::Display,
    fs::File,
    io::{self, BufRead, BufReader},
    path::{Path, PathBuf},
};

use clap::Parser;
use space::{CubeSpace, CubeSpace1D, CubeSpaceND, Position};

mod space;

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    InconsistentCubeRectRow(usize, usize), // (count of column in given row, count of column in earlier row(s)).
    InvalidCubeStateChar(char),
    IncompleteSelectorToPosition(String, usize), //(Text of selector, expected dimension).
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IOError(ioe) => write!(f, "I/O error: {}", ioe),
            Error::InconsistentCubeRectRow(this_col_n, expect_col_n) => write!(f, "Found inconsistent column count({}) in given row, expect {} columns as in earlier row(s).", this_col_n, expect_col_n),
            Error::InvalidCubeStateChar(c) => write!(f, "Invalid character({}) for cube state.", c),
            Error::IncompleteSelectorToPosition(sel_s, dims) => write!(f, "Incomplete selector({}) can't convert to position, expect full {}D selector", sel_s, dims),
        }
    }
}

impl error::Error for Error {}

#[derive(Debug, Parser)]
pub struct CLIArgs {
    pub input_path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CubeState {
    InActive,
    Active,
}

pub struct CubeRect2D {
    states: Vec<CubeState>,
    row_n: usize,
    col_n: usize,
}

struct CubeRect2DBuilder {
    states: Vec<CubeState>,
    row_n: usize,
    col_n: Option<usize>,
}

impl CubeRect2DBuilder {
    pub fn new() -> Self {
        Self {
            states: Vec::new(),
            row_n: 0,
            col_n: None,
        }
    }

    pub fn push_row(&mut self, row_text: &str) -> Result<(), Error> {
        let this_col_n = row_text.chars().count();
        if *self.col_n.get_or_insert(this_col_n) != this_col_n {
            return Err(Error::InconsistentCubeRectRow(
                this_col_n,
                self.col_n.unwrap(),
            ));
        }

        for c in row_text.chars() {
            let state = match c {
                '.' => CubeState::InActive,
                '#' => CubeState::Active,
                other => return Err(Error::InvalidCubeStateChar(other)),
            };
            self.states.push(state);
        }

        self.row_n += 1;
        Ok(())
    }

    pub fn build(self) -> CubeRect2D {
        CubeRect2D {
            states: self.states,
            row_n: self.row_n,
            col_n: self.col_n.unwrap_or(0),
        }
    }
}

const MAX_DIMENSION: usize = 4;
pub type CubeSpace2D = CubeSpaceND<2, CubeSpace1D>;
pub type CubeSpace3D = CubeSpaceND<3, CubeSpace2D>;
pub type CubeSpace4D = CubeSpaceND<4, CubeSpace3D>;

pub struct CubeSpaceSimulator<CS: CubeSpace> {
    bufs: [CS; 2],
    cur_buf_ind: usize,
}

impl<CS: CubeSpace> Display for CubeSpaceSimulator<CS> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let space = self.cur_buf();
        let dims = space.dims();
        if dims > 2 {
            for selector in space.ranges().select_dim(2) {
                writeln!(f, "Hyper rectangle {}", selector)?;
                writeln!(f, "{}", space.inner(&selector).unwrap())?;
            }
        } else {
            write!(f, "{}", space)?;
        }

        Ok(())
    }
}

impl<CS: CubeSpace> CubeSpaceSimulator<CS> {
    pub fn new(init_states: &CubeRect2D) -> Self {
        let space = CS::from(init_states);
        Self {
            bufs: [space.clone(), space],
            cur_buf_ind: 0,
        }
    }

    pub fn step(&mut self) {
        let (r_buf, w_buf) = self.rw_bufs();
        let pos = Position::new();
        w_buf.step(r_buf, &pos);
        self.swap_bufs();
    }

    pub fn active_n(&self) -> usize {
        self.cur_buf().active_n()
    }

    fn rw_bufs(&mut self) -> (&CS, &mut CS) {
        let (l_bufs, r_bufs) = self.bufs.split_at_mut(1);
        if self.cur_buf_ind == 0 {
            (&l_bufs[0], &mut r_bufs[0])
        } else {
            (&r_bufs[0], &mut l_bufs[0])
        }
    }

    fn swap_bufs(&mut self) {
        self.cur_buf_ind = 1 - self.cur_buf_ind;
    }

    fn cur_buf(&self) -> &CS {
        &self.bufs[self.cur_buf_ind]
    }
}

pub fn read_state<P: AsRef<Path>>(path: P) -> Result<CubeRect2D, Error> {
    let file = File::open(path).map_err(Error::IOError)?;
    let reader = BufReader::new(file);
    let mut builder = CubeRect2DBuilder::new();
    for line in reader.lines() {
        let s = line.map_err(Error::IOError)?;
        builder.push_row(&s)?;
    }

    Ok(builder.build())
}
