use day2::int_code::{execute_int_code, read_int_code};

fn main() {
    let int_code_file = "inputs.txt";
    let mut int_code = read_int_code(int_code_file)
        .expect(&format!("Failed to read int code from {}", int_code_file));
    int_code[1] = 12;
    int_code[2] = 2;
    match execute_int_code(&mut int_code) {
        Ok(step_count) => print!(
            "After {} steps, program halt, code[0] = {}",
            step_count, int_code[0]
        ),
        Err(e) => print!("Failed to run int code, get error({})", e),
    }
}
