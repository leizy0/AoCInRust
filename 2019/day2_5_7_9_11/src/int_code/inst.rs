use std::{collections::HashMap, fmt::Debug};

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
    Relative = 2,
}

pub trait Instruction: Debug {
    fn opcode_ind(&self) -> u32;
    fn length(&self) -> usize;
    fn params(&self) -> &[i64];
    fn params_mut(&mut self) -> &mut [i64];
    fn param_modes(&self) -> &[ParameterMode];
    fn param_modes_mut(&mut self) -> &mut [ParameterMode];
    fn execute(&self, exe_state: &mut dyn ExecutionState) -> Result<(), Error>;

    fn forward_inst_p(&self, exe_state: &mut dyn ExecutionState) {
        *exe_state.inst_p_mut() += self.length();
    }

    fn read_mem(
        exe_state: &mut dyn ExecutionState,
        param: i64,
        param_mode: ParameterMode,
    ) -> Result<i64, Error>
    where
        Self: Sized,
    {
        match param_mode {
            ParameterMode::Position | ParameterMode::Relative => {
                let pos = if param_mode == ParameterMode::Relative {
                    exe_state.rel_base() + param
                } else {
                    param
                };

                if pos < 0 {
                    Err(Error::ImageIndexError(pos))
                } else {
                    Ok(exe_state.read_mem(pos as usize))
                }
            }
            ParameterMode::Immediate => Ok(param),
        }
    }

    fn write_mem(
        exe_state: &mut dyn ExecutionState,
        param: i64,
        param_mode: ParameterMode,
        value: i64,
    ) -> Result<(), Error>
    where
        Self: Sized,
    {
        match param_mode {
            ParameterMode::Position | ParameterMode::Relative => {
                let pos = if param_mode == ParameterMode::Relative {
                    exe_state.rel_base() + param
                } else {
                    param
                };

                if pos < 0 {
                    Err(Error::ImageIndexError(param))
                } else {
                    Ok(exe_state.write_mem(pos as usize, value))
                }
            }
            ParameterMode::Immediate => Err(Error::InvalidWriteMemoryMode(param_mode.int_value())),
        }
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntEnum, Hash)]
pub enum InstOpcodeInd {
    Add = 1,
    Multiply = 2,
    Input = 3,
    Output = 4,
    JumpIfTrue = 5,
    JumpIfFalse = 6,
    LessThan = 7,
    Equals = 8,
    AdjustRelativeBase = 9,
    Halt = 99,
}

type ParseFunc = fn(&[i64]) -> Result<Box<dyn Instruction>, Error>;
static INST_PARSE_MAP: Lazy<HashMap<InstOpcodeInd, ParseFunc>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert(InstOpcodeInd::Add, parse_inst::<Add> as ParseFunc);
    map.insert(InstOpcodeInd::Multiply, parse_inst::<Multiply> as ParseFunc);
    map.insert(InstOpcodeInd::Input, parse_inst::<Input> as ParseFunc);
    map.insert(InstOpcodeInd::Output, parse_inst::<Output> as ParseFunc);
    map.insert(
        InstOpcodeInd::JumpIfTrue,
        parse_inst::<JumpIfTrue> as ParseFunc,
    );
    map.insert(
        InstOpcodeInd::JumpIfFalse,
        parse_inst::<JumpIfFalse> as ParseFunc,
    );
    map.insert(InstOpcodeInd::LessThan, parse_inst::<LessThan> as ParseFunc);
    map.insert(InstOpcodeInd::Equals, parse_inst::<Equals> as ParseFunc);
    map.insert(
        InstOpcodeInd::AdjustRelativeBase,
        parse_inst::<AdjustRelativeBase> as ParseFunc,
    );
    map.insert(InstOpcodeInd::Halt, parse_inst::<Halt> as ParseFunc);

    map
});

pub fn parse_cur_inst(code: &[i64]) -> Result<Box<dyn Instruction>, Error> {
    let cur_opcode_ind = parse_opcode_ind(code[0])
        .and_then(|n| InstOpcodeInd::from_int(n).map_err(|_| Error::InvalidOpcodeIndex(n)))?;
    let parse_func = INST_PARSE_MAP
        .get(&cur_opcode_ind)
        .ok_or(Error::InvalidOpcodeIndex(cur_opcode_ind.int_value()))?;
    parse_func(code)
}

fn parse_inst<I>(code: &[i64]) -> Result<Box<dyn Instruction>, Error>
where
    I: Instruction + Default + 'static,
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

fn parse_opcode(
    opcode: i64,
    expect_opcode_ind: u32,
    param_modes: &mut [ParameterMode],
) -> Result<(), Error> {
    if !parse_opcode_ind(opcode).is_ok_and(|ind| ind == expect_opcode_ind) {
        return Err(Error::OpcodeNotMatchingForInstruction(
            opcode.to_string(),
            expect_opcode_ind,
        ));
    }

    let mut cur_ratio = 100;
    for i in 0..param_modes.len() {
        let cur_digit = u32::try_from(opcode).unwrap() / cur_ratio % 10;
        param_modes[i] = ParameterMode::from_int(cur_digit as u8)
            .map_err(|_| Error::UnknownParameterMode(cur_digit))?;

        cur_ratio *= 10;
    }

    Ok(())
}

fn parse_opcode_ind(opcode: i64) -> Result<u32, Error> {
    if opcode < 0 {
        Err(Error::InvalidOpcode(opcode))
    } else {
        Ok(u32::try_from(opcode).unwrap() % 100)
    }
}

