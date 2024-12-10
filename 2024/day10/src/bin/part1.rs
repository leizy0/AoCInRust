use anyhow::{Context, Result};
use clap::Parser;
use day10::CLIArgs;

fn main() -> Result<()> {
    let args = CLIArgs::parse();
    let map = day10::read_map(&args.input_path).with_context(|| {
        format!(
            "Failed to read topographic map from given file({}).",
            args.input_path.display()
        )
    })?;

    let score_sum = map
        .trailheads()
        .iter()
        .map(|pos| map.score_from(pos))
        .sum::<usize>();
    println!(
        "The score sum of all trailheads from given topographic map is {}.",
        score_sum
    );

    Ok(())
}
