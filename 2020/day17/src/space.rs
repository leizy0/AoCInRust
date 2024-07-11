use std::{array, collections::VecDeque, fmt::Display, iter, mem, ops::Range};

use crate::{CubeRect2D, CubeState, Error, MAX_DIMENSION};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PositionND<const N: usize> {
    pos: [isize; N], // Store with dimension decrementally.
    dims: usize,
}
pub type Position = PositionND<MAX_DIMENSION>;

impl<const N: usize> Display for PositionND<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.dims == 0 {
            return write!(f, "()");
        }

        write!(f, "({}", self.coord(self.dims))?;

        for d in (1..self.dims).rev() {
            write!(f, ", {}", self.coord(d))?;
        }
        write!(f, ")")
    }
}

impl<const N: usize> TryFrom<&SelectorND<N>> for PositionND<N> {
    type Error = Error;

    fn try_from(value: &SelectorND<N>) -> Result<Self, Self::Error> {
        if !value.dim_range.contains(&1) {
            return Err(Error::IncompleteSelectorToPosition(value.to_string(), N));
        }

        let mut pos = Self::new();
        for d in value.dim_range().clone().rev() {
            pos.extend();
            *pos.back_mut() = *value.coord(d);
        }

        Ok(pos)
    }
}

impl<const N: usize> PositionND<N> {
    pub fn new() -> Self {
        Self {
            pos: [0; N],
            dims: 0,
        }
    }

    pub fn dims(&self) -> usize {
        self.dims
    }

    pub fn coord(&self, dim: usize) -> &isize {
        &self.pos[self.dim_to_ind(dim)]
    }

    pub fn neighbors(&self) -> NeighborPositionNDIter<N> {
        let mut ranges = RangesND::<N>::new();
        for d in 1..=self.dims {
            let coord = *self.coord(d);
            ranges.extend_up(&((coord - 1)..(coord + 2)));
        }

        NeighborPositionNDIter::<N>::new(&ranges, &self)
    }

    pub fn extend(&mut self) {
        assert!(
            self.dims < N,
            "Position is already full on dimension({}).",
            N
        );
        self.dims += 1;
    }

    pub fn back_mut(&mut self) -> &mut isize {
        &mut self.pos[self.dims - 1]
    }

    fn dim_to_ind(&self, dim: usize) -> usize {
        assert!(
            dim > 0 && dim <= self.dims,
            "Invalid dimension({}) for position in {} dimension.",
            dim,
            self.dims
        );
        self.dims - dim
    }
}

pub struct NeighborPositionNDIter<const N: usize> {
    sel_iter: SelectorNDIter<N>,
    self_pos: PositionND<N>,
}

impl<const N: usize> Iterator for NeighborPositionNDIter<N> {
    type Item = PositionND<N>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(pos) = self
            .sel_iter
            .next()
            .map(|s| Self::Item::try_from(&s).unwrap())
        {
            if pos != self.self_pos {
                return Some(pos);
            }
        }

        None
    }
}

impl<const N: usize> NeighborPositionNDIter<N> {
    pub fn new(ranges: &RangesND<N>, self_pos: &PositionND<N>) -> Self {
        Self {
            sel_iter: ranges.select_dim(0),
            self_pos: self_pos.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SelectorND<const N: usize> {
    coords: [isize; N], // Store with dimension incrementally.
    dim_range: Range<usize>,
}
type Selector = SelectorND<MAX_DIMENSION>;

impl<const N: usize> Display for SelectorND<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut dims = self.dim_range.clone().rev();
        if let Some(first_dim) = dims.next() {
            write!(f, "({}", self.coord(first_dim))?;
        }

        for d in dims {
            write!(f, ", {}", self.coord(d))?;
        }
        write!(f, ")")
    }
}

impl<const N: usize> SelectorND<N> {
    pub fn new(start_dim: usize, coords: impl Iterator<Item = isize>) -> Self {
        let mut end_dim = start_dim;
        let mut selector = Self {
            coords: [0; N],
            dim_range: 0..0,
        };
        for (ind, c) in coords.enumerate() {
            selector.coords[ind] = c;
            end_dim += 1;
        }
        selector.dim_range = start_dim..end_dim;

        selector
    }

