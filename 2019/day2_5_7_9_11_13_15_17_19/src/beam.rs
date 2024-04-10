use std::{fmt::Display, ops::Range};

use int_enum::IntEnum;

use crate::int_code::{
    com::{ProcessState, SeqIntCodeComputer},
    io::{Channel, SeqIODevice},
};

#[derive(Debug)]
pub enum Error {
    IntCodeError(crate::Error),
    InvalidDroneState(ProcessState),
    EmptyDroneResult(Point),
    InvalidDroneResult(i64),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IntCodeError(ice) => write!(
                f,
                "Get error({}) in execution of given intcode program",
                ice
            ),
            Error::InvalidDroneState(s) => write!(f, "Get invalid process state({:?})", s),
            Error::EmptyDroneResult(p) => write!(f, "Get empty drone result for point({:?})", p),
            Error::InvalidDroneResult(i) => {
                write!(f, "Get invalid drone result({}), expect 0 or 1", i)
            }
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Default, IntEnum, PartialEq, Eq)]
pub enum PointType {
    #[default]
    Stationary = 0,
    Pulled = 1,
}

#[derive(Debug, Clone, Copy)]
pub struct Point(usize, usize);

impl Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x(), self.y())
    }
}

impl Point {
    pub fn new(x: usize, y: usize) -> Self {
        Self(x, y)
    }

    pub fn x(&self) -> usize {
        self.0
    }

    pub fn y(&self) -> usize {
        self.1
    }
}

#[derive(Debug, Clone)]
pub enum ScannerState {
    InputX(Point),
    InputY(Point),
    WaitOutput(Point),
    End,
}

pub struct Area {
    points: Vec<PointType>,
    x_range: Range<usize>,
    y_range: Range<usize>,
}

impl Display for Area {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let col_n = self.x_range.len();
        for row_ind in 0..self.y_range.len() {
            let start_ind = row_ind * col_n;
            let row_str = self.points[start_ind..(start_ind + col_n)]
                .iter()
                .map(|pt| match pt {
                    PointType::Stationary => '.',
                    PointType::Pulled => '#',
                })
                .collect::<String>();
            writeln!(f, "{}", row_str)?;
        }

        Ok(())
    }
}

impl Area {
    pub fn new(x_range: Range<usize>, y_range: Range<usize>) -> Self {
        Self {
            points: vec![PointType::Stationary; x_range.len() * y_range.len()],
            x_range,
            y_range,
        }
    }

    pub fn p(&self, p: &Point) -> &PointType {
        let ind = self.p_to_ind(p);
        &self.points[ind]
    }

    pub fn p_mut(&mut self, p: &Point) -> &mut PointType {
        let ind = self.p_to_ind(p);
        &mut self.points[ind]
    }

    pub fn iter(&self) -> impl Iterator<Item = &PointType> {
        self.points.iter()
    }

    fn p_to_ind(&self, p: &Point) -> usize {
        assert!(self.x_range.contains(&p.x()) && self.y_range.contains(&p.y()));
        (p.y() - self.y_range.start) * self.x_range.len() + (p.x() - self.x_range.start)
    }
}

struct DroneSystem {
    code: Vec<i64>,
    io_dev: SeqIODevice<Channel>,
    computer: SeqIntCodeComputer,
}

impl DroneSystem {
    pub fn new(code: Vec<i64>) -> Self {
        Self {
            code,
            io_dev: SeqIODevice::new(Channel::new(&[])),
            computer: SeqIntCodeComputer::new(false),
        }
    }

    pub fn place_drone(&mut self, p: &Point) -> Result<PointType, Error> {
        self.io_dev.tweak(|chan| {
            let inputs = chan.data_mut();
            inputs.clear();
            inputs.push_back(i64::try_from(p.x()).unwrap());
            inputs.push_back(i64::try_from(p.y()).unwrap());
        });
        self.computer
            .execute_with_io(
                &self.code,
                self.io_dev.input_device(),
                self.io_dev.output_device(),
            )
            .map_err(Error::IntCodeError)
            .and_then(|res| {
                if res.state() != ProcessState::Halt {
                    return Err(Error::InvalidDroneState(res.state()));
                }

                self.io_dev.tweak(|chan| {
                    // println!("Place drone at {}, get {}", p, chan.data()[0]);
                    chan.data_mut()
                        .pop_front()
                        .ok_or(Error::EmptyDroneResult(*p))
                        .and_then(|v| {
                            u8::try_from(v)
                                .map_err(|_| Error::InvalidDroneResult(v))
                                .and_then(|u| {
                                    PointType::from_int(u).map_err(|_| Error::InvalidDroneResult(v))
                                })
                        })
                })
            })
    }
}

pub struct Scanner {
    drone_sys: DroneSystem,
    beam_row_ranges: Vec<Range<usize>>,
}

impl Scanner {
    pub fn new(drone_sys_code: Vec<i64>) -> Self {
        Self {
            drone_sys: DroneSystem::new(drone_sys_code),
            beam_row_ranges: Vec::new(),
        }
    }

    pub fn scan_area(
        &mut self,
        x_range: Range<usize>,
        y_range: Range<usize>,
    ) -> Result<Area, Error> {
        let mut area = Area::new(x_range.clone(), y_range.clone());
        for y in y_range {
            for x in x_range.clone() {
                let p = Point::new(x, y);
                *area.p_mut(&p) = self.drone_sys.place_drone(&p)?;
            }
        }

        Ok(area)
    }

