use day2_5_7_9_11_13_15_17_19_21_23::{
    int_code::{
        com::{ProcessState, SeqIntCodeComputer},
        io::SeqIODevice,
        read_int_code,
    },
    pilot::Autopilot,
};

fn main() {
    let input_path = "day15_inputs.txt";
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

    let io_dev = SeqIODevice::new(Autopilot::new(
        Vec::new(),
        Box::new(|ap| ap.oxygen_sys_pos().is_some()),
    ));
    let mut computer = SeqIntCodeComputer::new(false);
    match computer.execute_with_io(&int_code, io_dev.input_device(), io_dev.output_device()) {
        Ok(res) => {
            println!(
                "After {} steps, remote control program stopped.",
                res.step_count()
            );
            match res.state() {
                ProcessState::Block => {
                    println!("Remote control program blocked by autopilot, found oxygen system.");
                    io_dev.check(|ap| {
                        let oxygen_sys_pos = ap.oxygen_sys_pos().expect("No oxygen system position recorded, some internal logic error happened.");
                        let moves = ap.moves_from_origin(&oxygen_sys_pos);
                        println!("It takes at least {} steps for repair droid moving to oxygen system(at {}).", moves.len(), oxygen_sys_pos);
                    });
                }
                ProcessState::Halt => eprintln!("Remote control system halt unexpectedly."),
                _ => eprintln!(
                    "Unexpected state({:?}) found after program stopped.",
                    res.state()
                ),
            }
        }
        Err(e) => eprintln!("Failed to run arcade program, get error({})", e),
    };
}
