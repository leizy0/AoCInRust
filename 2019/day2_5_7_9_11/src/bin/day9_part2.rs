use std::{cell::RefCell, rc::Rc};

use day2_5_7_9_11::int_code::{
    com::{Channel, IntCodeComputer},
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

    let mut computer = IntCodeComputer::new(true);
    let input_chan = Rc::new(RefCell::new(Channel::new(&[2])));
    let output_chan = Rc::new(RefCell::new(Channel::new(&[])));
    match computer.execute_with_io(&int_code, input_chan, output_chan.clone()) {
        Ok(res) => println!(
            "Boost program in sensor boost mode takes {} steps to finish, get outputs({:?})",
            res.step_count(),
            output_chan.borrow().data()
        ),
        Err(e) => eprintln!(
            "Failed to execute Boost program in sensor boost, get error({})",
            e
        ),
    }
}
