use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    rc::Rc,
};

use crate::Error;

use super::inst::{parse_cur_inst, Instruction};

pub trait ExecutionState {
    fn read_mem(&mut self, ind: usize) -> i64;
    fn write_mem(&mut self, ind: usize, value: i64);
    fn input(&mut self) -> Option<i64>;
    fn output(&mut self, value: i64) -> Result<(), Error>;
    fn inst_p_mut(&mut self) -> &mut usize;
    fn rel_base(&self) -> i64;
    fn rel_base_mut(&mut self) -> &mut i64;
    fn halt(&mut self);
}

// Input port for process, data source
pub trait InputPort {
    fn get(&mut self) -> Option<i64>;
    fn reg_proc(&mut self, proc_id: usize);
}

// Output port for process, data sink
pub trait OutputPort {
    fn put(&mut self, value: i64) -> Result<(), Error>;
    fn wait_proc_id(&self) -> Option<usize>;
}

pub struct IntCodeComputer {
    pub enable_debug_output: bool,
    processes: Vec<Option<Process>>,
}

impl IntCodeComputer {
    pub fn new(enable_debug_output: bool) -> Self {
        IntCodeComputer {
            enable_debug_output,
            processes: Vec::new(),
        }
    }

    // pub fn execute(&mut self, image: Vec<i64>, inputs: Vec<i64>) -> Result<ExecutionResult, Error> {
    //     let input_chan_id = self.new_channel(&inputs);
    //     let output_chan_id = self.new_channel(&[]);
    //     let proc_id = self.new_proc(&image, input_chan_id, output_chan_id);

    //     self.exe_procs(&[proc_id], proc_id)
    //         .and_then(|mut proc_res| {
    //             proc_res
    //                 .extract(proc_id, output_chan_id)
    //                 .ok_or(Error::ProcessResultNotFound(proc_id, output_chan_id))
    //         })
    // }

    pub fn execute_with_io(
        &mut self,
        image: &[i64],
        input_port: Rc<RefCell<dyn InputPort>>,
        output_port: Rc<RefCell<dyn OutputPort>>,
    ) -> Result<ProcessResult, Error> {
        let proc_id = self.new_proc(&image, input_port, output_port);
        loop {
            self.exe_proc(proc_id)?;
            let state = self.proc(proc_id).unwrap().state;
            if state == ProcessState::Halt || state == ProcessState::Block {
                break;
            }
        }

        Ok(self.take_proc(proc_id).map(|p| p.into_snap()).unwrap())
    }

    pub fn exe_procs(
        &mut self,
        proc_ids: &[usize],
        start_proc_ind: usize,
    ) -> Result<ProcsExecutionResult, Error> {
        let mut cur_proc_id_ind = Some(start_proc_ind);
        loop {
            match cur_proc_id_ind {
                Some(ind) => {
                    self.exe_proc(proc_ids[ind])?;

                    cur_proc_id_ind = None;
                    for i in 0..proc_ids.len() {
                        let next_proc_id_ind = (i + ind) % proc_ids.len();
                        let next_proc_id = proc_ids[next_proc_id_ind];
                        if self
                            .proc(next_proc_id)
                            .is_some_and(|p| p.state == ProcessState::Ready)
                        {
                            cur_proc_id_ind = Some(next_proc_id_ind);
                            break;
                        }
                    }
                }
                None => {
                    let images = proc_ids
                        .iter()
                        .flat_map(|&id| self.take_proc(id).map(|p| (id, p.into_snap())))
                        .collect::<HashMap<_, _>>();
                    return Ok(ProcsExecutionResult::new(images));
                }
            };
        }
    }

