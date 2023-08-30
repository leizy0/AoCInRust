use day2_5_7_9_11::{
    int_code::{
        com::{IODevice, IntCodeComputer},
        read_int_code,
    },
    paint::{PaintRobot, PaintSimulator},
};

fn main() {
    let input_path = "day11_inputs.txt";
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

    let io_dev = IODevice::new(PaintSimulator::new(PaintRobot::new()));
    let mut computer = IntCodeComputer::new(true);
    match computer.execute_with_io(&int_code, io_dev.input_device(), io_dev.output_device()) {
        Ok(res) => io_dev.check(|ps| {
            println!(
                "After {} steps, painting program halt, get outputs({:?})",
                res.step_count(),
                ps.outputs()
            )
        }),
        Err(e) => eprintln!("Failed to run painting program, get error({})", e),
    };

    io_dev.check(|ps| {
        println!(
            "In whole painting process, robot has painted {} times and {} blocks",
            ps.robot().paint_count(),
            ps.robot().block_count()
        );
    });
}
