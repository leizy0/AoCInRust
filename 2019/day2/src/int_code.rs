use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use crate::Error;

pub fn read_int_code<P>(path: P) -> Result<Vec<u32>, Error>
where
    P: AsRef<Path>,
{
    let code_file = File::open(path).map_err(Error::IOError)?;
    let reader = BufReader::new(code_file);

    reader.lines().next().map_or(Err(Error::EmptyError), |res| {
        res.map_err(Error::IOError).and_then(|s| {
            s.split(',')
                .map(|s| {
                    str::parse::<u32>(s.trim()).map_err(|_| Error::ParseIntError(s.to_string()))
                })
                .collect::<Result<Vec<_>, Error>>()
        })
    })
}

pub fn execute_int_code(code: &mut Vec<u32>) -> Result<usize, Error> {
    let mut inst_p = 0;
    let mut step_count = 0usize;
    loop {
        let op_code = read_code(code, inst_p)?;
        let input0_p = read_code(code, inst_p + 1)?;
        let input1_p = read_code(code, inst_p + 2)?;
        let output_p = read_code(code, inst_p + 3)?;
        inst_p += 4;

        let op_res = match op_code {
            1 => Ok(read_code(code, input0_p as usize)? + read_code(code, input1_p as usize)?),
            2 => Ok(read_code(code, input0_p as usize)? * read_code(code, input1_p as usize)?),
            99 => break,
            code => Err(Error::InvalidOpCode(code)),
        }?;
        write_code(code, output_p as usize, op_res)?;
        step_count += 1;
    }

    Ok(step_count)
}

fn read_code(code: &Vec<u32>, index: usize) -> Result<u32, Error> {
    code.get(index).ok_or(Error::IndexError(index)).map(|n| *n)
}

fn write_code(code: &mut Vec<u32>, index: usize, value: u32) -> Result<(), Error> {
    code.get_mut(index)
        .ok_or(Error::IndexError(index))
        .map(|n| *n = value)
}

#[test]
fn test_execute_int_code0() {
    let mut int_code = vec![1, 9, 10, 3, 2, 3, 11, 0, 99, 30, 40, 50];
    let _step_count = execute_int_code(&mut int_code).unwrap();
    assert_eq!(int_code[0], 3500);
}
