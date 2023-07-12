use day16_19::inst::{self, Executor, Error, RegisterGroup};

fn main() {
    let input_path = "day21_input.txt";
    let program = inst::load_program(input_path).expect(&format!("Failed to load program from input file({})", input_path));
    let mut executor = Executor::with_regs(&RegisterGroup::from_arr(&[0]));
    executor.set_break_at(28);
    match executor.execute(&program) {
        Err(e) => {
            match e {
                Error::InvalidInstructionPointer(ip) => {
                    println!("Program halt at instruction#{}", ip);
                    println!("Final state is {}", executor.regs());
                },
                Error::Break(ip) => {
                    println!("Program halt at instruction#{}", ip);
                    println!("Current state is {}, if set register 0 to register 1 can halt program and cost least steps", executor.regs());
                }
                other => println!("Failed to execute program, get error({})", other),
            }
        },
        _ => {},
    }
}