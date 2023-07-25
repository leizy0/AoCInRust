use std::collections::{HashMap, HashSet, VecDeque};

use crate::Error;

use super::inst::{parse_cur_inst, Instruction};

pub trait ExecutionState {
    fn image(&self) -> &[i32];
    fn image_mut(&mut self) -> &mut [i32];
    fn input(&mut self) -> Option<i32>;
    fn output(&mut self, value: i32);
    fn inst_p_mut(&mut self) -> &mut usize;
    fn halt(&mut self);
}

pub struct IntCodeComputer {
    pub enable_debug_output: bool,
    channels: Vec<Option<Channel>>,
    processes: Vec<Option<Process>>,
}

impl IntCodeComputer {
    pub fn new(enable_debug_output: bool) -> Self {
        IntCodeComputer {
            enable_debug_output,
            channels: Vec::new(),
            processes: Vec::new(),
        }
    }

    pub fn execute(&mut self, image: Vec<i32>, inputs: Vec<i32>) -> Result<ExecutionResult, Error> {
        let input_chan_id = self.new_channel(&inputs);
        let output_chan_id = self.new_channel(&[]);
        let proc_id = self.new_proc(&image, input_chan_id, output_chan_id);

        self.exe_procs(&[proc_id], proc_id)
            .and_then(|mut proc_res| {
                proc_res
                    .extract(proc_id, output_chan_id)
                    .ok_or(Error::ProcessResultNotFound(proc_id, output_chan_id))
            })
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
                    let output_chan_ids = proc_ids
                        .iter()
                        .flat_map(|pid| self.proc(*pid).map(|o| o.output_chan_id))
                        .collect::<HashSet<_>>();
                    let outputs = output_chan_ids
                        .iter()
                        .flat_map(|&id| self.take_chan(id).map(|c| (id, c.data)))
                        .collect::<HashMap<_, _>>();
                    let images = proc_ids
                        .iter()
                        .flat_map(|&id| self.take_proc(id).map(|p| (id, p.into_snap())))
                        .collect::<HashMap<_, _>>();
                    return Ok(ProcsExecutionResult::new(images, outputs));
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

    pub fn new_channel(&mut self, init_input: &[i32]) -> usize {
        let chan = Channel::new(init_input);
        for (i, slot) in self.channels.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(chan);
                return i;
            }
        }

        self.channels.push(Some(chan));

