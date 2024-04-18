use std::{fmt::Display, iter};

use crate::int_code::io::{InputPort, OutputPort};

#[derive(Debug)]
pub enum Error {
    MultipleDamageInDetection(usize, i64), // (earlier damage, current value)
    InvalidOutputInDetection(i64),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MultipleDamageInDetection(last_d, cur_v) => write!(f, "Multiple damage value({} -> {}) returned by hull detection program", last_d, cur_v),
            Error::InvalidOutputInDetection(v) => write!(f, "Invalid output value({}) in hull detection program.", v),
        }
    }
}

pub enum RRegister {
    A, // ground detector(1 distance away)
    B, // ground detector(2 distance away)
    C, // ground detector(3 distance away)
    D, // ground detector(4 distance away)
    T, // Temporary
    J, // Jump
}

impl Display for RRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            RRegister::A => 'A',
            RRegister::B => 'B',
            RRegister::C => 'C',
            RRegister::D => 'D',
            RRegister::T => 'T',
            RRegister::J => 'J',
        };

        write!(f, "{}", name)
    }
}

pub enum WRegister {
    T, // Temporary
    J, // Jump
}

impl Display for WRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            WRegister::T => 'T',
            WRegister::J => 'J',
        };

        write!(f, "{}", name)
    }
}

pub enum SSInstruction {
    Or(RRegister, WRegister),
    And(RRegister, WRegister),
    Not(RRegister, WRegister),
}

impl Display for SSInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SSInstruction::Or(rr, wr) => write!(f, "OR {} {}", rr, wr),
            SSInstruction::And(rr, wr) => write!(f, "AND {} {}", rr, wr),
            SSInstruction::Not(rr, wr) => write!(f, "NOT {} {}", rr, wr),
        }
    }
}

impl SSInstruction {
    pub fn or(rr: RRegister, wr: WRegister) -> Self {
        Self::Or(rr, wr)
    }

    pub fn and(rr: RRegister, wr: WRegister) -> Self {
        Self::And(rr, wr)
    }

    pub fn not(rr: RRegister, wr: WRegister) -> Self {
        Self::Not(rr, wr)
    }
}

pub struct HullDetector {
    script_chars: Vec<char>,
    input_ind: usize,
    damage: Option<usize>,
    log: String,
}

impl HullDetector {
    pub fn new(script: &[SSInstruction]) -> Self {
        let script_chars = script
            .iter()
            .map(|inst| inst.to_string())
            .chain(iter::once("WALK".to_string()))
            .flat_map(|s| s.chars().chain(iter::once('\n')).collect::<Vec<_>>())
            .collect::<Vec<_>>();

        Self {
            script_chars,
            input_ind: 0,
            damage: None,
            log: String::new(),
        }
    }

    pub fn damage(&self) -> Option<usize> {
        self.damage
    }

    pub fn log(&self) -> &str {
        &self.log
    }
}

impl InputPort for HullDetector {
    fn get(&mut self) -> Option<i64> {
        let res = self.script_chars.get(self.input_ind).map(|c| *c as i64);
        if self.input_ind < self.script_chars.len() {
            self.input_ind += 1;
        }

        res
    }

    fn reg_proc(&mut self, _proc_id: usize) {}
}

impl OutputPort for HullDetector {
    fn put(&mut self, value: i64) -> Result<(), crate::Error> {
        fn to_ascii(value: i64) -> Option<char> {
            u32::try_from(value)
                .ok()
                .and_then(|v| char::from_u32(v).filter(|c| c.is_ascii()))
        }

        if let Some(c) = to_ascii(value) {
            self.log.push(c);
        } else if let Ok(v) = usize::try_from(value) {
            if let Some(d) = self.damage {
                return Err(crate::Error::IOProcessError(
                    Error::MultipleDamageInDetection(d, value).to_string(),
                ));
            } else {
                self.damage = Some(v);
            }
        } else {
            return Err(crate::Error::IOProcessError(
                Error::InvalidOutputInDetection(value).to_string(),
            ));
        }

        Ok(())
    }

    fn wait_proc_id(&self) -> Option<usize> {
        None
    }
}
