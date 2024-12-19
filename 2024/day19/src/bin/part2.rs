use anyhow::{Context, Result};
use clap::Parser;
use day19::CLIArgs;

fn main() -> Result<()> {
    let args = CLIArgs::parse();
    let (patterns, designs) = day19::read_pattern_design(&args.input_path).with_context(|| {
        format!(
            "Failed to read patterns and designs from given file({}).",
            args.input_path.display()
        )
    })?;

    let possible_ways_sum = designs
        .iter()
        .map(|d| d.possible_ways_n(&patterns))
        .sum::<usize>();
    println!(
        "The sum of possible ways that each design can be constructed with given patterns is {}.",
        possible_ways_sum
    );

    Ok(())
}
