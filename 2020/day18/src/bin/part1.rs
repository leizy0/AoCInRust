use std::collections::HashMap;

use clap::Parser;
use day18::{CLIArgs, Operator};

fn main() {
    let args = CLIArgs::parse();
    let ops_prec = HashMap::from([(Operator::Add, 1), (Operator::Mul, 1)]);
    let exps = day18::read_exps(&args.input_path, &ops_prec)
        .inspect_err(|e| {
            eprintln!(
                "Failed to read expressions from given input file({}), get error({}).",
                args.input_path.display(),
                e
            )
        })
        .unwrap();
    let res_sum = exps.iter().map(|e| e.value()).sum::<usize>();
    println!("The sum of given expressions results is {}.", res_sum);
}
