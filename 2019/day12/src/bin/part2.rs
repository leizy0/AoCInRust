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

    let simulator = NBodySimulator::new(init_bodies.clone());
    println!("After {} step(s), initial state of bodies repeats.", simulator.cycle_len());
}
