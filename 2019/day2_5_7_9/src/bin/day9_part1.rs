use day2_5_7_9::int_code::{com::IntCodeComputer, read_int_code};

fn main() {
    let input_path = "day9_inputs.txt";
    let int_code = match read_int_code(input_path) {
        Ok(ic) => ic,
        Err(e) => {
            eprintln!(
                "Failed to read int code from file({}), get error({})",
                input_path, e
            );
            return;
        }
    };

    let mut computer = IntCodeComputer::new(true);
    let test_mode_input = vec![1];
    match computer.execute(int_code, test_mode_input) {
        Ok(res) => println!(
            "Boost program in test mode takes {} steps to finish, get outputs({:?})",
            res.step_count(),
            res.outputs()
        ),
        Err(e) => eprintln!(
            "Failed to execute Boost program in test mode, get error({})",
            e
        ),
    }
}
