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

    let mut simulator = NBodySimulator::new(init_bodies);
    let step_count = 1001;
    let report_interval = 100;
    for i in 0..step_count {
        if i % report_interval == 0 {
            println!("After {} steps, n-body state is:", i);
            for body in simulator.bodies() {
                println!("{:?}", body);
            }

            println!(
                "Potential energy = {}, kinetic energy = {}, total energy = {}",
                simulator.potential_energy(),
                simulator.kinetic_energy(),
                simulator.total_energy()
            );
        }

        simulator.step();
    }
}
