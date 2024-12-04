use anyhow::{Context, Result};
use clap::Parser;
use day4::Part2CLIArgs;

fn main() -> Result<()> {
    let args = Part2CLIArgs::parse();
    let letter_mat = day4::read_letter_mat(&args.input_path).with_context(|| {
        format!(
            "Failed to read letter matrix from given file({}).",
            args.input_path.display()
        )
    })?;

    let patterns = day4::read_patterns(&args.patterns_path).with_context(|| {
        format!(
            "Failed to read patterns from given file({}).",
            args.patterns_path.display()
        )
    })?;

    let pats_count = letter_mat.search_pats(&patterns);
    println!(
        "X-MAS appears {} time(s) in given letter matrix.",
        pats_count
    );

    Ok(())
}
