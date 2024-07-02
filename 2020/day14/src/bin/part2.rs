use clap::Parser;
use day14::{CLIArgs, MaskMode, SPComputer};

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
    let mut computer = SPComputer::new(MaskMode::MaskAddr);
    for op in &ops {
        computer.execute(op);
    }

    println!(
        "After initialization, the sum of non zero value in memory is {}.",
        computer.non_zero_mem_sum()
    );
}
