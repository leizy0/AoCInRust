use std::{
    cmp::{max, min},
    fs::File,
    io::{BufRead, BufReader},
    ops::{Range, RangeInclusive},
    path::Path,
    str::FromStr,
};

use crate::Error;

#[derive(Debug)]
pub struct Point {
    x: i32,
    y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Point {
        Point { x, y }
    }

    pub fn mht_dist(&self) -> i32 {
        self.x.abs() + self.y.abs()
    }
}

#[derive(Debug)]
pub struct CrossPoint {
    p: Point,
    total_length: u32,
}

impl CrossPoint {
    pub fn new(x: i32, y: i32, total_length: u32) -> CrossPoint {
        CrossPoint {
            p: Point::new(x, y),
            total_length,
        }
    }

    pub fn point(&self) -> &Point {
        &self.p
    }

    pub fn total_len(&self) -> u32 {
        self.total_length
    }
}

fn range_intersect(l_range: &RangeInclusive<i32>, r_range: &RangeInclusive<i32>) -> Vec<i32> {
    (*max(l_range.start(), r_range.start())..=*min(l_range.end(), r_range.end())).collect()
}

fn cross_horz_vert_seg(
    horz_seg: &HorizontalSegment,
    vert_seg: &VerticalSegment,
) -> Option<CrossPoint> {
    if vert_seg.y_range().contains(&horz_seg.y) && horz_seg.x_range().contains(&vert_seg.x) {
        let horz_len = horz_seg.start_length + vert_seg.x.abs_diff(horz_seg.x_begin) + 1;
        let vert_len = vert_seg.start_length + horz_seg.y.abs_diff(vert_seg.y_begin) + 1;
        Some(CrossPoint::new(vert_seg.x, horz_seg.y, horz_len + vert_len))
    } else {
        None
    }
}

struct HorizontalSegment {
    x_begin: i32, // Inclusive
    x_end: i32,   // Inclusive
    y: i32,
    start_length: u32,
}

impl HorizontalSegment {
    fn x_range(&self) -> RangeInclusive<i32> {
        min(self.x_begin, self.x_end)..=max(self.x_begin, self.x_end)
    }

    fn cross_horz_seg(&self, other: &HorizontalSegment) -> Vec<CrossPoint> {
        if self.y != other.y {
            Vec::new()
        } else {
            range_intersect(&self.x_range(), &other.x_range())
                .iter()
                .map(|&x| {
                    let length = self.start_length + x.abs_diff(self.x_begin) + 1;
                    let other_length = other.start_length + x.abs_diff(other.x_begin) + 1;
                    CrossPoint::new(x, self.y, length + other_length)
                })
                .collect()
        }
    }

    fn cross_vert_seg(&self, vert_seg: &VerticalSegment) -> Option<CrossPoint> {
        cross_horz_vert_seg(self, vert_seg)
    }
}

struct VerticalSegment {
    x: i32,
    y_begin: i32, // Inclusive
    y_end: i32,   // Inclusive
    start_length: u32,
}

impl VerticalSegment {
    fn y_range(&self) -> RangeInclusive<i32> {
        min(self.y_begin, self.y_end)..=max(self.y_begin, self.y_end)
    }

    fn cross_vert_seg(&self, other: &VerticalSegment) -> Vec<CrossPoint> {
        if self.x != other.x {
            Vec::new()
        } else {
            range_intersect(&self.y_range(), &other.y_range())
                .iter()
                .map(|&y| {
                    let length = self.start_length + y.abs_diff(self.y_begin) + 1;
                    let other_length = other.start_length + y.abs_diff(other.y_begin) + 1;
                    CrossPoint::new(self.x, y, length + other_length)
                })
                .collect()
        }
    }

    fn cross_horz_seg(&self, horz_seg: &HorizontalSegment) -> Option<CrossPoint> {
        cross_horz_vert_seg(horz_seg, self)
    }
}

pub struct Wire {
    horz_segs: Vec<HorizontalSegment>,
    vert_segs: Vec<VerticalSegment>,
}

impl FromStr for Wire {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut cur_point = Point::new(0, 0);
        let mut cur_length: u32 = 0;
        let mut wire = Wire {
            horz_segs: Vec::new(),
            vert_segs: Vec::new(),
        };

