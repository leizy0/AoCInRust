use day16_19::inst::{self, Executor, Error, RegisterGroup};

fn main() {
    let input_path = "day19_input.txt";
    let program = inst::load_program(input_path).expect(&format!("Failed to load program from input file({})", input_path));
    // let mut executor = Executor::new(); // Part1
    let mut executor = Executor::with_regs(&RegisterGroup::from_arr(&[1])); // Part 2
    match executor.execute(&program) {
        Err(e) => {
            match e {
                Error::InvalidInstructionPointer(ip) => {
                    println!("Program halt at instruction#{}", ip);
                    println!("Final state is {}", executor.regs());
                },
                other => println!("Failed to execute program, get error({})", other),
            }
        },
        _ => {},
    }
}