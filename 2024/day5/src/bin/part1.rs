use anyhow::{Context, Result};
use clap::Parser;
use day5::CLIArgs;

fn main() -> Result<()> {
    let args = CLIArgs::parse();
    let (rules, updates) = day5::read_printer_settings(&args.input_path).with_context(|| {
        format!(
            "Failed to read printer settings from given file({}).",
            args.input_path.display()
        )
    })?;

    let page_n_sum = updates
        .iter()
        .filter(|up| rules.is_valid(up))
        .map(|up| *up.get(up.len() / 2).unwrap_or(&0))
        .sum::<usize>();
    println!(
        "The sum of middle page number in updates which satisfy given rules is {}.",
        page_n_sum
    );

    Ok(())
}