        for path in s.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
            let step_count = str::parse::<i32>(&path[1..])
                .map_err(|_| Error::ParsePathError(path.to_string()))?;
            match path.chars().nth(0).unwrap() {
                'U' => {
                    wire.vert_segs.push(VerticalSegment {
                        x: cur_point.x,
                        y_begin: cur_point.y + 1,
                        y_end: cur_point.y + step_count,
                        start_length: cur_length,
                    });
                    cur_point.y += step_count;
                }
                'D' => {
                    wire.vert_segs.push(VerticalSegment {
                        x: cur_point.x,
                        y_begin: cur_point.y - 1,
                        y_end: cur_point.y - step_count,
                        start_length: cur_length,
                    });
                    cur_point.y -= step_count;
                }
                'L' => {
                    wire.horz_segs.push(HorizontalSegment {
                        x_begin: cur_point.x - 1,
                        x_end: cur_point.x - step_count,
                        y: cur_point.y,
                        start_length: cur_length,
                    });
                    cur_point.x -= step_count;
                }
                'R' => {
                    wire.horz_segs.push(HorizontalSegment {
                        x_begin: cur_point.x + 1,
                        x_end: cur_point.x + step_count,
                        y: cur_point.y,
                        start_length: cur_length,
                    });
                    cur_point.x += step_count;
                }
                c => return Err(Error::UnknownPathDirection(c)),
            }

            cur_length += u32::try_from(step_count.abs()).unwrap();
        }

        wire.horz_segs.sort_unstable_by_key(|s| s.y);
        wire.vert_segs.sort_unstable_by_key(|s| s.x);

        Ok(wire)
    }
}

impl Wire {
    pub fn cross(&self, other: &Wire) -> Vec<CrossPoint> {
        let mut cross_points = Vec::new();
        for h_seg in &self.horz_segs {
            // Horizontal cross horizontal
            let same_y_left_ind = other.horz_segs.partition_point(|s| s.y < h_seg.y);
            let same_y_right_ind = other.horz_segs.partition_point(|s| s.y <= h_seg.y);
            if same_y_left_ind < same_y_right_ind {
                cross_points.extend(
                    other.horz_segs[same_y_left_ind..same_y_right_ind]
                        .iter()
                        .flat_map(|s| h_seg.cross_horz_seg(s)),
                );
            }

            // Horizontal cross vertical
            let in_x_range_left_ind = other
                .vert_segs
                .partition_point(|s| s.x < *h_seg.x_range().start());
            let in_x_range_right_ind = other
                .vert_segs
                .partition_point(|s| s.x <= *h_seg.x_range().end());
            if in_x_range_left_ind < in_x_range_right_ind {
                cross_points.extend(
                    other.vert_segs[in_x_range_left_ind..in_x_range_right_ind]
                        .iter()
                        .filter_map(|s| h_seg.cross_vert_seg(s)),
                );
            }
        }

        for v_seg in &self.vert_segs {
            // Vertical cross vertical
            let same_x_left_ind = other.vert_segs.partition_point(|s| s.x < v_seg.x);
            let same_x_right_ind = other.vert_segs.partition_point(|s| s.x <= v_seg.x);
            if same_x_left_ind < same_x_right_ind {
                cross_points.extend(
                    other.vert_segs[same_x_left_ind..same_x_right_ind]
                        .iter()
                        .flat_map(|s| v_seg.cross_vert_seg(s)),
                );
            }

            // Vertical cross horizontal
            let in_y_range_left_ind = other
                .horz_segs
                .partition_point(|s| s.y < *v_seg.y_range().start());
            let in_y_range_right_ind = other
                .horz_segs
                .partition_point(|s| s.y <= *v_seg.y_range().end());
            if in_y_range_left_ind < in_y_range_right_ind {
                cross_points.extend(
                    other.horz_segs[in_y_range_left_ind..in_y_range_right_ind]
                        .iter()
                        .filter_map(|s| v_seg.cross_horz_seg(s)),
                );
            }
        }

        cross_points
    }
}

pub fn read_wires<P>(path: P) -> Result<Vec<Wire>, Error>
where
    P: AsRef<Path>,
{
    let input_file = File::open(path).map_err(Error::IOError)?;
    let reader = BufReader::new(input_file);
    reader
        .lines()
        .map(|l| l.map_err(Error::IOError).and_then(|s| Wire::from_str(&s)))
        .collect::<Result<Vec<_>, Error>>()
}
