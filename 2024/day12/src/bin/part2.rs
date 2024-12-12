use anyhow::{Context, Result};
use clap::Parser;
use day12::CLIArgs;

fn main() -> Result<()> {
    let args = CLIArgs::parse();
    let map = day12::read_map(&args.input_path).with_context(|| {
        format!(
            "Failed to read garden map from given file({}).",
            args.input_path.display()
        )
    })?;

    let fences_price = map
        .all_regions()
        .iter()
        .map(|r| r.area() * r.sides_n())
        .sum::<usize>();
    println!(
        "The total price of fencing given garden map is {}.",
        fences_price
    );

    Ok(())
}
