use clap::Parser;
use day1::CliArgs;

fn main() {
    let args = CliArgs::parse();
    let mut ints = day1::read_ints(&args.input_path)
        .inspect_err(|e| {
            eprintln!(
                "Failed to read integers from given input file({}), get error({})",
                args.input_path.display(),
                e
            )
        })
        .unwrap();
    ints.sort_unstable();

    let expect_sum = 2020;
    if let Some(ns) = day1::find_ints_of_sum(&ints, expect_sum, 3) {
        let prod = ns.iter().fold(1, |prod, n| prod * n);
        println!(
            "In given inputs, the sum of {:?} is {}, their product is {}.",
            ns, expect_sum, prod
        );
    } else {
        println!(
            "Can't find numbers whose sum is {} in given inputs.",
            expect_sum
        );
    }
}
