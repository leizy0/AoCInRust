use std::{collections::{BTreeMap, BTreeSet, HashMap}, fs::File, path::Path, io::{BufReader, BufRead}, ops::{Range, Deref}, cell::{RefCell, Ref}};
use std::ops::Bound::{Included, Excluded, Unbounded};

use once_cell::sync::Lazy;
use regex::Regex;

use super::{Error, Position};

#[derive(Clone)]
pub struct UnderGroundMap {
    map: BTreeMap<usize, BlockRangeRow>,
    search_map: RefCell<HashMap<usize, Vec<usize>>>,
    horz_beg: usize,
    horz_end: usize,
}

impl UnderGroundMap {
    pub fn new() -> Self {
        UnderGroundMap { map: BTreeMap::new(), search_map: RefCell::new(HashMap::new()), horz_beg: 0, horz_end: 0 }
    }

    pub fn add_rect(&mut self, rect: &BlockRect) {
        for (r_ind, r_range) in rect.rows() {
            self.add_row_range(r_ind, &r_range);
        }
    }

    pub fn add_row_range(&mut self, r_ind: usize, range: &BlockRange) {
        self.horz_beg = self.horz_beg.min(range.beg);
        self.horz_end = self.horz_end.max(range.end);
        self.search_map.get_mut().remove(&r_ind);
        self.map.entry(r_ind).or_insert(BlockRangeRow::new()).add_range(&range);
    }

    pub fn horz_range(&self) -> Range<usize> {
        self.horz_beg..self.horz_end
    }

    pub fn vert_range(&self) -> Range<usize> {
        let min = self.map.iter().next().map(|op| *op.0).unwrap_or(0);
        let max = self.map.iter().next_back().map(|op| *op.0).unwrap_or(0);
        Range { start: min, end: max.checked_add(1).unwrap_or(usize::MAX) }
    }

    pub fn is_blocked(&self, r: usize, c: usize) -> bool {
        let (end_type, _, _) = self.search_same_col_range(r, c);
        end_type == FlowEndType::Block
    }

    pub fn flow_hort(&self, r: usize, c: usize) -> Result<(FlowEnd, FlowEnd), Error> {
        let (cur_row_type, cur_left_pos, cur_right_pos) = self.search_same_col_range(r, c);
        if cur_row_type == FlowEndType::Block {
            return Err(Error::WaterFlowThroughClay(Position::new(r, c)))
        }

        let (next_row_type, next_left_pos, next_right_pos) = self.search_same_col_range(r + 1, c);
        if next_row_type == FlowEndType::Leak {
            return Err(Error::WaterBlockedByGhost(Position::new(r + 1, c)))
        }

        let left_end = if cur_left_pos >= next_left_pos {
            FlowEnd::block(cur_left_pos.checked_sub(1).unwrap_or(0))
        } else {
            FlowEnd::leak(next_left_pos.checked_sub(1).unwrap_or(0))
        };
        let right_end = if cur_right_pos <= next_right_pos {
            FlowEnd::block(cur_right_pos)
        } else {
            FlowEnd::leak(next_right_pos)
        };

        Ok((left_end, right_end))
    }

    fn search_same_col_range(&self, r: usize, c: usize) -> (FlowEndType, usize, usize) {
        if !self.map.contains_key(&r) {
            return (FlowEndType::Leak, 0, usize::MAX);
        }

        let search_row = self.search_row(r);
        match search_row.binary_search(&c) {
            Ok(ind) => {
                if ind % 2 == 0 {
                    // Block, at begin of one range
                    (FlowEndType::Block, search_row[ind], *search_row.get(ind + 1).unwrap_or(&usize::MAX))
                } else {
                    // Leak, at end of one range
                    (FlowEndType::Leak, search_row[ind], *search_row.get(ind + 1).unwrap_or(&usize::MAX))
                }
            },
            Err(ind) => {
                if ind % 2 == 0 {
                    // Leak, between of two neighbor range
                    (FlowEndType::Leak, ind.checked_sub(1).map(|i| search_row[i]).unwrap_or(0usize), *search_row.get(ind).unwrap_or(&usize::MAX))
                } else {
                    // Block, in one range
                    (FlowEndType::Block, ind.checked_sub(1).map(|i| search_row[i]).unwrap_or(0usize), *search_row.get(ind).unwrap_or(&usize::MAX))
                }
            }
        }
    }

    fn search_row(&self, r: usize) -> impl Deref<Target = Vec<usize>> + '_ {
        if !self.search_map.borrow().contains_key(&r) {
            self.search_map.borrow_mut().insert(r, self.map[&r].search_arr());
        }

        Ref::map(self.search_map.borrow(), |map| &map[&r])
    }
}

#[derive(PartialEq, Eq)]
pub enum FlowEndType {
    Block,
    Leak,
}

pub struct FlowEnd {
    pub end_type: FlowEndType,
    pub ind: usize,
}

