use clap::Parser;
use day4::CliArgs;

fn main() {
    let args = CliArgs::parse();
    let passports = day4::read_pp(&args.input_path)
        .inspect_err(|e| {
            eprintln!(
                "Failed to read passports from given input file({}), get error({}).",
                args.input_path.display(),
                e
            )
        })
        .unwrap();
    let required_prop_names = ["byr", "iyr", "eyr", "hgt", "hcl", "ecl", "pid"];
    let valid_counts = passports
        .iter()
        .filter(|pp| {
            required_prop_names
                .iter()
                .all(|p_name| pp.contains_prop(p_name))
        })
        .count();
    println!(
        "There are {} valid passports in given {}.",
        valid_counts,
        passports.len()
    );
}
