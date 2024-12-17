use anyhow::{Context, Result};
use clap::Parser;
use day17::{CLIArgs, Computer};

fn main() -> Result<()> {
    let args = CLIArgs::parse();
    let (registers, program) = day17::read_debug_info(&args.input_path)
        .with_context(|| format!("failed to open given file({}).", args.input_path.display()))?;

    let register_a = accumulate_reg_a(&registers, &program);
    println!("The least positive value of register A that makes output from program is the program itself is {}.", register_a);

    Ok(())
}

fn accumulate_reg_a(org_registers: &[usize; 3], program: &[usize]) -> usize {
    accumulate_reg_a_recur(org_registers, program, 0)
        .iter()
        .copied()
        .min()
        .unwrap()
}

fn accumulate_reg_a_recur(
    org_registers: &[usize; 3],
    program: &[usize],
    target_ind: usize,
) -> Vec<usize> {
    let mut a_registers = Vec::new();
    let last_a_registers = if target_ind + 1 < program.len() {
        accumulate_reg_a_recur(org_registers, program, target_ind + 1)
    } else {
        vec![0]
    };
    let mut registers = org_registers.clone();
    for last_a_register in last_a_registers {
        for cur_digit in 0..8 {
            let cur_a_register = last_a_register * 8 + cur_digit;
            registers[0] = cur_a_register;
            let mut computer = Computer::new(&registers);
            if let Ok(()) = computer.run(&program) {
                if computer.output() == &program[target_ind..] {
                    a_registers.push(cur_a_register);
                }
            }
        }
    }

    a_registers
}