    pub fn scan_first_block(
        &mut self,
        width: usize,
        height: usize,
    ) -> Result<(Point, Point), Error> {
        let mut first_check_row_ind = None;
        let mut row_ind = 0;
        loop {
            let row_range = self.scan_row(row_ind)?;
            let row_width = row_range.len();
            if let Some(first_check_row_ind) = first_check_row_ind {
                if row_ind >= first_check_row_ind {
                    // Assume this row is the bottom edge of block, top row is the top edge.
                    let top_row_ind = row_ind - height + 1;
                    let top_row_range = self.scan_row(top_row_ind)?;

                    // If block's width can fit into column range between this row's start column and top row's end column, then the block is found.
                    if top_row_range
                        .end
                        .checked_sub(row_range.start)
                        .is_some_and(|diff| diff >= width)
                    {
                        return Ok((
                            Point::new(row_range.start, top_row_ind),
                            Point::new(top_row_range.end - 1, row_ind),
                        ));
                    }
                }
            } else if row_width >= width {
                first_check_row_ind = Some(row_ind + height - 1);
            }

            row_ind += 1;
        }
    }

    fn scan_row(&mut self, ind: usize) -> Result<Range<usize>, Error> {
        if self.beam_row_ranges.is_empty() {
            let first_row_range = self.scan_row_linear(0, usize::MAX)?;
            self.beam_row_ranges.push(first_row_range);
        }

        let cached_row_end = self.beam_row_ranges.len();
        for scan_row_ind in cached_row_end..=ind {
            let this_row_range = self.scan_row_induct(scan_row_ind)?;
            self.beam_row_ranges.push(this_row_range);
        }

        Ok(self.beam_row_ranges[ind].clone())
    }

    fn scan_row_linear(
        &mut self,
        row_ind: usize,
        col_ind_limit: usize,
    ) -> Result<Range<usize>, Error> {
        let mut start_col_ind = None;
        let mut col_ind = 0;
        while col_ind <= col_ind_limit {
            match self.drone_sys.place_drone(&Point::new(col_ind, row_ind))? {
                PointType::Stationary => {
                    if let Some(start_col_ind) = start_col_ind {
                        return Ok(start_col_ind..col_ind);
                    }
                }
                PointType::Pulled => {
                    if start_col_ind.is_none() {
                        start_col_ind = Some(col_ind);
                    }
                }
            }

            col_ind += 1;
        }

        Ok(0..0)
    }

    fn scan_row_induct(&mut self, row_ind: usize) -> Result<Range<usize>, Error> {
        assert!(
            row_ind > 0 && (row_ind - 1 < self.beam_row_ranges.len()),
            "The last row({})'s range isn't cached, so can't induct this row range from it.",
            row_ind - 1
        );
        let last_row_range = self.beam_row_ranges[row_ind - 1].clone();
        if last_row_range.is_empty() {
            // Degrade into linear search, and assume the width of this row <= width of the next row
            return self.scan_row_linear(row_ind, row_ind * 2);
        }

        let start_col_ind = match self
            .drone_sys
            .place_drone(&Point::new(last_row_range.start, row_ind))?
        {
            // Outside of beam, scan to right for start of this beam row.
            PointType::Stationary => self.scan_first_col_point(
                (last_row_range.start + 1)..(last_row_range.start + 1 + last_row_range.len() * 2),
                row_ind,
                PointType::Pulled,
                usize::MAX,
            )?,
            // Inside of beam, scan to left for start of this beam row.
            PointType::Pulled => {
                self.scan_first_col_point(
                    (0..last_row_range.start).into_iter().rev(),
                    row_ind,
                    PointType::Stationary,
                    0,
                )? + 1
            }
        };

        assert!(
            last_row_range.end > 0,
            "There is a empty beam row({})",
            row_ind - 1
        );
        let end_col_ind = match self
            .drone_sys
            .place_drone(&Point::new(last_row_range.end, row_ind))?
        {
            PointType::Stationary => {
                // Outside of beam, scan to left side for end of this beam row.
                self.scan_first_col_point(
                    (0..last_row_range.end).into_iter().rev(),
                    row_ind,
                    PointType::Pulled,
                    0,
                )? + 1
            }
            PointType::Pulled => self.scan_first_col_point(
                // Inside of beam, scan to right for end of this beam row.
                (last_row_range.end + 1)..(last_row_range.end + 1 + last_row_range.len()),
                row_ind,
                PointType::Stationary,
                usize::MAX,
            )?,
        };

        Ok(start_col_ind..end_col_ind)
    }

    fn scan_first_col_point<I: Iterator<Item = usize>>(
        &mut self,
        col_ind_iter: I,
        row_ind: usize,
        target_pt: PointType,
        default: usize,
    ) -> Result<usize, Error> {
        col_ind_iter
            .map(|col_ind| {
                (
                    col_ind,
                    self.drone_sys.place_drone(&Point::new(col_ind, row_ind)),
                )
            })
            .filter(|(_, pt_res)| {
                pt_res.is_err() || pt_res.as_ref().is_ok_and(|pt| *pt == target_pt)
            })
            .map(|(col_ind, pt_res)| pt_res.map(|_| col_ind))
            .next()
            .unwrap_or(Ok(default))
    }
}
