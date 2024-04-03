use day2_5_7_9_11_13_15_17::{
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

    // Find oxygen system first.
    let io_dev = SeqIODevice::new(Autopilot::new(
        Vec::new(),
        Box::new(|ap| ap.oxygen_sys_pos().is_some()),
    ));
    let mut computer = SeqIntCodeComputer::new(false);
    let moves_to_oxygen_sys =
        match computer.execute_with_io(&int_code, io_dev.input_device(), io_dev.output_device()) {
            Ok(res) => {
                println!(
                    "After {} steps, remote control program stopped.",
                    res.step_count()
                );
                match res.state() {
                    ProcessState::Block => {
                        println!(
                            "Remote control program blocked by autopilot, found oxygen system."
                        );
                        Some(io_dev.check(|ap| ap.moves_from_origin(&ap.oxygen_sys_pos().unwrap())))
                    }
                    ProcessState::Halt => {
                        eprintln!("Remote control system halt unexpectedly.");
                        None
                    }
                    _ => {
                        eprintln!(
                            "Unexpected state({:?}) found after program stopped.",
                            res.state()
                        );
                        None
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to run arcade program, get error({})", e);
                None
            }
        }
        .unwrap();

    // Run another time, start from position of oxygen system, iterates over the whole room.
    let io_dev = SeqIODevice::new(Autopilot::new(moves_to_oxygen_sys, Box::new(|_| false)));
    match computer.execute_with_io(&int_code, io_dev.input_device(), io_dev.output_device()) {
        Ok(res) => {
            println!(
                "After {} steps, remote control program stopped.",
                res.step_count()
            );
            match res.state() {
                ProcessState::Block => {
                    println!(
                        "Remote control program blocked by autopilot, iterated the whole room."
                    );
                    io_dev.check(|ap| {
                        let final_pos = ap.cur_pos();
                        let steps_from_oxygen_sys = ap.moves_from_origin(&final_pos).len();
                        println!("Final position searched is {}, {} steps from oxygen system, so it takes {} minutes for oxygen spreading out the whole room.", final_pos, steps_from_oxygen_sys, steps_from_oxygen_sys);
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
    }
}
