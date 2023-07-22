use std::fmt::Display;

use crate::int_code::com::IntCodeComputer;

pub struct AmpSettings {
    settings: Vec<Vec<i32>>,
}

impl AmpSettings {
    pub fn new(amp_count: usize) -> AmpSettings {
        let init_setting = (0..amp_count as i32).collect::<Vec<_>>();
        AmpSettings {
            settings: Self::gen_permutation(init_setting),
        }
    }

    pub fn iter(&self) -> AmpSettingsIterator {
        AmpSettingsIterator::new(&self.settings)
    }

    fn gen_permutation(mut numbers: Vec<i32>) -> Vec<Vec<i32>> {
        let mut permutations = Vec::new();
        let number_count = numbers.len();
        Self::gen_permutation_recur(&mut permutations, &mut numbers, number_count);
        permutations
    }

    // Heap's algorithm to generate permutation of slice.
    fn gen_permutation_recur(
        permutations: &mut Vec<Vec<i32>>,
        numbers: &mut [i32],
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
    settings: &'a [Vec<i32>],
    next_ind: usize,
}

impl<'a> AmpSettingsIterator<'a> {
    fn new(settings: &'a [Vec<i32>]) -> Self {
        AmpSettingsIterator {
            settings,
            next_ind: 0,
        }
    }
}

impl<'a> Iterator for AmpSettingsIterator<'a> {
    type Item = &'a [i32];

    fn next(&mut self) -> Option<Self::Item> {
        let cur_ind = self.next_ind;
        self.next_ind += 1;
        self.settings.get(cur_ind).map(|v| v.as_slice())
    }
}

#[derive(Debug)]
pub enum Error {
    EmptyAmplifierResult(Vec<i32>),
    ExecutionError(crate::Error, Vec<i32>),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::EmptyAmplifierResult(s) => {
                write!(f, "Amplifiers have empty result with settings({:?})", s)
            }
            Error::ExecutionError(e, s) => write!(
                f,
                "Error({}) in execution of amplifiers with settings({:?}",
                e, s
            ),
        }
    }
}

pub fn amp_once(
    computer: &mut IntCodeComputer,
    int_code: &[i32],
    settings: &[i32],
) -> Result<i32, Error> {
    let mut amp_res = 0;
    for i in 0..settings.len() {
        let image = Vec::from(int_code);
        let res = computer
            .execute(image, vec![settings[i], amp_res])
            .map_err(|e| Error::ExecutionError(e, Vec::from(settings)))?;
        amp_res = res
            .outputs()
            .get(0)
            .cloned()
            .ok_or(Error::EmptyAmplifierResult(Vec::from(settings)))?;
    }

    Ok(amp_res)
}
