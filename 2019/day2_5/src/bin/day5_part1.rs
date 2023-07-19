use day2_5::int_code::{read_int_code, com::IntCodeComputer};

fn main() {
    let int_code_file = "day5_inputs.txt";
    let int_code = read_int_code(int_code_file).expect(&format!("Failed to read int code from file({})", int_code_file));
    let mut computer = IntCodeComputer::new();
    let code_inputs = vec![1];
    match computer.execute(int_code, code_inputs) {
        Ok(res) => println!("After {} steps, execution finished, Outputs: {:?}", res.step_count(), res.outputs()),
        Err(e) => eprintln!("Failed to execute int code, get error({})", e),
    }
}