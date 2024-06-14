use clap::Parser;
use day9::CliArgs;

fn main() {
    let args = CliArgs::parse();
    let nums = day9::read_num(&args.input_path)
        .inspect_err(|e| {
            eprintln!(
                "Failed to read numbers from given input file({}), get error({}).",
                args.input_path.display(),
                e
            )
        })
        .unwrap();
    const ADDEND_LEN: usize = 25;
    if nums.len() <= ADDEND_LEN {
        println!("Given numbers({} in total) aren't enough to be encrypted data using XMAS, at least {} numbers are expected.", nums.len(), ADDEND_LEN + 1);
        return;
    }

    if let Some(invalid_value) = day9::invalid_xmax_v(&nums, ADDEND_LEN) {
        println!(
            "The first invalid value according to XMAS rule is {}.",
            invalid_value
        );
    } else {
        println!(
            "There isn't any invalid value according to XMAS rule in given numbers({} in total)",
            nums.len()
        );
    }
}
