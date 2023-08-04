use std::{cell::RefCell, rc::Rc};

use day2_5_7_9_11::int_code::{
    com::{Channel, IntCodeComputer},
    read_int_code,
};

fn main() {
    let int_code_file = "day5_inputs.txt";
    let int_code = read_int_code(int_code_file).expect(&format!(
        "Failed to read int code from file({})",
        int_code_file
    ));
    let mut computer = IntCodeComputer::new(false);
    let input_chan = Rc::new(RefCell::new(Channel::new(&[5])));
    let output_chan = Rc::new(RefCell::new(Channel::new(&[])));
    match computer.execute_with_io(&int_code, input_chan, output_chan.clone()) {
        Ok(res) => println!(
            "After {} steps, execution finished, Outputs: {:?}",
            res.step_count(),
            output_chan.borrow().data()
        ),
        Err(e) => eprintln!("Failed to execute int code, get error({})", e),
    }
}
