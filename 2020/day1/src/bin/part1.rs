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
    for n in &ints {
        if *n > expect_sum / 2 {
            break;
        }

        let expect_n = expect_sum - n;
        if let Ok(_) = ints.binary_search(&expect_n) {
            println!(
                "In given inputs, the sum of {} and {} is {}, and their product is {}.",
                n,
                expect_n,
                expect_sum,
                n * expect_n
            );
            return;
        }
    }

    println!(
        "Can't find two numbers whose sum is {} in given inputs.",
        expect_sum
    );
}
