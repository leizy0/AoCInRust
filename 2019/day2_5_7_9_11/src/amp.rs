use std::{cell::RefCell, fmt::Display, rc::Rc};

use crate::int_code::com::{Channel, IntCodeComputer, ProcessState};

pub struct AmpSettings {
    settings: Vec<Vec<i64>>,
}

impl From<&[i64]> for AmpSettings {
    fn from(init_setting: &[i64]) -> Self {
        AmpSettings {
            settings: Self::gen_permutation(init_setting),
        }
    }
}

impl AmpSettings {
    pub fn new(amp_count: usize) -> AmpSettings {
        let init_setting = (0..amp_count as i64).collect::<Vec<_>>();
        Self::from(init_setting.as_slice())
    }

    pub fn iter(&self) -> AmpSettingsIterator {
        AmpSettingsIterator::new(&self.settings)
    }

    fn gen_permutation(init_numbers: &[i64]) -> Vec<Vec<i64>> {
        let mut permutations = Vec::new();
        let mut numbers = Vec::from(init_numbers);
        let number_count = numbers.len();
        Self::gen_permutation_recur(&mut permutations, &mut numbers, number_count);
        permutations
    }

    // Heap's algorithm to generate permutation of slice.
    fn gen_permutation_recur(
        permutations: &mut Vec<Vec<i64>>,
        numbers: &mut [i64],
        cur_size: usize,
    ) {
        if cur_size <= 1 {
            permutations.push(Vec::from(numbers));
            return;
        }

        for i in 0..cur_size {
            Self::gen_permutation_recur(permutations, numbers, cur_size - 1);

            if cur_size % 2 == 1 {
                numbers.swap(0, cur_size - 1);
            } else {
                numbers.swap(i, cur_size - 1);
            }
        }
    }
}

pub struct AmpSettingsIterator<'a> {
    settings: &'a [Vec<i64>],
    next_ind: usize,
}

impl<'a> AmpSettingsIterator<'a> {
    fn new(settings: &'a [Vec<i64>]) -> Self {
        AmpSettingsIterator {
            settings,
            next_ind: 0,
        }
    }
}

impl<'a> Iterator for AmpSettingsIterator<'a> {
    type Item = &'a [i64];

    fn next(&mut self) -> Option<Self::Item> {
        let cur_ind = self.next_ind;
        self.next_ind += 1;
        self.settings.get(cur_ind).map(|v| v.as_slice())
    }
}

#[derive(Debug)]
pub enum Error {
    ProcessBlockInChain(usize),
    EmptyAmplifierResult(Vec<i64>),
    ExecutionError(crate::Error, Vec<i64>),
    ProcessesExecutionError(crate::Error),
    AmplifierInLoopStuck,
    EmptyOutputFromAmplifierLoop,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ProcessBlockInChain(ind) => {
                write!(f, "Process(Amplifier #{}) blocked in amplifier chain", ind)
            }
            Error::EmptyAmplifierResult(s) => {
                write!(f, "Amplifiers have empty result with settings({:?})", s)
            }
            Error::ExecutionError(e, s) => write!(
                f,
                "Error({}) in execution of amplifiers with settings({:?}",
                e, s
            ),
            Error::ProcessesExecutionError(e) => {
                write!(f, "Error({}) in execution of amplifier loop", e)
            }
            Error::AmplifierInLoopStuck => {
                write!(f, "There has some processes blocked in amplifier loop")
            }
            Error::EmptyOutputFromAmplifierLoop => write!(
                f,
                "Got empty output from amplifier loop after all are halted"
            ),
        }
    }
}

pub fn amp_chain(
    computer: &mut IntCodeComputer,
    int_code: &[i64],
    settings: &[i64],
) -> Result<i64, Error> {
    let mut amp_res = 0;
    for i in 0..settings.len() {
        let image = Vec::from(int_code);
        let input_chan = Rc::new(RefCell::new(Channel::new(&[settings[i], amp_res])));
        let output_chan = Rc::new(RefCell::new(Channel::new(&[])));
        let res = computer
            .execute_with_io(&image, input_chan.clone(), output_chan.clone())
            .map_err(|e| Error::ExecutionError(e, Vec::from(settings)))?;

        if res.state() != ProcessState::Halt {
            return Err(Error::ProcessBlockInChain(i));
        }

        amp_res = output_chan
            .borrow()
            .data()
            .get(0)
            .cloned()
            .ok_or(Error::EmptyAmplifierResult(Vec::from(settings)))?;
    }

    Ok(amp_res)
}

pub fn amp_loop(
    computer: &mut IntCodeComputer,
    int_code: &[i64],
    setting: &[i64],
) -> Result<i64, Error> {
    let amp_count = setting.len();
    let amp_channels = (0..amp_count)
        .map(|i| {
            if i == 0 {
                Rc::new(RefCell::new(Channel::new(&[setting[0], 0])))
            } else {
                Rc::new(RefCell::new(Channel::new(&setting[i..(i + 1)])))
            }
        })
        .collect::<Vec<_>>();
    let mut amp_procs = (0..amp_count)
        .map(|i| {
            computer.new_proc(
                int_code,
                amp_channels[i].clone(),
                amp_channels[(i + 1) % amp_count].clone(),
            )
        })
        .collect::<Vec<_>>();

    computer
        .exe_procs(&mut amp_procs, 0)
        .map_err(Error::ProcessesExecutionError)
        .and_then(|res| {
            if amp_procs
                .iter()
                .map(|&id| res.proc_snapshots(id))
                .any(|op| op.is_none() || op.is_some_and(|p| p.state() != ProcessState::Halt))
            {
                Err(Error::AmplifierInLoopStuck)
            } else {
                amp_channels[0]
                    .borrow()
                    .data()
                    .get(0)
                    .copied()
                    .ok_or(Error::EmptyOutputFromAmplifierLoop)
            }
        })
}
