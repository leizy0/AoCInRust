use day2_5_7_9_11_13_15_17_19_21_23::int_code::{
    com::SeqIntCodeComputer,
    io::{Channel, SeqInputDevice, SeqOutputDevice},
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

    let mut computer = SeqIntCodeComputer::new(true);
    let input_dev = SeqInputDevice::new(Channel::new(&[2]));
    let output_dev = SeqOutputDevice::new(Channel::new(&[]));
    match computer.execute_with_io(&int_code, input_dev, output_dev.clone()) {
        Ok(res) => output_dev.check(|c| {
            println!(
                "Boost program in sensor boost mode takes {} steps to finish, get outputs({:?})",
                res.step_count(),
                c.data()
            )
        }),
        Err(e) => eprintln!(
            "Failed to execute Boost program in sensor boost, get error({})",
            e
        ),
    }
}
