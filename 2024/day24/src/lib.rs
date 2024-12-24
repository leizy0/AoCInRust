use std::{
    collections::{HashMap, LinkedList},
    error,
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use clap::Parser;
use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Debug)]
pub enum Error {
    InvalidInitWireText(String),
    InvalidWireValueText(String),
    InvalidGateText(String),
    InvalidAndGateText(String),
    InvalidOrGateText(String),
    InvalidXorGateText(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidInitWireText(s) => {
                write!(f, "Invalid text({}) for initial wire settings.", s)
            }
            Error::InvalidWireValueText(s) => {
                write!(f, "Invalid text({}) for wire value, expect 0 or 1.", s)
            }
            Error::InvalidGateText(s) => write!(
                f,
                "Invalid text({}) for known gate, only support and, xor and or gates.",
                s
            ),
            Error::InvalidAndGateText(s) => write!(f, "Invalid text({}) for and gate.", s),
            Error::InvalidOrGateText(s) => write!(f, "Invalid text({}) for or gate.", s),
            Error::InvalidXorGateText(s) => write!(f, "Invalid text({}) for xor gate.", s),
        }
    }
}

impl error::Error for Error {}

#[derive(Debug, Parser)]
pub struct CLIArgs {
    pub input_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Wire {
    name: String,
}

impl Wire {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

trait Gate: std::fmt::Debug {
    fn input_wires(&self) -> &[Wire];
    fn output_wire(&self) -> &Wire;
    fn output(&self, inputs: &HashMap<Wire, Option<bool>>) -> Option<bool>;
}

#[derive(Debug)]
pub struct AndGate {
    inputs: [Wire; 2],
    output: Wire,
}

impl TryFrom<&str> for AndGate {
    type Error = Error;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        static AND_GATE_PATTERN: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"(\w+)\s+AND\s+(\w+)\s*->\s*(\w+)").unwrap());

        if let Some(caps) = AND_GATE_PATTERN.captures(value) {
            Ok(Self {
                inputs: [Wire::new(&caps[1].trim()), Wire::new(&caps[2].trim())],
                output: Wire::new(&caps[3].trim()),
            })
        } else {
            Err(Error::InvalidAndGateText(value.to_string()))
        }
    }
}

impl Gate for AndGate {
    fn input_wires(&self) -> &[Wire] {
        &self.inputs
    }

    fn output_wire(&self) -> &Wire {
        &self.output
    }

    fn output(&self, inputs: &HashMap<Wire, Option<bool>>) -> Option<bool> {
        let Some(left_input) = inputs.get(&self.inputs[0]).and_then(|value_op| *value_op) else {
            return None;
        };

        let Some(right_input) = inputs.get(&self.inputs[1]).and_then(|value_op| *value_op) else {
            return None;
        };

        Some(left_input && right_input)
    }
}

#[derive(Debug)]
pub struct OrGate {
    inputs: [Wire; 2],
    output: Wire,
}

impl TryFrom<&str> for OrGate {
    type Error = Error;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        static OR_GATE_PATTERN: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"(\w+)\s+OR\s+(\w+)\s*->\s*(\w+)").unwrap());

        if let Some(caps) = OR_GATE_PATTERN.captures(value) {
            Ok(Self {
                inputs: [Wire::new(&caps[1].trim()), Wire::new(&caps[2].trim())],
                output: Wire::new(&caps[3].trim()),
            })
        } else {
            Err(Error::InvalidOrGateText(value.to_string()))
        }
    }
}

impl Gate for OrGate {
    fn input_wires(&self) -> &[Wire] {
        &self.inputs
    }

    fn output_wire(&self) -> &Wire {
        &self.output
    }

    fn output(&self, inputs: &HashMap<Wire, Option<bool>>) -> Option<bool> {
        let Some(left_input) = inputs.get(&self.inputs[0]).and_then(|value_op| *value_op) else {
            return None;
        };

        let Some(right_input) = inputs.get(&self.inputs[1]).and_then(|value_op| *value_op) else {
            return None;
        };

        Some(left_input || right_input)
    }
}

#[derive(Debug)]
pub struct XorGate {
    inputs: [Wire; 2],
    output: Wire,
}

impl TryFrom<&str> for XorGate {
    type Error = Error;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        static XOR_GATE_PATTERN: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"(\w+)\s+XOR\s+(\w+)\s*->\s*(\w+)").unwrap());

        if let Some(caps) = XOR_GATE_PATTERN.captures(value) {
            Ok(Self {
                inputs: [Wire::new(&caps[1].trim()), Wire::new(&caps[2].trim())],
                output: Wire::new(&caps[3].trim()),
            })
        } else {
            Err(Error::InvalidXorGateText(value.to_string()))
        }
    }
}

impl Gate for XorGate {
    fn input_wires(&self) -> &[Wire] {
        &self.inputs
    }

    fn output_wire(&self) -> &Wire {
        &self.output
    }

