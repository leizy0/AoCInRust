use day2_5_7_9_11_13_15_17_19_21_23_25::{
    day13::{Screen, TileId},
    int_code::{
        com::SeqIntCodeComputer,
        io::{Channel, SeqInputDevice, SeqOutputDevice},
        read_int_code,
    },
};

fn main() {
    let input_path = "day13_inputs.txt";
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

    let input_dev = SeqInputDevice::new(Channel::new(&[]));
    let output_dev = SeqOutputDevice::new(Channel::new(&[]));
    let mut computer = SeqIntCodeComputer::new(true);
    match computer.execute_with_io(&int_code, input_dev, output_dev.clone()) {
        Ok(res) => output_dev.check(|c| {
            println!(
                "After {} steps, arcade program halt, get outputs({:?})",
                res.step_count(),
                c.data(),
            )
        }),
        Err(e) => eprintln!("Failed to run arcade program, get error({})", e),
    };

    output_dev.check(|c| {
        let screen = Screen::from_ints(c.data().iter().copied());
        match screen {
            Ok(s) => println!(
                "There are {} blocks in the last screen.",
                s.count_id(TileId::Block)
            ),
            Err(e) => eprintln!("Invalid output for render screen, get error({}).", e),
        }
    });
}
