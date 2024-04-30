use day2_5_7_9_11_13_15_17_19_21_23::int_code::{
    com::SeqIntCodeComputer,
    io::{Channel, SeqInputDevice, SeqOutputDevice},
    read_int_code,
};

fn main() {
    let int_code_file = "day5_inputs.txt";
    let int_code = read_int_code(int_code_file).expect(&format!(
        "Failed to read int code from file({})",
        int_code_file
    ));
    let mut computer = SeqIntCodeComputer::new(false);
    let input_dev = SeqInputDevice::new(Channel::new(&[5]));
    let output_dev = SeqOutputDevice::new(Channel::new(&[]));
    match computer.execute_with_io(&int_code, input_dev, output_dev.clone()) {
        Ok(res) => output_dev.check(|c| {
            println!(
                "After {} steps, execution finished, Outputs: {:?}",
                res.step_count(),
                c.data()
            )
        }),
        Err(e) => eprintln!("Failed to execute int code, get error({})", e),
    }
}
