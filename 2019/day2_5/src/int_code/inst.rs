use std::collections::HashMap;

use int_enum::IntEnum;
use once_cell::sync::Lazy;

use crate::Error;

use super::com::ExecutionState;

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy, IntEnum, PartialEq, Eq)]
pub enum ParameterMode {
    #[default]
    Position = 0,
    Immediate = 1,
}


pub trait Instruction {
    fn opcode_ind(&self) -> u32;
    fn length(&self) -> usize;
    fn params(&self) -> &[i32];
    fn params_mut(&mut self) -> &mut [i32];
    fn param_modes(&self) -> &[ParameterMode];
    fn param_modes_mut(&mut self) -> &mut [ParameterMode];
    fn execute(&self, exe_state: &mut dyn ExecutionState) -> Result<(), Error>;
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntEnum, Hash)]
pub enum InstOpcodeInd {
    Add = 1,
    Multiply = 2,
    Input = 3,
    Output = 4,
    Halt = 99,
}


type ParseFunc = fn(&[i32]) -> Result<Box<dyn Instruction>, Error>;
static INST_PARSE_MAP: Lazy<HashMap<InstOpcodeInd, ParseFunc>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert(InstOpcodeInd::Add, parse_inst::<Add> as ParseFunc);
    map.insert(InstOpcodeInd::Multiply, parse_inst::<Multiply> as ParseFunc);
    map.insert(InstOpcodeInd::Input, parse_inst::<Input> as ParseFunc);
    map.insert(InstOpcodeInd::Output, parse_inst::<Output> as ParseFunc);
    map.insert(InstOpcodeInd::Halt, parse_inst::<Halt> as ParseFunc);

    map
});

pub fn parse_cur_inst(code: &[i32]) -> Result<Box<dyn Instruction>, Error> {
    let cur_opcode_ind = parse_opcode_ind(code[0]).and_then(|n| InstOpcodeInd::from_int(n).map_err(|_| Error::InvalidOpcodeIndex(n)))?;
    let parse_func = INST_PARSE_MAP.get(&cur_opcode_ind).ok_or(Error::InvalidOpcodeIndex(cur_opcode_ind.int_value()))?;
    parse_func(code)
}

fn parse_inst<I>(code: &[i32]) -> Result<Box<dyn Instruction>, Error>
    where I: Instruction + Default + 'static,
{
    let mut inst = I::default();
    if code.len() < inst.length() {
        return Err(Error::MissingCodeForInstruction(inst.opcode_ind()));
    }

    parse_opcode(code[0], inst.opcode_ind(), inst.param_modes_mut())?;
    for i in 0..inst.params().len() {
        // Skip operation code
        inst.params_mut()[i] = code[i + 1];
    }

    Ok(Box::new(inst))
}

fn parse_opcode(opcode: i32, expect_opcode_ind: u32, param_modes: &mut [ParameterMode]) -> Result<(), Error> {
    if !parse_opcode_ind(opcode).is_ok_and(|ind| ind == expect_opcode_ind) {
        return Err(Error::OpcodeNotMatchingForInstruction(opcode.to_string(), expect_opcode_ind));
    }

    
    let mut cur_ratio = 100;
    for i in 0..param_modes.len() {
        let cur_digit = u32::try_from(opcode).unwrap() / cur_ratio % 10;
        param_modes[i] = ParameterMode::from_int(cur_digit as u8).map_err(|_| Error::UnknownParameterMode(cur_digit))?;

        cur_ratio *= 10;
    }

    Ok(())
}

fn parse_opcode_ind(opcode: i32) -> Result<u32, Error> {
    if opcode < 0 {
        Err(Error::InvalidOpcode(opcode))
    } else {
        Ok(u32::try_from(opcode).unwrap() % 100)
    }
}



fn read_image(image: &[i32], param: i32, param_mode: ParameterMode) -> Result<i32, Error> {
    match param_mode {
        ParameterMode::Position => if param < 0 || param >= image.len() as i32 {
            Err(Error::ImageIndexError(param))
        } else {
            Ok(image[param as usize])
        },
        ParameterMode::Immediate => Ok(param),
    }
}

fn write_image(image: &mut [i32], param: i32, param_mode: ParameterMode, value: i32) -> Result<(), Error> {
    if param_mode != ParameterMode::Position {
        return Err(Error::InvalidWriteImageMode(param_mode.int_value()))
    }

    if param < 0 || param >= image.len() as i32 {
        Err(Error::ImageIndexError(param))
    } else {
        image[param as usize] = value;
        Ok(())
    }
}


#[derive(Debug, Default)]
pub struct Add {
    params: [i32; 3],
    param_modes: [ParameterMode; 3],
}

impl Instruction for Add {
    #[inline]
    fn length(&self) -> usize {
        4
    }

    #[inline]
    fn opcode_ind(&self) -> u32 {
        InstOpcodeInd::Add.int_value()
    }

