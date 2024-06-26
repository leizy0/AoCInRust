use day2_5_7_9_11_13_15_17_19_21_23_25::int_code::{
    com::SeqIntCodeComputer,
    io::{Channel, SeqInputDevice, SeqOutputDevice},
    read_int_code,
};

fn main() {
    let int_code_file = "day2_inputs.txt";
    let mut int_code = read_int_code(int_code_file)
        .expect(&format!("Failed to read int code from {}", int_code_file));
    int_code[1] = 12;
    int_code[2] = 2;
    let mut computer = SeqIntCodeComputer::new(false);
    let input_dev = SeqInputDevice::new(Channel::new(&[]));
    let output_dev = SeqOutputDevice::new(Channel::new(&[]));
    match computer.execute_with_io(&int_code, input_dev, output_dev) {
        Ok(res) => println!(
            "After {} steps, program halt, code[0] = {}",
            res.step_count(),
            res.image()[0]
        ),
        Err(e) => eprintln!("Failed to run int code, get error({})", e),
    }
}
