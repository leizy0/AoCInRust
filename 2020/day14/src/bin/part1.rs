use clap::Parser;
use day14::{CLIArgs, SPCSimulator};

fn main() {
    let args = CLIArgs::parse();
    let ops = day14::read_ops(&args.input_path)
        .inspect_err(|e| {
            eprintln!(
                "Failed to read operations from given input file({}), get error({}).",
                args.input_path.display(),
                e
            )
        })
        .unwrap();
    let mut simulator = SPCSimulator::new();
    for op in &ops {
        simulator.execute(op);
    }

    let non_zero_sum = simulator.non_zero_mem().iter().map(|(_, v)| v).sum::<usize>();
    println!(
        "After initialization, the sum of non zero value in memory is {}.",
        non_zero_sum
    );
}