impl FlowEnd {
    pub fn block(ind: usize) -> FlowEnd {
        FlowEnd { end_type: FlowEndType::Block, ind }
    }

    pub fn leak(ind: usize) -> FlowEnd {
        FlowEnd { end_type: FlowEndType::Leak, ind }
    }
}

#[derive(Clone)]
struct BlockRangeRow {
    row: BTreeSet<BlockRange>,
}

impl BlockRangeRow {
    pub fn new() -> BlockRangeRow {
        BlockRangeRow { row: BTreeSet::new() }
    }

    pub fn add_range(&mut self, range: &BlockRange) {
        let search_range = (Included(BlockRange::beg_len(range.beg, 0)), 
            Included(BlockRange::beg_end(range.end, usize::MAX)));
        let mut comb_range = *range;
        let insect_ranges: Vec<_> = self.row.range(search_range).copied().collect();
        for insect_range in insect_ranges {
            self.row.remove(&insect_range);
            comb_range.combine(&insect_range);
        }

        if let Some(prev_range) = self.row.range((Unbounded, Excluded(BlockRange::beg_len(range.beg, 0))))
            .next_back().copied() {
            if comb_range.combine(&prev_range) {
                self.row.remove(&prev_range);
            }
        }

        self.row.insert(comb_range);
    }

    pub fn search_arr(&self) -> Vec<usize> {
        self.row.iter()
            .flat_map(|r| [r.beg, r.end])
            .collect()
    }
}

pub struct BlockRect {
    top_left: Position,
    bottom_right: Position,
}

impl TryFrom<&str> for BlockRect {
    type Error = Error;

    fn try_from(text: &str) -> Result<Self, Self::Error> {
        static XY_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"x=([0-9.]+), y=([0-9.]+)").unwrap());
        static YX_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"y=([0-9.]+), x=([0-9.]+)").unwrap());

        let (x_range_text, y_range_text) = if let Some(xy_caps) = XY_PATTERN.captures(text) {
            (xy_caps[1].to_string(), xy_caps[2].to_string())
        } else if let Some(yx_caps) = YX_PATTERN.captures(text) {
            (yx_caps[2].to_string(), yx_caps[1].to_string())
        } else {
            return Err(Error::BlockRectParseFormat(text.to_string()));
        };

        Ok(Self::from_range(&BlockRange::try_from(y_range_text.as_str())?, &BlockRange::try_from(x_range_text.as_str())?))
    }
}

impl BlockRect {
    pub fn from_range(row_range: &BlockRange, col_range: &BlockRange) -> BlockRect {
        BlockRect { top_left: Position{r: row_range.beg, c: col_range.beg}, bottom_right: Position{r: row_range.end - 1, c: col_range.end - 1} }
    }

    pub fn rows(&self) -> BlockRectRowIter {
        BlockRectRowIter::new(self)
    }
}

pub struct BlockRectRowIter<'a> {
    rect: &'a BlockRect,
    iter_ind: usize,
}

impl <'a> Iterator for BlockRectRowIter<'a> {
    type Item = (usize, BlockRange);

    fn next(&mut self) -> Option<Self::Item> {
        let row_ind = self.iter_ind + self.rect.top_left.r;
        if row_ind <= self.rect.bottom_right.r {
            self.iter_ind += 1;
            Some((row_ind, BlockRange{beg: self.rect.top_left.c, end: self.rect.bottom_right.c + 1}))
        } else {
            None
        }
    }
}

impl BlockRectRowIter<'_> {
    pub fn new(rect: &BlockRect) -> BlockRectRowIter {
        BlockRectRowIter {rect, iter_ind: 0}
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct BlockRange {
    beg: usize,
    end: usize,
}

impl TryFrom<&str> for BlockRange {
    type Error = Error;

    fn try_from(text: &str) -> Result<Self, Self::Error> {
        static NUMBER_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\d+$").unwrap());
        static RANGE_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\d+)..(\d+)$").unwrap());
        
        if NUMBER_PATTERN.is_match(text) {
            let beg = text.parse::<usize>().unwrap();
            Ok(Self::beg_len(beg, 1))
        } else if let Some(caps) = RANGE_PATTERN.captures(text) {
            let beg = caps[1].parse::<usize>().unwrap();
            let end = caps[2].parse::<usize>().unwrap();
            Ok(Self::beg_end(beg, end + 1))
        } else {
            Err(Error::BlockRangeParseError(text.to_string()))
        }
    }
}

impl Ord for BlockRange {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.beg.cmp(&other.beg) {
            std::cmp::Ordering::Equal => self.end.cmp(&other.end),
            ordering => ordering,
        }
    }
}

impl BlockRange {
    pub fn beg_len(beg: usize, len: usize) -> Self {
        BlockRange { beg, end: beg + len }
    }

    pub fn beg_end(beg: usize, end: usize) -> Self {
        assert!(beg <= end);
        BlockRange { beg, end }
    }

