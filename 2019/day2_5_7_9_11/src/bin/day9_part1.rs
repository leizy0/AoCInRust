use day2_5_7_9_11::int_code::{
    com::{Channel, InputDevice, IntCodeComputer, OutputDevice},
    read_int_code,
};

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
    let input_dev = InputDevice::new(Channel::new(&[1]));
    let output_dev = OutputDevice::new(Channel::new(&[]));
    match computer.execute_with_io(&int_code, input_dev, output_dev.clone()) {
        Ok(res) => output_dev.check(|c| {
            println!(
                "Boost program in test mode takes {} steps to finish, get outputs({:?})",
                res.step_count(),
                c.data()
            )
        }),
        Err(e) => eprintln!(
            "Failed to execute Boost program in test mode, get error({})",
            e
        ),
    }
}
