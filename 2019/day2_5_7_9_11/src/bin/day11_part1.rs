use std::{cell::RefCell, rc::Rc};

use day2_5_7_9_11::{
    int_code::{com::IntCodeComputer, read_int_code},
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

    let simulator = Rc::new(RefCell::new(PaintSimulator::new(PaintRobot::new())));
    let mut computer = IntCodeComputer::new(true);
    match computer.execute_with_io(&int_code, simulator.clone(), simulator.clone()) {
        Ok(res) => println!(
            "After {} steps, painting program halt, get outputs({:?})",
            res.step_count(),
            simulator.borrow().outputs()
        ),
        Err(e) => eprintln!("Failed to run painting program, get error({})", e),
    };

    println!(
        "In whole painting process, robot has painted {} times and {} blocks",
        simulator.borrow().robot().paint_count(),
        simulator.borrow().robot().block_count()
    );
}
