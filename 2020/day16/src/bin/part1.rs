use clap::Parser;
use day16::CLIArgs;

fn main() {
    let args = CLIArgs::parse();
    let (field_rules, _, other_tickets) = day16::read_ticket_info(&args.input_path)
        .inspect_err(|e| {
            eprintln!(
                "Failed to read ticket information from given input file({}), get error({}).",
                args.input_path.display(),
                e
            )
        })
        .unwrap();

    let invalid_sum = other_tickets
        .iter()
        .flat_map(|t| t)
        .filter(|n| !field_rules.iter().any(|r| r.contains(**n)))
        .sum::<usize>();
    println!(
        "The ticket scanning rate(sum of invalid ticket field) is {}.",
        invalid_sum
    );
}
