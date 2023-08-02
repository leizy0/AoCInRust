use std::{
    collections::HashMap,
    fmt::Display,
    sync::atomic::{AtomicBool, Ordering},
};

use int_enum::IntEnum;

use crate::{int_code::com::InputPort, Error as ExecutionError};

#[derive(Debug)]
pub enum Error {
    InvalidCommand(Vec<i64>),
    InvalidPaintColor(i64),
    InvalidTurnDirection(i64),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidCommand(c) => {
                write!(f, "Invalid command from outputs({:?}) of paint program", c)
            }
            Error::InvalidPaintColor(n) => write!(
                f,
                "Invalid paint color number({}) found in output of paint program",
                n
            ),
            Error::InvalidTurnDirection(n) => write!(
                f,
                "Invalid turn direction number({}) found in output of paint program",
                n
            ),
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
struct Block {
    x: i32,
    y: i32,
}

impl Block {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[repr(u8)]
#[derive(IntEnum, Debug, Clone, Copy)]
enum Direction {
    North = 0,
    East = 1,
    South = 2,
    West = 3,
}

impl Direction {
    fn left(&self) -> Self {
        Self::from_int((self.int_value() + 4 - 1) % 4).unwrap()
    }

    fn right(&self) -> Self {
        Self::from_int((self.int_value() + 1) % 4).unwrap()
    }

    fn unit_vec(&self) -> (i32, i32) {
        match self {
            Direction::North => (0, 1),
            Direction::East => (1, 0),
            Direction::South => (0, -1),
            Direction::West => (-1, 0),
        }
    }
}

#[repr(u8)]
#[derive(IntEnum, Debug, Clone, Copy)]
enum Color {
    Black = 0,
    White = 1,
}

pub struct PaintRobot {
    cur_dir: Direction,
    cur_block: Block,
    paint_count: usize,
    paint_blocks: HashMap<Block, Color>,
}

impl PaintRobot {
    pub fn new() -> Self {
        Self {
            cur_dir: Direction::North,
            cur_block: Block::new(0, 0),
            paint_count: 0,
            paint_blocks: HashMap::new(),
        }
    }

    pub fn paint_count(&self) -> usize {
        self.paint_count
    }

    pub fn block_count(&self) -> usize {
        self.paint_blocks.len()
    }

    fn cur_color(&self) -> Color {
        self.paint_blocks
            .get(&self.cur_block)
            .copied()
            .unwrap_or(Color::Black)
    }

    fn paint(&mut self, color: Color) {
        *self.paint_blocks.entry(self.cur_block).or_insert(color) = color;
        self.paint_count += 1;
    }

    fn turn_left(&mut self) {
        self.cur_dir = self.cur_dir.left();
    }

    fn turn_right(&mut self) {
        self.cur_dir = self.cur_dir.right();
    }

    fn forward(&mut self) {
        let (offset_x, offset_y) = self.cur_dir.unit_vec();
        self.cur_block.x += offset_x;
        self.cur_block.y += offset_y;
    }
}

pub fn sim_paint(
    robot: &mut PaintRobot,
    input_port: &mut dyn InputPort,
    outputs: &[i64],
) -> Result<(), ExecutionError> {
    static IS_INIT: AtomicBool = AtomicBool::new(true);

    // Run command from output
    if IS_INIT.fetch_and(false, Ordering::Relaxed) {
        // At first call, don't process outputs
    } else if outputs.len() != 2 {
        return Err(ExecutionError::IOProcessError(
            Error::InvalidCommand(Vec::from(outputs)).to_string(),
        ));
    } else {
        let paint_color = u8::try_from(outputs[0])
            .ok()
            .and_then(|n| Color::from_int(n).ok())
            .ok_or(ExecutionError::IOProcessError(
                Error::InvalidPaintColor(outputs[0]).to_string(),
            ))?;
        robot.paint(paint_color);
        match outputs[1] {
            0 => robot.turn_left(),
            1 => robot.turn_right(),
            _ => {
                return Err(ExecutionError::IOProcessError(
                    Error::InvalidTurnDirection(outputs[1]).to_string(),
                ))
            }
        }
        robot.forward();
    }

    // Report current state to input
    input_port.input(robot.cur_color().int_value().into());

    Ok(())
}