    fn params(&self) -> &[i32] {
        &self.params
    }

    fn params_mut(&mut self) -> &mut [i32] {
        &mut self.params
    }

    fn param_modes(&self) -> &[ParameterMode] {
        &self.param_modes
    }

    fn param_modes_mut(&mut self) -> &mut [ParameterMode] {
        &mut self.param_modes
    }

    fn execute(&self, exe_state: &mut dyn ExecutionState) -> Result<(), Error> {
        let input0 = read_image(exe_state.image(), self.params[0], self.param_modes[0])?;
        let input1 = read_image(exe_state.image(), self.params[1], self.param_modes[1])?;
        write_image(exe_state.image_mut(), self.params[2], self.param_modes[2], input0 + input1)?;
        *exe_state.inst_p_mut() += self.length();

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct Multiply {
    params: [i32; 3],
    param_modes: [ParameterMode; 3],
}

impl Instruction for Multiply {
    #[inline]
    fn length(&self) -> usize {
        4
    }

    #[inline]
    fn opcode_ind(&self) -> u32 {
        InstOpcodeInd::Multiply.int_value()
    }

    fn params(&self) -> &[i32] {
        &self.params
    }

    fn params_mut(&mut self) -> &mut [i32] {
        &mut self.params
    }

    fn param_modes(&self) -> &[ParameterMode] {
        &self.param_modes
    }

    fn param_modes_mut(&mut self) -> &mut [ParameterMode] {
        &mut self.param_modes
    }

    fn execute(&self, exe_state: &mut dyn ExecutionState) -> Result<(), Error> {
        let input0 = read_image(exe_state.image(), self.params[0], self.param_modes[0])?;
        let input1 = read_image(exe_state.image(), self.params[1], self.param_modes[1])?;
        write_image(exe_state.image_mut(), self.params[2], self.param_modes[2], input0 * input1)?;
        *exe_state.inst_p_mut() += self.length();

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct Halt{
    params: [i32; 0],
    param_modes: [ParameterMode; 0],
}

impl Instruction for Halt {
    #[inline]
    fn length(&self) -> usize {
        1
    }

    #[inline]
    fn opcode_ind(&self) -> u32 {
        InstOpcodeInd::Halt.int_value()
    }

    fn params(&self) -> &[i32] {
        &self.params
    }

    fn params_mut(&mut self) -> &mut [i32] {
        &mut self.params
    }

    fn param_modes(&self) -> &[ParameterMode] {
        &self.param_modes
    }

    fn param_modes_mut(&mut self) -> &mut [ParameterMode] {
        &mut self.param_modes
    }

    fn execute(&self, exe_state: &mut dyn ExecutionState) -> Result<(), Error> {
        exe_state.halt();
        *exe_state.inst_p_mut() += self.length();

        Ok(())
    }
}


#[derive(Debug, Default)]
pub struct Input{
    params: [i32; 1],
    param_modes: [ParameterMode; 1],
}

impl Instruction for Input {
    #[inline]
    fn length(&self) -> usize {
        2
    }

    #[inline]
    fn opcode_ind(&self) -> u32 {
        InstOpcodeInd::Input.int_value()
    }

    fn params(&self) -> &[i32] {
        &self.params
    }

    fn params_mut(&mut self) -> &mut [i32] {
        &mut self.params
    }

    fn param_modes(&self) -> &[ParameterMode] {
        &self.param_modes
    }

    fn param_modes_mut(&mut self) -> &mut [ParameterMode] {
        &mut self.param_modes
    }

    fn execute(&self, exe_state: &mut dyn ExecutionState) -> Result<(), Error> {
        let input = exe_state.input().ok_or(Error::NotEnoughInput)?;
        write_image(exe_state.image_mut(), self.params[0], self.param_modes[0], input)?;
        *exe_state.inst_p_mut() += self.length();

        Ok(())
    }
}


#[derive(Debug, Default)]
pub struct Output{
    params: [i32; 1],
    param_modes: [ParameterMode; 1],
}

impl Instruction for Output {
    #[inline]
    fn length(&self) -> usize {
        2
    }

    #[inline]
    fn opcode_ind(&self) -> u32 {
        InstOpcodeInd::Output.int_value()
    }

    fn params(&self) -> &[i32] {
        &self.params
    }

    fn params_mut(&mut self) -> &mut [i32] {
        &mut self.params
    }

    fn param_modes(&self) -> &[ParameterMode] {
        &self.param_modes
    }

    fn param_modes_mut(&mut self) -> &mut [ParameterMode] {
        &mut self.param_modes
    }

    fn execute(&self, exe_state: &mut dyn ExecutionState) -> Result<(), Error> {
        exe_state.output(read_image(exe_state.image(), self.params[0], self.param_modes[0])?);
        *exe_state.inst_p_mut() += self.length();

        Ok(())
    }
}
