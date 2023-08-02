use day2_5_7_9_11::{
    int_code::{com::IntCodeComputer, read_int_code},
    paint::{sim_paint, PaintRobot},
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
    let mut computer = IntCodeComputer::new(true);
    match computer.execute_with(&int_code, |input, output| {
        sim_paint(&mut robot, input, output)
    }) {
        Ok(res) => println!(
            "After {} steps, painting program halt, get outputs({:?})",
            res.step_count(),
            res.outputs()
        ),
        Err(e) => eprintln!("Failed to run painting program, get error({})", e),
    };

    println!(
        "In whole painting process, robot has painted {} times and {} blocks",
        robot.paint_count(),
        robot.block_count()
    );
}
