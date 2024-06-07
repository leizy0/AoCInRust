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
    if let Some(contain_n) = bag_rules.contain_bags_n(bag_qualifier) {
        println!(
            "According to given rules, one {} bag can contain {} bags in total.",
            bag_qualifier, contain_n
        );
    } else {
        println!("There's no {} bag in given rules.", bag_qualifier);
    }
}