    fn output(&self, inputs: &HashMap<Wire, Option<bool>>) -> Option<bool> {
        let Some(left_input) = inputs.get(&self.inputs[0]).and_then(|value_op| *value_op) else {
            return None;
        };

        let Some(right_input) = inputs.get(&self.inputs[1]).and_then(|value_op| *value_op) else {
            return None;
        };

        Some(left_input ^ right_input)
    }
}

#[derive(Debug)]
pub struct Circuit {
    wires: HashMap<Wire, Option<bool>>,
    gates: Vec<Box<dyn Gate>>,
}

impl Circuit {
    pub fn new() -> Self {
        Self {
            wires: HashMap::new(),
            gates: Vec::new(),
        }
    }

    pub fn init(&mut self, init_wires: &HashMap<Wire, bool>) {
        for (wire, value) in init_wires {
            if let Some(wire_value) = self.wires.get_mut(wire) {
                *wire_value = Some(*value);
            }
        }
    }

    pub fn simulate(&mut self) -> Option<usize> {
        let mut non_output_gates = LinkedList::from_iter(self.gates.iter());
        loop {
            let mut new_non_output_gates = LinkedList::new();
            let non_output_gates_n = non_output_gates.len();
            while let Some(gate) = non_output_gates.pop_front() {
                if let Some(value) = gate.output(&self.wires) {
                    if let Some(output_value) = self.wires.get_mut(gate.output_wire()) {
                        *output_value = Some(value);
                    }
                } else {
                    new_non_output_gates.push_back(gate);
                }
            }

            if new_non_output_gates.len() == non_output_gates_n {
                break;
            } else {
                non_output_gates = new_non_output_gates;
            }
        }

        let mut output_wires = (0..)
            .into_iter()
            .map_while(|bit_ind| {
                Some(Wire::new(&format!("z{:02}", bit_ind)))
                    .filter(|wire| self.wires.contains_key(wire))
            })
            .collect::<Vec<_>>();
        output_wires.reverse();
        output_wires
            .iter()
            .map(|wire| self.wires[wire])
            .fold(Some(0), |n_op, value_op| {
                if let Some(mut n) = n_op {
                    if let Some(value) = value_op {
                        n = n * 2 + if value { 1 } else { 0 };

                        Some(n)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
    }

    fn add_gate(&mut self, text: &str) -> Result<(), Error> {
        type GateConstructorFn = fn(&str) -> Result<Box<dyn Gate>, Error>;
        static GATE_CONSTRUCTORS: Lazy<Vec<GateConstructorFn>> = Lazy::new(|| {
            Vec::from([
                (|text: &str| AndGate::try_from(text).map(|gate| Box::new(gate) as Box<dyn Gate>))
                    as GateConstructorFn,
                (|text: &str| OrGate::try_from(text).map(|gate| Box::new(gate) as Box<dyn Gate>))
                    as GateConstructorFn,
                (|text: &str| XorGate::try_from(text).map(|gate| Box::new(gate) as Box<dyn Gate>))
                    as GateConstructorFn,
            ])
        });

        let gate = GATE_CONSTRUCTORS
            .iter()
            .flat_map(|constructor| constructor(text))
            .next()
            .ok_or(Error::InvalidGateText(text.to_string()))?;
        for input_wire in gate.input_wires() {
            self.wires.entry(input_wire.clone()).or_insert(None);
        }
        self.wires.entry(gate.output_wire().clone()).or_insert(None);
        self.gates.push(gate);

        Ok(())
    }
}

pub fn read_circuit_info<P: AsRef<Path>>(path: P) -> Result<(HashMap<Wire, bool>, Circuit)> {
    let file = File::open(&path)
        .with_context(|| format!("Failed to open given file({}).", path.as_ref().display()))?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    let mut init_wires = HashMap::new();
    let mut line_ind = 1;
    while let Some(line) = lines.next() {
        let line = line.with_context(|| {
            format!(
                "Failed to read line {} in given file({}).",
                line_ind + 1,
                path.as_ref().display()
            )
        })?;
        line_ind += 1;
        if line.is_empty() {
            break;
        }

        let comma_ind = line
            .find(':')
            .ok_or(Error::InvalidInitWireText(line.clone()))?;
        let value_text = &line[(comma_ind + 1)..].trim();
        let value = value_text
            .parse::<u8>()
            .ok()
            .and_then(|n| match n {
                0 => Some(false),
                1 => Some(true),
                _ => None,
            })
            .ok_or(Error::InvalidWireValueText(value_text.to_string()))?;
        let wire = Wire::new(&line[..comma_ind].trim());
        init_wires.insert(wire, value);
    }

    let mut circuit = Circuit::new();
    while let Some(line) = lines.next() {
        let line = line.with_context(|| {
            format!(
                "Failed to read line {} in given file({}).",
                line_ind + 1,
                path.as_ref().display()
            )
        })?;
        line_ind += 1;

        circuit.add_gate(&line)?;
    }

    Ok((init_wires, circuit))
}