macro_rules! def_instruction {
    (name=$name:ident, length=$length:literal, index=$opcode_ind:expr; execute($inst_var:ident, $exe_var:ident) => $exe_block:block) => {
        #[derive(Debug, Default)]
        pub struct $name {
            params: [i64; $length - 1],
            param_modes: [ParameterMode; $length - 1],
        }

        impl Instruction for $name {
            #[inline]
            fn length(&self) -> usize {
                $length
            }

            #[inline]
            fn opcode_ind(&self) -> u32 {
                $opcode_ind
            }

            fn params(&self) -> &[i64] {
                &self.params
            }

            fn params_mut(&mut self) -> &mut [i64] {
                &mut self.params
            }

            fn param_modes(&self) -> &[ParameterMode] {
                &self.param_modes
            }

            fn param_modes_mut(&mut self) -> &mut [ParameterMode] {
                &mut self.param_modes
            }

            fn execute(&self, exe_state: &mut dyn ExecutionState) -> Result<(), Error> {
                let $inst_var = self;
                let $exe_var = exe_state;

                $exe_block
            }
        }
    };
}

def_instruction!(name=Add, length=4, index=InstOpcodeInd::Add.int_value(); execute(inst, exe_state) => {
    let input0 = Self::read_mem(exe_state, inst.params[0], inst.param_modes[0])?;
    let input1 = Self::read_mem(exe_state, inst.params[1], inst.param_modes[1])?;
    Self::write_mem(
        exe_state,
        inst.params[2],
        inst.param_modes[2],
        input0 + input1,
    )?;
    inst.forward_inst_p(exe_state);

    Ok(())
});

def_instruction!(name=Multiply, length=4, index=InstOpcodeInd::Multiply.int_value(); execute(inst, exe_state) => {
    let input0 = Self::read_mem(exe_state, inst.params[0], inst.param_modes[0])?;
    let input1 = Self::read_mem(exe_state, inst.params[1], inst.param_modes[1])?;
    Self::write_mem(
        exe_state,
        inst.params[2],
        inst.param_modes[2],
        input0 * input1,
    )?;
    inst.forward_inst_p(exe_state);

    Ok(())
});

def_instruction!(name=Halt, length=1, index=InstOpcodeInd::Halt.int_value(); execute(inst, exe_state) => {
    exe_state.halt();
    inst.forward_inst_p(exe_state);

    Ok(())
});

def_instruction!(name=Input, length=2, index=InstOpcodeInd::Input.int_value(); execute(inst, exe_state) => {
    let input = exe_state.input().ok_or(Error::NotEnoughInput)?;
    Self::write_mem(exe_state, inst.params[0], inst.param_modes[0], input)?;
    inst.forward_inst_p(exe_state);

    Ok(())
});

def_instruction!(name=Output, length=2, index=InstOpcodeInd::Output.int_value(); execute(inst, exe_state) => {
    let value = Self::read_mem(exe_state, inst.params[0], inst.param_modes[0])?;
    exe_state.output(value)?;
    inst.forward_inst_p(exe_state);

    Ok(())
});

def_instruction!(name=JumpIfTrue, length=3, index=InstOpcodeInd::JumpIfTrue.int_value(); execute(inst, exe_state) => {
    let condition = Self::read_mem(exe_state, inst.params[0], inst.param_modes[0])?;
    if condition != 0 {
        let target = Self::read_mem(exe_state, inst.params[1], inst.param_modes[1])?;
        *exe_state.inst_p_mut() =
            usize::try_from(target).map_err(|_| Error::InvalidJumpTarget(target))?;
    } else {
        inst.forward_inst_p(exe_state);
    }

    Ok(())
});

def_instruction!(name=JumpIfFalse, length=3, index=InstOpcodeInd::JumpIfFalse.int_value(); execute(inst, exe_state) => {
    let condition = Self::read_mem(exe_state, inst.params[0], inst.param_modes[0])?;
    if condition == 0 {
        let target = Self::read_mem(exe_state, inst.params[1], inst.param_modes[1])?;
        *exe_state.inst_p_mut() =
            usize::try_from(target).map_err(|_| Error::InvalidJumpTarget(target))?;
    } else {
        inst.forward_inst_p(exe_state);
    }

    Ok(())
});

def_instruction!(name=LessThan, length=4, index=InstOpcodeInd::LessThan.int_value(); execute(inst, exe_state) => {
    let input0 = Self::read_mem(exe_state, inst.params[0], inst.param_modes[0])?;
    let input1 = Self::read_mem(exe_state, inst.params[1], inst.param_modes[1])?;
    Self::write_mem(
        exe_state,
        inst.params[2],
        inst.param_modes[2],
        if input0 < input1 { 1 } else { 0 },
    )?;
    inst.forward_inst_p(exe_state);

    Ok(())
});

def_instruction!(name=Equals, length=4, index=InstOpcodeInd::Equals.int_value(); execute(inst, exe_state) => {
    let input0 = Self::read_mem(exe_state, inst.params[0], inst.param_modes[0])?;
    let input1 = Self::read_mem(exe_state, inst.params[1], inst.param_modes[1])?;
    Self::write_mem(
        exe_state,
        inst.params[2],
        inst.param_modes[2],
        if input0 == input1 { 1 } else { 0 },
    )?;
    inst.forward_inst_p(exe_state);

    Ok(())
});

def_instruction!(name=AdjustRelativeBase, length=2, index=InstOpcodeInd::AdjustRelativeBase.int_value(); execute(inst, exe_state) => {
    let offset = Self::read_mem(exe_state, inst.params[0], inst.param_modes[0])?;
    *exe_state.rel_base_mut() += offset;
    inst.forward_inst_p(exe_state);

    Ok(())
});
