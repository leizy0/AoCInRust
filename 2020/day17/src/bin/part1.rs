use clap::Parser;
use day17::{CLIArgs, CubeSpace3D, CubeSpaceSimulator};

fn main() {
    let args = CLIArgs::parse();
    let init_state = day17::read_state(&args.input_path)
        .inspect_err(|e| {
            eprintln!(
                "Failed to read initial states from given input file({}), get error({}).",
                args.input_path.display(),
                e
            )
        })
        .unwrap();
    let mut simulator = CubeSpaceSimulator::<CubeSpace3D>::new(&init_state);
    const STEP_COUNT: usize = 6;

    for _s_ind in 0..STEP_COUNT {
        simulator.step();
    }

    println!(
        "After {} step(s), the whole space has {} active cubes.",
        STEP_COUNT,
        simulator.active_n()
    );
}