    pub fn exe_proc(&mut self, cur_proc_id: usize) -> Result<(), Error> {
        self.proc_mut(cur_proc_id)
            .ok_or(Error::RunningUnknownProcess(cur_proc_id))?;

        let debug_enabled = self.enable_debug_output;
        let mut run_proc = RunningProcess {
            computer: self,
            run_proc_id: cur_proc_id,
        };
        if run_proc.run_proc_mut().state != ProcessState::Ready {
            return Ok(());
        }

        run_proc.run_proc_mut().state = ProcessState::Running;
        loop {
            let inst = run_proc.run_proc_mut().cur_inst()?;
            if debug_enabled {
                let step_count = run_proc.run_proc_mut().step_count;
                let inst_p = run_proc.run_proc_mut().inst_p();
                println!(
                    "Process({}) step # {}: {:?} @ {}.",
                    cur_proc_id, step_count, inst, inst_p
                );
            }

            match inst.execute(&mut run_proc) {
                Ok(_) => {
                    run_proc.run_proc_mut().step_count += 1;
                    if run_proc.run_proc().is_halt() {
                        if debug_enabled {
                            println!("Process({}) halt.", cur_proc_id);
                        }
                        break;
                    }
                }
                Err(e) => match e {
                    Error::NotEnoughInput => {
                        // Process is blocked by input instruction when input of this process has no data
                        run_proc.run_proc_mut().state = ProcessState::Block;
                        if debug_enabled {
                            println!("Process({}) blocked by requiring input.", cur_proc_id);
                        }
                        break;
                    }
                    _ => return Err(e),
                },
            };
        }

        Ok(())
    }

    pub fn new_proc(
        &mut self,
        int_code: &[i64],
        input_port: Rc<RefCell<dyn InputPort>>,
        output_port: Rc<RefCell<dyn OutputPort>>,
    ) -> usize {
        let proc = Process::new(int_code, input_port.clone(), output_port);
        let mut exist_slot_id = None;
        for (i, slot) in self.processes.iter_mut().enumerate() {
            if slot.is_none() {
                exist_slot_id = Some(i);
                break;
            }
        }

        let id = match exist_slot_id {
            Some(id) => {
                self.processes[id] = Some(proc);
                id
            }
            None => {
                self.processes.push(Some(proc));
                self.processes.len() - 1
            }
        };
        input_port.borrow_mut().reg_proc(id);

        id
    }

    fn take_proc(&mut self, proc_id: usize) -> Option<Process> {
        self.processes.get_mut(proc_id).and_then(|o| o.take())
    }

    fn proc_mut(&mut self, proc_id: usize) -> Option<&mut Process> {
        self.processes.get_mut(proc_id).and_then(|o| o.as_mut())
    }

    fn proc(&self, proc_id: usize) -> Option<&Process> {
        self.processes.get(proc_id).and_then(|o| o.as_ref())
    }

    fn awake_proc(&mut self, proc_id: usize) {
        match self.proc_mut(proc_id) {
            Some(p) => {
                if p.state == ProcessState::Block {
                    p.state = ProcessState::Ready;
                }
            }
            _ => (),
        }
    }
}

pub struct Channel {
    data: VecDeque<i64>,
    output_reg_proc_id: Option<usize>,
}

impl InputPort for Channel {
    fn get(&mut self) -> Option<i64> {
        self.data.pop_front()
    }

    fn reg_proc(&mut self, proc_id: usize) {
        self.output_reg_proc_id = Some(proc_id);
    }
}

impl OutputPort for Channel {
    fn put(&mut self, value: i64) -> Result<(), Error> {
        Ok(self.data.push_back(value))
    }

    fn wait_proc_id(&self) -> Option<usize> {
        self.output_reg_proc_id
    }
}

impl Channel {
    pub fn new(init_input: &[i64]) -> Self {
        Self {
            data: VecDeque::from_iter(init_input.iter().copied()),
            output_reg_proc_id: None,
        }
    }

    pub fn data(&self) -> &VecDeque<i64> {
        &self.data
    }
}

pub struct ProcessResult {
    step_count: usize,
    state: ProcessState,
    image: Vec<i64>,
}

impl ProcessResult {
    pub fn state(&self) -> ProcessState {
        self.state
    }

    pub fn step_count(&self) -> usize {
        self.step_count
    }

    pub fn image(&self) -> &[i64] {
        &self.image
    }
}

pub struct ProcsExecutionResult {
    proc_snapshots: HashMap<usize, ProcessResult>,
}

