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

    let possible_n = designs
        .iter()
        .filter(|d| d.is_possible_with(&patterns))
        .count();
    println!(
        "There is(are) {} possible design(s) with given patterns.",
        possible_n
    );

    Ok(())
}
