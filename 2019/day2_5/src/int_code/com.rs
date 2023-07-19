use crate::Error;

use super::inst::{Instruction, parse_cur_inst};

pub trait ExecutionState {
    fn image(&self) -> &[i32];
    fn image_mut(&mut self) -> &mut [i32];
    fn input(&mut self) -> Option<i32>;
    fn output(&mut self, value: i32);
    fn inst_p_mut(&mut self) -> &mut usize;
    fn halt(&mut self);
}

pub struct IntCodeComputer {
}

pub struct ExecutionResult {
    step_count: usize,
    image: Vec<i32>,
    outputs: Vec<i32>,
}

impl ExecutionResult {
    fn new(step_count: usize, image: Vec<i32>, outputs: Vec<i32>) -> ExecutionResult {
        ExecutionResult { step_count, image, outputs }
    }

    pub fn step_count(&self) -> usize {
        self.step_count
    }

    pub fn image(&self) -> &[i32] {
        &self.image
    }

    pub fn outputs(&self) -> &[i32] {
        &self.outputs
    }
}

impl IntCodeComputer {
    pub fn new() -> Self {
        IntCodeComputer {  }
    }

    pub fn execute(&mut self, mut image: Vec<i32>, inputs: Vec<i32>) -> Result<ExecutionResult, Error> {
        let mut process = Process::new(&mut image, inputs);
        let mut step_count = 0;
        loop {
            let inst = process.cur_inst()?;
            inst.execute(&mut process)?;
            step_count += 1;
            if process.is_halt() {
                break;
            }
        }

        let outputs = process.outputs;
        Ok(ExecutionResult::new(step_count, image, outputs))
    }
}

struct Process<'a> {
    inst_p: usize,
    image: &'a mut Vec<i32>,
    is_halt: bool,
    inputs: Vec<i32>,
    input_ind: usize,
    outputs: Vec<i32>,
}

impl <'a> Process<'a> {
    fn new(image: &'a mut Vec<i32>, inputs: Vec<i32>) -> Self {
        Process {
            inst_p: 0,
            image,
            is_halt: false,
            inputs,
            input_ind: 0,
            outputs: Vec::new(),
        }
    }

    fn cur_inst(&self) -> Result<Box<dyn Instruction>, Error> {
        if self.inst_p >= self.image.len() {
            Err(Error::ExecutionExceedIntCode(self.inst_p, self.image.len()))
        } else {
            parse_cur_inst(&self.image[self.inst_p..])
        }
    }

    fn is_halt(&self) -> bool {
        self.is_halt
    }
}

impl <'a> ExecutionState for Process<'a> {
    fn image(&self) -> &[i32] {
        &self.image
    }

    fn image_mut(&mut self) -> &mut [i32] {
        &mut self.image
    }

    fn input(&mut self) -> Option<i32> {
        let cur_ind = self.input_ind;

        if cur_ind < self.inputs.len() {
            self.input_ind += 1;
            Some(self.inputs[cur_ind])
        } else {
            None
        }
    }

    fn output(&mut self, value: i32) {
        self.outputs.push(value);
    }

    fn halt(&mut self) {
        self.is_halt = true;
    }

    fn inst_p_mut(&mut self) -> &mut usize {
        &mut self.inst_p
    }
}
