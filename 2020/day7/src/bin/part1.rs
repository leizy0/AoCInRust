use clap::Parser;
use day7::CliArgs;

fn main() {
    let args = CliArgs::parse();
    let bag_rules = day7::read_br(&args.input_path)
        .inspect_err(|e| {
            eprintln!(
                "Failed to read bag rules from given input file({}), get error({}).",
                args.input_path.display(),
                e
            )
        })
        .unwrap();
    let bag_qualifier = "shiny gold";
    if let Some(contained_n) = bag_rules.contained_kinds_n(bag_qualifier) {
        println!(
            "There are {} kinds of bag can contain at least one specified {} bag.",
            contained_n, bag_qualifier
        );
    } else {
        println!("There's no {} bag in given rules.", bag_qualifier);
    }
}
