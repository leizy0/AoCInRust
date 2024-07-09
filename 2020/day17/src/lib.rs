use std::{
    collections::VecDeque,
    error,
    fmt::Display,
    fs::File,
    io::{self, BufRead, BufReader},
    iter, mem,
    ops::Range,
    path::{Path, PathBuf},
};

use clap::Parser;

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    InconsistentCubeRectRow(usize, usize), // (count of column in given row, count of column in earlier row(s)).
    InvalidCubeStateChar(char),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IOError(ioe) => write!(f, "I/O error: {}", ioe),
            Error::InconsistentCubeRectRow(this_col_n, expect_col_n) => write!(f, "Found inconsistent column count({}) in given row, expect {} columns as in earlier row(s).", this_col_n, expect_col_n),
            Error::InvalidCubeStateChar(c) => write!(f, "Invalid character({}) for cube state.", c),
        }
    }
}

impl error::Error for Error {}

#[derive(Debug, Parser)]
pub struct CLIArgs {
    pub input_path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CubeState {
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

#[derive(Debug, Clone)]
struct CubeSpace1D {
    cubes: VecDeque<CubeState>,
    dim_range: Range<isize>,
}

impl From<&[CubeState]> for CubeSpace1D {
    fn from(value: &[CubeState]) -> Self {
        Self {
            cubes: VecDeque::from_iter(value.iter().copied()),
            dim_range: 0..(value.len() as isize),
        }
    }
}

impl Display for CubeSpace1D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for s in &self.cubes {
            let c = match s {
                CubeState::InActive => '.',
                CubeState::Active => '#',
            };

            write!(f, "{}", c)?;
        }

        Ok(())
    }
}

impl CubeSpace1D {
    pub fn new(range: &Range<isize>) -> Self {
        Self {
            cubes: VecDeque::from_iter(
                iter::repeat(CubeState::InActive).take(range.clone().count()),
            ),
            dim_range: range.clone(),
        }
    }

    pub fn with_size(other: &Self) -> Self {
        Self::new(&other.dim_range)
    }

    pub fn step(&mut self, env: &CubeSpace3D, l_pos: isize, r_pos: isize) -> usize {
        fn update(
            cube: &mut CubeState,
            env: &CubeSpace3D,
            l_pos: isize,
            r_pos: isize,
            c_pos: isize,
        ) -> bool {
            let active_count = ((l_pos - 1)..=(l_pos + 1))
                .flat_map(|l| ((r_pos - 1)..=(r_pos + 1)).map(move |r| (l, r)))
                .flat_map(|(l, r)| {
                    ((c_pos - 1)..=(c_pos + 1))
                        .map(move |c| (l, r, c))
                        .filter(|(l, r, c)| *l != l_pos || *r != r_pos || *c != c_pos)
                })
                .filter_map(|(l, r, c)| env.state(l, r, c))
                .filter(|s| **s == CubeState::Active)
                .count();
            match env
                .state(l_pos, r_pos, c_pos)
                .unwrap_or(&CubeState::InActive)
            {
                CubeState::Active if active_count != 2 && active_count != 3 => {
                    *cube = CubeState::InActive;
                    true
                }
                CubeState::InActive if active_count == 3 => {
                    *cube = CubeState::Active;
                    true
                }
                org_state => {
                    *cube = *org_state;
                    false
                }
            }
        }

        self.extend(&env.c_range());
        let mut chg_count = 0;
        for (c_ind, c) in self.cubes.iter_mut().enumerate() {
            if update(c, env, l_pos, r_pos, c_ind as isize + self.dim_range.start) {
                chg_count += 1;
            }
        }

        let mut new_start_reserve_state = CubeState::InActive;
        if update(
            &mut new_start_reserve_state,
            env,
            l_pos,
            r_pos,
            self.dim_range.start - 1,
        ) {
            self.dim_range.start -= 1;
            self.cubes.push_front(new_start_reserve_state);
            chg_count += 1;
        }

        let mut new_end_reserve_state = CubeState::InActive;
        if update(
            &mut new_end_reserve_state,
            env,
            l_pos,
            r_pos,
            self.dim_range.end,
        ) {
            self.dim_range.end += 1;
            self.cubes.push_back(new_end_reserve_state);
            chg_count += 1;
        }

        chg_count
    }

    pub fn extend(&mut self, c_range: &Range<isize>) {
        for _ in c_range.start..self.dim_range.start {
            self.cubes.push_front(CubeState::InActive);
            self.dim_range.start -= 1;
        }

        for _ in self.dim_range.end..c_range.end {
            self.cubes.push_back(CubeState::InActive);
            self.dim_range.end += 1;
        }
    }

    pub fn dim_range(&self) -> &Range<isize> {
        &self.dim_range
    }

    pub fn active_n(&self) -> usize {
        self.cubes
            .iter()
            .filter(|s| **s == CubeState::Active)
            .count()
    }

    pub fn cube(&self, c_pos: isize) -> Option<&CubeState> {
        usize::try_from(c_pos - self.dim_range.start)
            .ok()
            .and_then(|ind| self.cubes.get(ind))
    }
}

