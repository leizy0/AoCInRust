use std::collections::HashSet;

use clap::Parser;
use day8::{CliArgs, GCState, GameConsole, Instruction};

fn main() {
    let args = CliArgs::parse();
    let mut code = day8::read_code(&args.input_path)
        .inspect_err(|e| {
            eprintln!(
                "Failed to read code from given input file({}), get error({}).",
                args.input_path.display(),
                e
            )
        })
        .unwrap();
    let mut console = GameConsole::new();
    let mut inst_ind_set = HashSet::new();
    let expect_inst_ptr = code.len();
    for ind in 0..(code.len()) {
        match code[ind] {
            Instruction::Acc(_) => continue,
            inst => {
                // Exchange instruction.
                code[ind] = if let Instruction::Jmp(n) = inst {
                    Instruction::Nop(n)
                } else if let Instruction::Nop(n) = inst {
                    Instruction::Jmp(n)
                } else {
                    unreachable!("No other instruction exists.");
                };

                // Test exchanged code.
                inst_ind_set.clear();
                match console.run_while(&code, &mut |state: &GCState| inst_ind_set.insert(state.inst_ptr) && state.inst_ptr != expect_inst_ptr) {
                    Ok(state) if state.inst_ptr == expect_inst_ptr => {
                        println!("Exchanged {:?}(at {}) to {:?}, code run through to end without loop, and the final accumulator value is {}.", code[ind], ind, inst, state.acc);
                        return;
                    },
                    Err(e) => eprintln!("Exchanged {:?}(at {}) to {:?}, failed to run exchanged code, get error({})", code[ind], ind, inst, e),
                    _ => (),
                }

                // Restore code.
                code[ind] = inst;
            }
        }
    }

    println!("There's no exchange can make code run through to end without loop.");
}
