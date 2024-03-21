use std::{collections::HashMap, fmt::Display};

use int_enum::IntEnum;

use crate::{
    int_code::io::{InputPort, OutputPort},
    Error as ExecutionError,
};

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
                write!(f, "Invalid command from outputs({:?}) of paint program, expect exactly two numbers(the first is for painting color, the second is for turnning direction", c)
            }
            Error::InvalidPaintColor(n) => write!(
                f,
                "Invalid paint color number({}) found in output of paint program, expect 0(black) or 1(white)",
                n
            ),
            Error::InvalidTurnDirection(n) => write!(
                f,
                "Invalid turn direction number({}) found in output of paint program, expect 0(left) or 1(right)",
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
pub enum Color {
    Black = 0,
    White = 1,
}

pub struct Image {
    pixels: Vec<Color>,
    width: usize,
    height: usize,
}

impl Image {
    fn from_blocks(blocks: &HashMap<Block, Color>) -> Self {
        let (mut x_min, mut y_min, mut x_max, mut y_max) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
        for (b, _) in blocks {
            if b.x < x_min {
                x_min = b.x;
            } else if b.x > x_max {
                x_max = b.x;
            }

            if b.y < y_min {
                y_min = b.y;
            } else if b.y > y_max {
                y_max = b.y;
            }
        }

        let width = x_max - x_min + 1;
        let height = y_max - y_min + 1;
        if width <= 0 || height <= 0 {
            Self {
                pixels: Vec::new(),
                width: 0,
                height: 0,
            }
        } else {
            let mut pixels = vec![Color::Black; (width * height) as usize];
            for (b, &c) in blocks {
                let r_ind = height - 1 - (b.y - y_min);
                let c_ind = b.x - x_min;
                let ind = r_ind * width + c_ind;
                pixels[ind as usize] = c;
            }

            Self {
                pixels,
                width: width as usize,
                height: height as usize,
            }
        }
    }
}

impl Display for Image {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for r in 0..self.height {
            let r_offset = r * self.width;
            for c in 0..self.width {
                let ind = r_offset + c;
                write!(f, "{}", self.pixels[ind].int_value())?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
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

    pub fn paint(&mut self, color: Color) {
        *self.paint_blocks.entry(self.cur_block).or_insert(color) = color;
        self.paint_count += 1;
    }

    pub fn image(&self) -> Image {
        Image::from_blocks(&self.paint_blocks)
    }

    fn cur_color(&self) -> Color {
        self.paint_blocks
            .get(&self.cur_block)
            .copied()
            .unwrap_or(Color::Black)
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

pub struct PaintSimulator {
    robot: PaintRobot,
    sim_proc_id: Option<usize>,
    sim_proc_output: Vec<i64>,
}

impl InputPort for PaintSimulator {
    fn get(&mut self) -> Option<i64> {
        // Report current state to input
        Some(self.robot.cur_color().int_value().into())
    }

    fn reg_proc(&mut self, proc_id: usize) {
        self.sim_proc_id = Some(proc_id);
    }
}

impl OutputPort for PaintSimulator {
    fn put(&mut self, value: i64) -> Result<(), ExecutionError> {
        // Get command from simulation process
        if self.sim_proc_output.len() % 2 == 0 {
            // Paint command
            let paint_color = u8::try_from(value)
                .ok()
                .and_then(|n| Color::from_int(n).ok())
                .ok_or(ExecutionError::IOProcessError(
                    Error::InvalidPaintColor(value).to_string(),
                ))?;
            self.robot.paint(paint_color);
        } else {
            // Turn around and forward command
            match value {
                0 => self.robot.turn_left(),
                1 => self.robot.turn_right(),
                _ => {
                    return Err(ExecutionError::IOProcessError(
                        Error::InvalidTurnDirection(value).to_string(),
                    ))
                }
            }
            self.robot.forward();
        }
        self.sim_proc_output.push(value);

        Ok(())
    }

    fn wait_proc_id(&self) -> Option<usize> {
        self.sim_proc_id
    }
}

impl PaintSimulator {
    pub fn new(robot: PaintRobot) -> Self {
        Self {
            robot,
            sim_proc_id: None,
            sim_proc_output: Vec::new(),
        }
    }

    pub fn outputs(&self) -> &[i64] {
        &self.sim_proc_output
    }

    pub fn robot(&self) -> &PaintRobot {
        &self.robot
    }
}
