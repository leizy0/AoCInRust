use clap::Parser;
use day5::CliArgs;

fn main() {
    let args = CliArgs::parse();
    let passes = day5::read_pass(&args.input_path)
        .inspect_err(|e| {
            eprintln!(
                "Failed to read boarding pass from given input file({}), get error({}).",
                args.input_path.display(),
                e
            )
        })
        .unwrap();
    let max_id = passes.iter().map(|p| p.id()).max().unwrap();
    println!(
        "The maximium seat id in given {} boarding pass is {}.",
        passes.len(),
        max_id
    );
}
