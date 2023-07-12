pub mod sim;
pub mod unit;

use std::io;

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    NotMatchGroupPattern(String),
    NotMatchWithUnitPattern(String),
    NotMatchWithImmWeakPattern(String),
    UnknownDamageType(String),
    NoArmyLeft,
    SimulationInDraw,
}
