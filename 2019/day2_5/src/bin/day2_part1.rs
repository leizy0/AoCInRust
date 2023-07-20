use day2_5::int_code::{com::IntCodeComputer, read_int_code};

fn main() {
    let int_code_file = "day2_inputs.txt";
    let mut int_code = read_int_code(int_code_file)
        .expect(&format!("Failed to read int code from {}", int_code_file));
    int_code[1] = 12;
    int_code[2] = 2;
    let mut computer = IntCodeComputer::new();
    match computer.execute(int_code, Vec::new()) {
        Ok(res) => println!(
            "After {} steps, program halt, code[0] = {}",
            res.step_count(),
            res.image()[0]
        ),
        Err(e) => eprintln!("Failed to run int code, get error({})", e),
    }
}
