use std::collections::HashSet;

use clap::Parser;
use day8::{CliArgs, GCState, GameConsole};

fn main() {
    let args = CliArgs::parse();
    let code = day8::read_code(&args.input_path)
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
    match console.run_while(&code, &mut |state: &GCState| inst_ind_set.insert(state.inst_ptr)) {
        Ok(state) => println!("Before the first repeated execution of a same instruction(at {}), the accumulator is {}.", state.inst_ptr, state.acc),
        Err(e) => eprint!("Failed to run given code, get error({}).", e),
    }
}
