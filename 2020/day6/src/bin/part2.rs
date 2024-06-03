use clap::Parser;
use day6::CliArgs;

fn main() {
    let args = CliArgs::parse();
    let grp_answers = day6::read_ga(&args.input_path)
        .inspect_err(|e| {
            eprintln!(
                "Failed to read group answers from given input file({}), get error({}).",
                args.input_path.display(),
                e
            )
        })
        .unwrap();
    let sum = grp_answers
        .iter()
        .map(|ga: &day6::GroupAnwser| ga.all_app_n())
        .sum::<usize>();
    println!(
        "The sum of count of unique approved quesitions from given answers is {}.",
        sum
    );
}