        self.channels.len() - 1
    }

    pub fn new_proc(
        &mut self,
        int_code: &[i32],
        input_chan_id: usize,
        output_chan_id: usize,
    ) -> usize {
        let proc = Process::new(int_code, input_chan_id, output_chan_id);
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

        match self.chan_mut(input_chan_id) {
            Some(c) => c.reg_proc(id),
            _ => (),
        };

        id
    }

    fn take_chan(&mut self, chan_id: usize) -> Option<Channel> {
        self.channels.get_mut(chan_id).and_then(|o| o.take())
    }

    fn chan_mut(&mut self, chan_id: usize) -> Option<&mut Channel> {
        self.channels.get_mut(chan_id).and_then(|o| o.as_mut())
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

struct Channel {
    data: VecDeque<i32>,
    output_reg_proc_id: Option<usize>,
}

impl Channel {
    fn new(init_input: &[i32]) -> Self {
        Self {
            data: VecDeque::from_iter(init_input.iter().copied()),
            output_reg_proc_id: None,
        }
    }

    fn reg_proc(&mut self, proc_id: usize) {
        self.output_reg_proc_id = Some(proc_id);
    }

    fn get(&mut self) -> Option<i32> {
        self.data.pop_front()
    }

    fn put(&mut self, value: i32) {
        self.data.push_back(value);
    }

    fn reg_proc_id(&self) -> Option<usize> {
        self.output_reg_proc_id
    }
}

pub struct ProcessSnapshot {
    step_count: usize,
    state: ProcessState,
    image: Vec<i32>,
}

impl ProcessSnapshot {
    pub fn state(&self) -> ProcessState {
        self.state
    }

    pub fn step_count(&self) -> usize {
        self.step_count
    }

    pub fn image(&self) -> &[i32] {
        &self.image
    }
}

pub struct ProcsExecutionResult {
    proc_snapshots: HashMap<usize, ProcessSnapshot>,
    outputs: HashMap<usize, VecDeque<i32>>,
}

impl ProcsExecutionResult {
    fn new(
        proc_snapshots: HashMap<usize, ProcessSnapshot>,
        mut outputs: HashMap<usize, VecDeque<i32>>,
    ) -> Self {
        outputs.iter_mut().for_each(|(_, vd)| {
            vd.make_contiguous();
        });

        Self {
            proc_snapshots,
            outputs,
        }
    }

    pub fn proc_snapshots(&self, proc_id: usize) -> Option<&ProcessSnapshot> {
        self.proc_snapshots.get(&proc_id)
    }

    pub fn output(&self, chan_id: usize) -> Option<&[i32]> {
        self.outputs.get(&chan_id).map(|vd| vd.as_slices().0)
    }

    fn extract(&mut self, proc_id: usize, chan_id: usize) -> Option<ExecutionResult> {
        let proc = self.proc_snapshots.remove(&proc_id);
        let output = self.outputs.remove(&chan_id);

        if proc.is_some() && output.is_some() {
            let proc = proc.unwrap();
            let output = output.unwrap();
            Some(ExecutionResult {
                step_count: proc.step_count,
                image: proc.image,
                outputs: Vec::from_iter(output.iter().copied()),
            })
        } else {
            None
        }
    }
}

pub struct ExecutionResult {
    step_count: usize,
    image: Vec<i32>,
    outputs: Vec<i32>,
}

impl ExecutionResult {
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
    image: Vec<i32>,
    input_chan_id: usize,
    output_chan_id: usize,
    step_count: usize,
}

impl Process {
    fn new(image: &[i32], input_chan_id: usize, output_chan_id: usize) -> Self {
        Process {
            state: ProcessState::Ready,
            inst_p: 0,
            image: Vec::from(image),
            input_chan_id,
            output_chan_id,
            step_count: 0,
        }
    }

    fn inst_p(&self) -> usize {
        self.inst_p
    }

    fn cur_inst(&self) -> Result<Box<dyn Instruction>, Error> {
        if self.inst_p >= self.image.len() {
            Err(Error::ExecutionExceedIntCode(self.inst_p, self.image.len()))
        } else {
            parse_cur_inst(&self.image[self.inst_p..])
        }
    }

    fn is_halt(&self) -> bool {
        self.state == ProcessState::Halt
    }

    fn into_snap(self) -> ProcessSnapshot {
        ProcessSnapshot {
            step_count: self.step_count,
            state: self.state,
            image: self.image,
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
    fn image(&self) -> &[i32] {
        &self.run_proc().image
    }

    fn image_mut(&mut self) -> &mut [i32] {
        &mut self.run_proc_mut().image
    }

    fn input(&mut self) -> Option<i32> {
        let res = self
            .computer
            .chan_mut(self.run_proc().input_chan_id)
            .and_then(|c| c.get());
        if res.is_none() {
            assert!(self.run_proc().state == ProcessState::Running);
            self.run_proc_mut().state = ProcessState::Block;
        }

        res
    }

    fn output(&mut self, value: i32) {
        let awake_id = self
            .computer
            .chan_mut(self.run_proc().output_chan_id)
            .and_then(|c| {
                c.put(value);
                c.reg_proc_id()
            });

        match awake_id {
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
    }

    fn halt(&mut self) {
        self.run_proc_mut().state = ProcessState::Halt;
    }

    fn inst_p_mut(&mut self) -> &mut usize {
        &mut self.run_proc_mut().inst_p
    }
}