#[derive(Debug, Clone)]
struct CubeSpace2D {
    start_reserve_row: CubeSpace1D,
    end_reserve_row: CubeSpace1D,
    rows: VecDeque<CubeSpace1D>,
    dim_range: Range<isize>,
}

impl From<&CubeRect2D> for CubeSpace2D {
    fn from(value: &CubeRect2D) -> Self {
        let reserve_row = CubeSpace1D::new(&(0..(value.col_n as isize)));
        let rows = VecDeque::from_iter(
            value
                .states
                .as_slice()
                .chunks(value.col_n)
                .map(|c| CubeSpace1D::from(c)),
        );
        Self {
            start_reserve_row: reserve_row.clone(),
            end_reserve_row: reserve_row,
            rows,
            dim_range: 0..(value.row_n as isize),
        }
    }
}

impl Display for CubeSpace2D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "row: {:?}, column: {:?}", self.dim_range, self.c_range())?;
        for r_pos in self.dim_range.clone() {
            writeln!(f, "{}", self.row(r_pos).unwrap())?;
        }

        Ok(())
    }
}

impl CubeSpace2D {
    pub fn new(row_range: &Range<isize>, col_range: &Range<isize>) -> Self {
        let row = CubeSpace1D::new(col_range);
        Self {
            start_reserve_row: row.clone(),
            rows: VecDeque::from_iter(
                iter::repeat_with(|| row.clone()).take(row_range.clone().count()),
            ),
            end_reserve_row: row,
            dim_range: row_range.clone(),
        }
    }

    pub fn dim_range(&self) -> &Range<isize> {
        &self.dim_range
    }

    pub fn step(&mut self, env: &CubeSpace3D, l_pos: isize) -> usize {
        fn step_reserve_row(
            row: &mut CubeSpace1D,
            env: &CubeSpace3D,
            l_pos: isize,
            r_pos: isize,
            chg_count: &mut usize,
        ) -> Option<CubeSpace1D> {
            let this_chg_count = row.step(env, l_pos, r_pos);
            *chg_count += this_chg_count;
            if this_chg_count != 0 {
                let mut new_reserve_row = CubeSpace1D::with_size(row);
                mem::swap(&mut new_reserve_row, row);
                Some(new_reserve_row)
            } else {
                None
            }
        }

        self.extend(&env.r_range(), &env.c_range());
        let mut chg_count = 0;
        let mut new_col_range = self.c_range();
        for (r_ind, row) in self.rows.iter_mut().enumerate() {
            chg_count += row.step(env, l_pos, r_ind as isize + self.dim_range.start);
            new_col_range = union_range(&new_col_range, &row.dim_range)
        }

        if let Some(new_start_row) = step_reserve_row(
            &mut self.start_reserve_row,
            env,
            l_pos,
            self.dim_range.start - 1,
            &mut chg_count,
        ) {
            new_col_range = union_range(&new_col_range, new_start_row.dim_range());
            self.rows.push_front(new_start_row);
            self.dim_range.start -= 1;
        }

        if let Some(new_end_row) = step_reserve_row(
            &mut self.end_reserve_row,
            env,
            l_pos,
            self.dim_range.end,
            &mut chg_count,
        ) {
            new_col_range = union_range(&new_col_range, new_end_row.dim_range());
            self.rows.push_back(new_end_row);
            self.dim_range.end += 1;
        }

        let new_row_range = self.dim_range.clone();
        self.extend(&new_row_range, &new_col_range);

        chg_count
    }

    pub fn extend(&mut self, r_range: &Range<isize>, c_range: &Range<isize>) {
        for r in self.rows.iter_mut() {
            r.extend(c_range);
        }

        for _ in (r_range.start..self.dim_range.start).rev() {
            self.rows.push_front(CubeSpace1D::new(c_range));
            self.dim_range.start -= 1;
        }

        for _ in self.dim_range.end..r_range.end {
            self.rows.push_back(CubeSpace1D::new(c_range));
            self.dim_range.end += 1;
        }
        self.start_reserve_row.extend(c_range);
        self.end_reserve_row.extend(c_range);
    }

    pub fn active_n(&self) -> usize {
        self.rows.iter().map(|r| r.active_n()).sum()
    }

    pub fn row(&self, r_pos: isize) -> Option<&CubeSpace1D> {
        usize::try_from(r_pos - self.dim_range.start)
            .ok()
            .and_then(|ind| self.rows.get(ind))
    }

    pub fn c_range(&self) -> Range<isize> {
        self.rows
            .get(0)
            .map(|r| r.dim_range.clone())
            .unwrap_or(0..0)
    }
}

#[derive(Debug, Clone)]
struct CubeSpace3D {
    start_reserve_layer: CubeSpace2D,
    layers: VecDeque<CubeSpace2D>,
    end_reserve_layer: CubeSpace2D,
    dim_range: Range<isize>,
}

impl Display for CubeSpace3D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for l_pos in self.dim_range.clone() {
            writeln!(f, "Layer {}:", l_pos)?;
            write!(f, "{}", self.layer(l_pos).unwrap())?;
        }

        Ok(())
    }
}

