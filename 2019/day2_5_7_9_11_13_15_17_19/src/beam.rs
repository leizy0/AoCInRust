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
}

impl Scanner {
    pub fn new(drone_sys_code: Vec<i64>) -> Self {
        Self {
            drone_sys: DroneSystem::new(drone_sys_code),
        }
    }

    pub fn scan(&mut self, x_range: Range<usize>, y_range: Range<usize>) -> Result<Area, Error> {
        let mut area = Area::new(x_range.clone(), y_range.clone());
        for y in y_range {
            for x in x_range.clone() {
                let p = Point::new(x, y);
                *area.p_mut(&p) = self.drone_sys.place_drone(&p)?;
            }
        }

        Ok(area)
    }
}
