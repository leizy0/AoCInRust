use clap::Parser;
use day19::CLIArgs;

fn main() {
    let args = CLIArgs::parse();
    const START_RULE_ID: usize = 0;
    let (checker, msgs) = day19::read_info(&args.input_path, START_RULE_ID).inspect_err(|e| eprintln!("Failed to read information(rules and messages) from given input file({}), get error({}).", args.input_path.display(), e)).unwrap();

    let valid_count = msgs.iter().filter(|m| checker.check(m)).count();
    println!(
        "There are {} messages which are valid according to the given rules(start from rule #{}).",
        valid_count, START_RULE_ID
    );
}
