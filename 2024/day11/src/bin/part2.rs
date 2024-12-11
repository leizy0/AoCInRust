use anyhow::{Context, Result};
use clap::Parser;
use day11::CLIArgs;

fn main() -> Result<()> {
    let args = CLIArgs::parse();
    let stones = day11::read_stones(&args.input_path).with_context(|| {
        format!(
            "Failed to read stones from given file({}).",
            args.input_path.display()
        )
    })?;

    let blink_count = 75;
    let stone_count = stones.stone_n_after_blink(blink_count);
    println!(
        "After {} blink(s), the number of given stones changes to {}.",
        blink_count, stone_count,
    );

    Ok(())
}