    pub fn combine(&mut self, other: &Self) -> bool {
        if self.beg > other.end || self.end < other.beg {
            return false;
        }

        self.beg = self.beg.min(other.beg);
        self.end = self.end.max(other.end);
        return true;
    }

    pub fn len(&self) -> usize {
        self.end - self.beg
    }
}

pub fn load_und_map<P>(input_path: P) -> Result<UnderGroundMap, Error> where P: AsRef<Path> {
    let input_file = File::open(input_path).map_err(Error::IOError)?;
    let reader = BufReader::new(input_file);
    let lines = reader.lines()
        .map(|le| le.map_err(Error::IOError))
        .collect::<Result<Vec<_>, _>>()?;
    
    let mut map = UnderGroundMap::new();
    for line in lines {
        let rect = BlockRect::try_from(line.as_str())?;
        map.add_rect(&rect);
    }

    Ok(map)
}

pub struct WaterMap {
    map: BTreeMap<usize, WaterRow>,
    vert_beg: usize,
    vert_end: usize,
    horz_beg: usize,
    horz_end: usize,
}

impl WaterMap {
    pub fn new(vert_range: &Range<usize>) -> WaterMap {
        WaterMap { map: BTreeMap::new(), vert_beg: vert_range.start, vert_end: vert_range.end, horz_beg: 0, horz_end: 0 }
    }

    pub fn add_row_range(&mut self, r: usize, range: &WaterRowRange) {
        if !self.is_in_vert_range(r) {
            return;
        }

        self.horz_beg = self.horz_beg.min(range.range.beg);
        self.horz_end = self.horz_end.max(range.range.end);
        let row = self.map.entry(r).or_insert(WaterRow::new());
        row.add_range(range);
    }

    pub fn count_water(&self) -> (usize, usize) {
        let mut reach_sum = 0;
        let mut rest_sum = 0;
        for (r_ind, row) in &self.map {
            if !self.is_in_vert_range(*r_ind) {
                continue;
            }

            let (row_reach, row_rest) = row.count_water();
            reach_sum += row_reach;
            rest_sum += row_rest;
        }

        (reach_sum, rest_sum)
    }

    pub fn horz_range(&self) -> Range<usize> {
        self.horz_beg..self.horz_end
    }

    fn is_in_vert_range(&self, r: usize) -> bool {
        r >= self.vert_beg && r < self.vert_end
    }
}

struct WaterRow {
    reach_row: BlockRangeRow,
    rest_row: BlockRangeRow,
}

impl WaterRow {
    pub fn new() -> WaterRow {
        WaterRow { reach_row: BlockRangeRow::new(), rest_row: BlockRangeRow::new() }
    }

    pub fn add_range(&mut self, range: &WaterRowRange) {
        if range.water_type == WaterType::Rest {
            self.rest_row.add_range(&range.range)
        }

        self.reach_row.add_range(&range.range);
    }

    pub fn count_water(&self) -> (usize, usize) {
        let reach_sum: usize = self.reach_row.row.iter().map(|r| r.len()).sum();
        let rest_sum = self.rest_row.row.iter().map(|r| r.len()).sum();

        (reach_sum - rest_sum, rest_sum)
    }
}

#[derive(PartialEq, Eq)]
pub enum WaterType {
    Reach,
    Rest,
}

pub struct WaterRowRange {
    water_type: WaterType,
    range: BlockRange,
}

impl WaterRowRange {
    pub fn reach(range: &BlockRange) -> WaterRowRange {
        WaterRowRange { water_type: WaterType::Reach, range: *range }
    }

    pub fn rest(range: &BlockRange) -> WaterRowRange {
        WaterRowRange { water_type: WaterType::Rest, range: *range }
    }
}

pub fn log_map_text(under_map: &UnderGroundMap, water_map: &WaterMap) -> Result<(), Error> {
    let under_horz_range = under_map.horz_range();
    let water_horz_range = water_map.horz_range();
    let horz_range = under_horz_range.start.min(water_horz_range.start)..under_horz_range.end.max(water_horz_range.end);
    let vert_range = under_map.vert_range();
    let horz_len = horz_range.end - horz_range.start;
    let horz_offset = horz_range.start;
    for r_ind in vert_range {
        let mut line_buf = vec!['.'; horz_len];
        if let Some(row) = under_map.map.get(&r_ind) {
            for range in &row.row {
                for c_ind in range.beg..range.end {
                    line_buf[c_ind - horz_offset] = '#';
                }
            }
        }

        if let Some(row) = water_map.map.get(&r_ind) {
            for range in &row.reach_row.row {
                for c_ind in range.beg..range.end {
                    line_buf[c_ind - horz_offset] = '|';
                }
            }

            for range in &row.rest_row.row {
                for c_ind in range.beg..range.end {
                    line_buf[c_ind - horz_offset] = '~';
                }
            }
        }

        println!("{}", line_buf.iter().collect::<String>());
    }

    Ok(())
}