    pub fn coord(&self, dim: usize) -> &isize {
        &self.coords[self.dim_to_ind(dim)]
    }

    pub fn dim_range(&self) -> &Range<usize> {
        &self.dim_range
    }

    pub fn coord_mut(&mut self, dim: usize) -> &mut isize {
        &mut self.coords[self.dim_to_ind(dim)]
    }

    fn dim_to_ind(&self, dim: usize) -> usize {
        assert!(
            self.dim_range.contains(&dim),
            "No dimension {} in selector(has range {:?}).",
            dim,
            self.dim_range
        );
        dim - self.dim_range.start
    }
}

pub struct SelectorNDIter<const N: usize> {
    cur_selector: SelectorND<N>,
    ranges: RangesND<N>,
    cur_dim: usize,
}

impl<const N: usize> Iterator for SelectorNDIter<N> {
    type Item = SelectorND<N>;

    fn next(&mut self) -> Option<Self::Item> {
        let dim_range = self.cur_selector.dim_range();
        let end_dim = dim_range.end;
        if self.cur_dim >= end_dim {
            return None;
        }

        let mut chg_dim = dim_range.start;
        let selector = self.cur_selector.clone();
        while chg_dim < end_dim {
            let cur_coord = self.cur_selector.coord_mut(chg_dim);
            *cur_coord += 1;
            let cur_range = self.ranges.range(chg_dim);
            if cur_range.contains(&cur_coord) {
                break;
            }

            *cur_coord = cur_range.start;
            chg_dim += 1;
            if chg_dim > self.cur_dim {
                self.cur_dim = chg_dim;
            }
        }

        Some(selector)
    }
}

impl<const N: usize> SelectorNDIter<N> {
    pub fn new(start_dim: usize, ranges: &RangesND<N>) -> Self {
        Self {
            cur_selector: SelectorND::<N>::new(
                start_dim,
                (start_dim..=ranges.dims()).map(|d| ranges.range(d).start),
            ),
            ranges: ranges.clone(),
            cur_dim: start_dim,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RangesND<const N: usize> {
    ranges: [Range<isize>; N], // Store with dimension incrementally.
    dims: usize,
}
type DimRanges = RangesND<MAX_DIMENSION>;

impl<const N: usize> RangesND<N> {
    pub fn new() -> Self {
        Self {
            ranges: array::from_fn(|_| 0..0),
            dims: 0,
        }
    }

    pub fn dims(&self) -> usize {
        self.dims
    }

    pub fn range(&self, dim: usize) -> &Range<isize> {
        &self.ranges[self.dim_to_ind(dim)]
    }

    pub fn extend_up(&mut self, range: &Range<isize>) {
        assert!(
            self.dims < N,
            "Can't extend any more, dimension is already full."
        );
        self.dims += 1;
        self.ranges[self.dim_to_ind(self.dims)] = range.clone();
    }

    pub fn extend_to_inc(&mut self, other: &Self) {
        for d in (self.dims + 1)..=(other.dims) {
            let ind = d - 1;
            self.ranges[ind] = other.ranges[ind].clone();
        }

        for d in 1..=self.dims {
            let ind = d - 1;
            self.ranges[ind] = range_to_inc(&self.ranges[ind], &other.ranges[ind]);
        }

        self.dims = self.dims.max(other.dims);
    }

    pub fn select_dim(&self, dim: usize) -> SelectorNDIter<N> {
        assert!(
            dim < self.dims,
            "Only lower dimension(< {}) can be selected.",
            self.dims
        );
        let start_dim = dim + 1;
        SelectorNDIter::<N>::new(start_dim, self)
    }

    fn dim_to_ind(&self, dim: usize) -> usize {
        assert!(
            dim > 0 && dim <= self.dims,
            "Given dimension({}) has exceeded dimension range([1, {}]).",
            dim,
            N
        );
        dim - 1
    }
}

pub trait CubeSpaceDynamics: Display {
    fn step(&mut self, env: &dyn CubeSpaceDynamics, pos: &Position) -> usize;
    fn extend(&mut self, ranges: &DimRanges);
    fn active_n(&self) -> usize;
    fn ranges(&self) -> DimRanges;
    fn state(&self, pos: &Position) -> Option<&CubeState>;
    fn dims(&self) -> usize;
    fn inner(&self, selector: &Selector) -> Option<&dyn CubeSpaceDynamics>;
}
pub trait CubeSpace:
    CubeSpaceDynamics
    + Clone
    + for<'a> From<&'a DimRanges>
    + for<'a> From<&'a CubeRect2D>
    + for<'a> From<&'a [CubeState]>
{
}

#[derive(Debug, Clone)]
pub struct CubeSpace1D {
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

impl From<&DimRanges> for CubeSpace1D {
    fn from(value: &DimRanges) -> Self {
        let range = value.range(1);
        Self {
            cubes: VecDeque::from_iter(
                iter::repeat(CubeState::InActive).take(range.clone().count()),
            ),
            dim_range: range.clone(),
        }
    }
}

impl From<&CubeRect2D> for CubeSpace1D {
    fn from(_: &CubeRect2D) -> Self {
        panic!("1D space can't be constructed from 2D rectangle area of cubes.");
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

impl CubeSpaceDynamics for CubeSpace1D {
    fn step(&mut self, env: &dyn CubeSpaceDynamics, pos: &Position) -> usize {
        fn update(cube: &mut CubeState, env: &dyn CubeSpaceDynamics, pos: &Position) -> bool {
            let active_count = pos
                .neighbors()
                .filter_map(|n_pos| env.state(&n_pos))
                .filter(|s| **s == CubeState::Active)
                .count();
            match env.state(pos).unwrap_or(&CubeState::InActive) {
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

        self.extend(&env.ranges());
        let mut chg_count = 0;
        let mut cur_pos = pos.clone();
        cur_pos.extend();
        for (c_ind, c) in self.cubes.iter_mut().enumerate() {
            *cur_pos.back_mut() = c_ind as isize + self.dim_range.start;
            if update(c, env, &cur_pos) {
                chg_count += 1;
            }
        }

        let mut new_start_reserve_state = CubeState::InActive;
        *cur_pos.back_mut() = self.dim_range.start - 1;
        if update(&mut new_start_reserve_state, env, &cur_pos) {
            self.dim_range.start -= 1;
            self.cubes.push_front(new_start_reserve_state);
            chg_count += 1;
        }

        let mut new_end_reserve_state = CubeState::InActive;
        *cur_pos.back_mut() = self.dim_range.end;
        if update(&mut new_end_reserve_state, env, &cur_pos) {
            self.dim_range.end += 1;
            self.cubes.push_back(new_end_reserve_state);
            chg_count += 1;
        }

        chg_count
    }

    fn extend(&mut self, ranges: &DimRanges) {
        let ex_range = ranges.range(1);
        for _ in ex_range.start..self.dim_range.start {
            self.cubes.push_front(CubeState::InActive);
            self.dim_range.start -= 1;
        }

        for _ in self.dim_range.end..ex_range.end {
            self.cubes.push_back(CubeState::InActive);
            self.dim_range.end += 1;
        }
    }

    fn active_n(&self) -> usize {
        self.cubes
            .iter()
            .filter(|c| **c == CubeState::Active)
            .count()
    }

    fn ranges(&self) -> DimRanges {
        let mut ranges = DimRanges::new();
        ranges.extend_up(&self.dim_range);

        ranges
    }

    fn state(&self, pos: &Position) -> Option<&CubeState> {
        usize::try_from(pos.coord(1) - self.dim_range.start)
            .ok()
            .and_then(|ind| self.cubes.get(ind))
    }

    fn dims(&self) -> usize {
        1
    }

    fn inner(&self, _: &Selector) -> Option<&dyn CubeSpaceDynamics> {
        None
    }
}

impl CubeSpace for CubeSpace1D {}

#[derive(Debug, Clone)]
pub struct CubeSpaceND<const N: usize, InnerSpace: CubeSpace> {
    start_reserve_inner: InnerSpace,
    inner_spaces: VecDeque<InnerSpace>,
    end_reserve_inner: InnerSpace,
    dim_range: Range<isize>,
}

impl<const N: usize, InnerSpace: CubeSpace> From<&DimRanges> for CubeSpaceND<N, InnerSpace> {
    fn from(value: &DimRanges) -> Self {
        assert!(value.dims() >= N);
        let dim_range = value.range(N).clone();
        let inner_dims = InnerSpace::from(value);
        let inner_spaces =
            VecDeque::from_iter(iter::repeat(inner_dims.clone()).take(dim_range.clone().count()));
        Self {
            start_reserve_inner: inner_dims.clone(),
            inner_spaces,
            end_reserve_inner: inner_dims,
            dim_range,
        }
    }
}

impl<const N: usize, InnerSpace: CubeSpace> From<&CubeRect2D> for CubeSpaceND<N, InnerSpace> {
    fn from(value: &CubeRect2D) -> Self {
        if N > 2 {
            let inner = InnerSpace::from(value);
            Self {
                start_reserve_inner: inner.clone(),
                inner_spaces: VecDeque::from_iter(iter::once(inner.clone())),
                end_reserve_inner: inner,
                dim_range: 0..1,
            }
        } else if N == 2 {
            let mut inner_range = DimRanges::new();
            inner_range.extend_up(&(0..(value.col_n as isize)));
            let reserve_inner = InnerSpace::from(&inner_range);
            let inner_spaces = VecDeque::from_iter(
                value
                    .states
                    .as_slice()
                    .chunks(value.col_n)
                    .map(|c| InnerSpace::from(c)),
            );
            Self {
                start_reserve_inner: reserve_inner.clone(),
                end_reserve_inner: reserve_inner,
                inner_spaces,
                dim_range: 0..(value.row_n as isize),
            }
        } else {
            panic!("Space with lower dimension({}) can't be constructed from 2d rectangle area of cubes.", N);
        }
    }
}

impl<const N: usize, InnerSpace: CubeSpace> From<&[CubeState]> for CubeSpaceND<N, InnerSpace> {
    fn from(_: &[CubeState]) -> Self {
        panic!("This method should only be implemented by special 1d cube space struct.");
    }
}

impl<const N: usize, InnerSpace: CubeSpace> Display for CubeSpaceND<N, InnerSpace> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if N == 2 {
            // Only print spaces lower than 2D dimension, higher dimensions are handled in simulator.
            writeln!(
                f,
                "row({:?}), column({:?}):",
                self.dim_range,
                self.ranges().range(1)
            )?;
            for inner in &self.inner_spaces {
                writeln!(f, "{}", inner)?;
            }
        }

        Ok(())
    }
}

impl<const N: usize, InnerSpace: CubeSpace> CubeSpaceDynamics for CubeSpaceND<N, InnerSpace> {
    fn step(&mut self, env: &dyn CubeSpaceDynamics, pos: &Position) -> usize {
        fn step_reserve_space<InnerSpace: CubeSpace>(
            inner_space: &mut InnerSpace,
            env: &dyn CubeSpaceDynamics,
            l_pos: &Position,
            chg_count: &mut usize,
        ) -> Option<InnerSpace> {
            let this_chg_count = inner_space.step(env, l_pos);
            *chg_count += this_chg_count;
            if this_chg_count != 0 {
                let mut new_reserve_layer = InnerSpace::from(&inner_space.ranges());
                mem::swap(&mut new_reserve_layer, inner_space);
                Some(new_reserve_layer)
            } else {
                None
            }
        }

        // assert!(pos.dims() + N >= MAX_DIMENSION, "Coordinates of upper dimension should be specified.");

        self.extend(&env.ranges());
        let mut chg_count = 0;
        let mut new_ranges = self.ranges();
        let mut cur_pos = pos.clone();
        cur_pos.extend();
        for (l_ind, inner_space) in self.inner_spaces.iter_mut().enumerate() {
            *cur_pos.back_mut() = l_ind as isize + self.dim_range.start;
            chg_count += inner_space.step(env, &cur_pos);
            new_ranges.extend_to_inc(&inner_space.ranges());
        }

        *cur_pos.back_mut() = self.dim_range.start - 1;
        if let Some(new_start_inner) =
            step_reserve_space(&mut self.start_reserve_inner, env, &cur_pos, &mut chg_count)
        {
            new_ranges.extend_to_inc(&new_start_inner.ranges());
            self.inner_spaces.push_front(new_start_inner);
            self.dim_range.start -= 1;
        }

        *cur_pos.back_mut() = self.dim_range.end;
        if let Some(new_end_inner) =
            step_reserve_space(&mut self.end_reserve_inner, env, &cur_pos, &mut chg_count)
        {
            new_ranges.extend_to_inc(&new_end_inner.ranges());
            self.inner_spaces.push_back(new_end_inner);
            self.dim_range.end += 1;
        }

        new_ranges.extend_to_inc(&self.ranges());
        self.extend(&new_ranges);

        chg_count
    }

    fn extend(&mut self, ranges: &DimRanges) {
        for inner in self.inner_spaces.iter_mut() {
            inner.extend(ranges);
        }

        let self_ex_range = ranges.range(N);
        for _ in (self_ex_range.start..self.dim_range.start).rev() {
            self.inner_spaces.push_front(InnerSpace::from(ranges));
            self.dim_range.start -= 1;
        }
        for _ in self.dim_range.end..self_ex_range.end {
            self.inner_spaces.push_back(InnerSpace::from(ranges));
            self.dim_range.end += 1;
        }
        self.start_reserve_inner.extend(ranges);
        self.end_reserve_inner.extend(ranges);
    }

    fn active_n(&self) -> usize {
        self.inner_spaces.iter().map(|is| is.active_n()).sum()
    }

    fn ranges(&self) -> DimRanges {
        let mut inner_ranges = self
            .inner_spaces
            .get(0)
            .map(|is| is.ranges())
            .unwrap_or(DimRanges::new());
        inner_ranges.extend_up(&self.dim_range);

        inner_ranges
    }

    fn state(&self, pos: &Position) -> Option<&CubeState> {
        assert!(
            pos.dims() >= N,
            "Given position({}D) should have specified coordinate for current dimension({}).",
            pos.dims(),
            N
        );
        usize::try_from(pos.coord(N) - self.dim_range.start)
            .ok()
            .and_then(|ind| self.inner_spaces.get(ind))
            .and_then(|is| is.state(pos))
    }

    fn dims(&self) -> usize {
        N
    }

    fn inner(&self, selector: &Selector) -> Option<&dyn CubeSpaceDynamics> {
        let this_inner = usize::try_from(selector.coord(N) - self.dim_range.start)
            .ok()
            .and_then(|ind| {
                self.inner_spaces
                    .get(ind)
                    .map(|inner| inner as &dyn CubeSpaceDynamics)
            });
        if selector.dim_range().start == N {
            this_inner
        } else {
            this_inner.and_then(|t| t.inner(selector))
        }
    }
}
impl<const N: usize, InnerSpace: CubeSpace> CubeSpace for CubeSpaceND<N, InnerSpace> {}

fn range_to_inc(l_range: &Range<isize>, r_range: &Range<isize>) -> Range<isize> {
    (l_range.start.min(r_range.start))..(l_range.end.max(r_range.end))
}
