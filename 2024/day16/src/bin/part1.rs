use anyhow::{Context, Result};
use clap::Parser;
use day16::CLIArgs;

fn main() -> Result<()> {
    let args = CLIArgs::parse();
    let map = day16::read_map(&args.input_path).with_context(|| {
        format!(
            "Failed to read map from given file({}).",
            args.input_path.display()
        )
    })?;

    if let Some(min_score_actions) = map.min_score_actions() {
        let min_score = min_score_actions
            .iter()
            .map(|action| action.score())
            .sum::<usize>();
        println!("The minimium score of completing the map is {}.", min_score);
    } else {
        eprintln!("There're no actions can complete the given map.");
    }

    Ok(())
}
