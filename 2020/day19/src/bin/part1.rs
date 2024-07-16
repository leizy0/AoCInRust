use clap::Parser;
use day19::CLIArgs;

fn main() {
    let args = CLIArgs::parse();
    let (rules, msgs) = day19::read_info(&args.input_path).inspect_err(|e| eprintln!("Failed to read information(rules and messages) from given input file({}), get error({}).", args.input_path.display(), e)).unwrap();

    const MAIN_RULE_ID: usize = 0;
    let Some(main_rule) = rules.get(MAIN_RULE_ID) else {
        eprintln!(
            "No main rule(#{}) found in given information.",
            MAIN_RULE_ID
        );
        return;
    };
    let valid_count = msgs
        .iter()
        .filter(|m| main_rule.check(m, &rules).is_some_and(|ind| ind == m.len()))
        .count();
    println!(
        "There are {} messages which are valid according to the given rules(start from rule #{}).",
        valid_count, MAIN_RULE_ID
    );
}