impl CubeSpace3D {
    pub fn new(init_states: &CubeRect2D) -> Self {
        let reserve_layer = CubeSpace2D::new(
            &(0..(init_states.row_n as isize)),
            &(0..(init_states.col_n as isize)),
        );
        Self {
            start_reserve_layer: reserve_layer.clone(),
            layers: VecDeque::from_iter(iter::once(CubeSpace2D::from(init_states))),
            end_reserve_layer: reserve_layer,
            dim_range: 0..1isize,
        }
    }

    pub fn step(&mut self, env: &CubeSpace3D) {
        fn step_reserve_layer(
            layer: &mut CubeSpace2D,
            env: &CubeSpace3D,
            l_pos: isize,
        ) -> Option<CubeSpace2D> {
            let chg_count = layer.step(env, l_pos);
            if chg_count != 0 {
                let mut new_reserve_layer = CubeSpace2D::new(layer.dim_range(), &layer.c_range());
                mem::swap(&mut new_reserve_layer, layer);
                Some(new_reserve_layer)
            } else {
                None
            }
        }

        self.extend(&env.dim_range, &env.r_range(), &env.c_range());
        let mut new_row_range = self.r_range();
        let mut new_col_range = self.c_range();
        for (l_ind, layer) in self.layers.iter_mut().enumerate() {
            layer.step(env, l_ind as isize + self.dim_range.start);
            new_row_range = union_range(&new_row_range, layer.dim_range());
            new_col_range = union_range(&new_col_range, &layer.c_range());
        }

        if let Some(new_start_layer) =
            step_reserve_layer(&mut self.start_reserve_layer, env, env.dim_range.start - 1)
        {
            new_row_range = union_range(&new_row_range, new_start_layer.dim_range());
            new_col_range = union_range(&new_col_range, &new_start_layer.c_range());
            self.layers.push_front(new_start_layer);
            self.dim_range.start -= 1;
        }

        if let Some(new_end_layer) =
            step_reserve_layer(&mut self.end_reserve_layer, env, env.dim_range.end)
        {
            new_row_range = union_range(&new_row_range, new_end_layer.dim_range());
            new_col_range = union_range(&new_col_range, &new_end_layer.c_range());
            self.layers.push_back(new_end_layer);
            self.dim_range.end += 1;
        }

        let new_layer_range = self.dim_range.clone();
        self.extend(&new_layer_range, &new_row_range, &new_col_range);
    }

    fn extend(&mut self, l_range: &Range<isize>, r_range: &Range<isize>, c_range: &Range<isize>) {
        for l in self.layers.iter_mut() {
            l.extend(&r_range, &c_range);
        }

        for _ in (l_range.start..self.dim_range.start).rev() {
            self.layers.push_front(CubeSpace2D::new(&r_range, &c_range));
            self.dim_range.start -= 1;
        }
        for _ in self.dim_range.end..l_range.end {
            self.layers.push_back(CubeSpace2D::new(&r_range, &c_range));
            self.dim_range.end += 1;
        }
        self.start_reserve_layer.extend(&r_range, &c_range);
        self.end_reserve_layer.extend(&r_range, &c_range);
    }

    pub fn active_n(&self) -> usize {
        self.layers.iter().map(|l| l.active_n()).sum()
    }

    pub fn layer(&self, l_pos: isize) -> Option<&CubeSpace2D> {
        usize::try_from(l_pos - self.dim_range.start)
            .ok()
            .and_then(|ind| self.layers.get(ind))
    }

    pub fn state(&self, l_pos: isize, r_pos: isize, c_pos: isize) -> Option<&CubeState> {
        self.layer(l_pos)
            .and_then(|l| l.row(r_pos))
            .and_then(|r| r.cube(c_pos))
    }

    fn r_range(&self) -> Range<isize> {
        self.layers
            .get(0)
            .map(|l| l.dim_range.clone())
            .unwrap_or(0..0)
    }

    fn c_range(&self) -> Range<isize> {
        self.layers.get(0).map(|l| l.c_range()).unwrap_or(0..0)
    }
}

pub struct CubeSpace {
    bufs: [CubeSpace3D; 2],
    cur_buf_ind: usize,
}

impl Display for CubeSpace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.cur_buf())
    }
}

impl CubeSpace {
    pub fn new(init_states: &CubeRect2D) -> Self {
        let space = CubeSpace3D::new(init_states);
        Self {
            bufs: [space.clone(), space],
            cur_buf_ind: 0,
        }
    }

    pub fn step(&mut self) {
        let (r_buf, w_buf) = self.rw_bufs();
        w_buf.step(r_buf);
        self.swap_bufs();
    }

    pub fn active_n(&self) -> usize {
        self.cur_buf().active_n()
    }

    fn rw_bufs(&mut self) -> (&CubeSpace3D, &mut CubeSpace3D) {
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

    fn cur_buf(&self) -> &CubeSpace3D {
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

fn union_range(l_range: &Range<isize>, r_range: &Range<isize>) -> Range<isize> {
    (l_range.start.min(r_range.start))..(l_range.end.max(r_range.end))
}