impl ProcsExecutionResult {
    fn new(proc_snapshots: HashMap<usize, ProcessResult>) -> Self {
        Self { proc_snapshots }
    }

    pub fn proc_snapshots(&self, proc_id: usize) -> Option<&ProcessResult> {
        self.proc_snapshots.get(&proc_id)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ProcessState {
    Ready,
    Running,
    Block,
    Halt,
}

struct Process {
    state: ProcessState,
    inst_p: usize,
    mem: Vec<i64>,
    input_port: Rc<RefCell<dyn InputPort>>,
    output_port: Rc<RefCell<dyn OutputPort>>,
    step_count: usize,
    rel_base: i64,
}

impl Process {
    fn new(
        image: &[i64],
        input_port: Rc<RefCell<dyn InputPort>>,
        output_port: Rc<RefCell<dyn OutputPort>>,
    ) -> Self {
        Process {
            state: ProcessState::Ready,
            inst_p: 0,
            mem: Vec::from(image),
            input_port,
            output_port,
            step_count: 0,
            rel_base: 0,
        }
    }

    fn inst_p(&self) -> usize {
        self.inst_p
    }

    fn cur_inst(&self) -> Result<Box<dyn Instruction>, Error> {
        if self.inst_p >= self.mem.len() {
            Err(Error::ExecutionExceedIntCode(self.inst_p, self.mem.len()))
        } else {
            parse_cur_inst(&self.mem[self.inst_p..])
        }
    }

    fn is_halt(&self) -> bool {
        self.state == ProcessState::Halt
    }

    fn into_snap(self) -> ProcessResult {
        ProcessResult {
            step_count: self.step_count,
            state: self.state,
            image: self.mem,
        }
    }
}

struct RunningProcess<'a> {
    computer: &'a mut IntCodeComputer,
    run_proc_id: usize,
}

impl<'a> RunningProcess<'a> {
    fn run_proc(&self) -> &Process {
        self.computer.proc(self.run_proc_id).unwrap()
    }

    fn run_proc_mut(&mut self) -> &mut Process {
        self.computer.proc_mut(self.run_proc_id).unwrap()
    }
}

impl<'a> ExecutionState for RunningProcess<'a> {
    fn read_mem(&mut self, ind: usize) -> i64 {
        if ind < self.run_proc().mem.len() {
            self.run_proc().mem[ind]
        } else {
            self.run_proc_mut().mem.resize(ind + 1, 0);
            self.run_proc().mem[ind]
        }
    }

    fn write_mem(&mut self, ind: usize, value: i64) {
        if ind >= self.run_proc().mem.len() {
            self.run_proc_mut().mem.resize(ind + 1, 0);
        }

        self.run_proc_mut().mem[ind] = value;
    }

    fn input(&mut self) -> Option<i64> {
        let res = self.run_proc_mut().input_port.borrow_mut().get();
        if res.is_none() {
            assert!(self.run_proc().state == ProcessState::Running);
            self.run_proc_mut().state = ProcessState::Block;
        }

        res
    }

    fn output(&mut self, value: i64) -> Result<(), Error> {
        self.run_proc_mut().output_port.borrow_mut().put(value)?;

        let wait_proc_id = self.run_proc_mut().output_port.borrow().wait_proc_id();
        match wait_proc_id {
            Some(id) => {
                if self.computer.enable_debug_output {
                    println!(
                        "Process({}) output data({}) and try to awake process({})",
                        self.run_proc_id, value, id
                    );
                }
                self.computer.awake_proc(id)
            }
            _ => (),
        };

        Ok(())
    }

    fn halt(&mut self) {
        self.run_proc_mut().state = ProcessState::Halt;
    }

    fn inst_p_mut(&mut self) -> &mut usize {
        &mut self.run_proc_mut().inst_p
    }

    fn rel_base(&self) -> i64 {
        self.run_proc().rel_base
    }

    fn rel_base_mut(&mut self) -> &mut i64 {
        &mut self.run_proc_mut().rel_base
    }
}
