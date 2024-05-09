use day2_5_7_9_11_13_15_17_19_21_23_25::{
    day11::{Color, PaintRobot, PaintSimulator},
    int_code::{com::SeqIntCodeComputer, io::SeqIODevice, read_int_code},
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

    let mut robot = PaintRobot::new();
    robot.paint(Color::White);
    let io_dev = SeqIODevice::new(PaintSimulator::new(robot));
    let mut computer = SeqIntCodeComputer::new(true);
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

    println!("After painting, robot get image:");
    io_dev.check(|ps| println!("{}", ps.robot().image()));
}
