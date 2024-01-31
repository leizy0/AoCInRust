use day12::n_body::{read_n_body, NBodySimulator};

fn main() {
    let input_path = "inputs.txt";
    let init_bodies = match read_n_body(input_path) {
        Ok(v) => v,
        Err(e) => {
            eprintln!(
                "Failed to read n-body setting from input file({}), get error({})",
                input_path, e
            );
            return;
        }
    };

    let mut simulator = NBodySimulator::new(init_bodies.clone());
    let mut step_count = 0usize;
    let report_interval = 50000000;

    loop {
        if (0..4).all(|i| simulator.bodies()[i] == init_bodies[i]) && step_count != 0 {
            println!("After {} step(s), initial state of bodies repeats.", step_count);
            break;
        }else if step_count % report_interval == 0 {
            println!("Initial state of bodies doesn't repeat after {} steps.", step_count);
        }

        simulator.step();
        step_count += 1;
    }
}
