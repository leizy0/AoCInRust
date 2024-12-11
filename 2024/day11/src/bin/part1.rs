use anyhow::{Context, Result};
use clap::Parser;
use day11::CLIArgs;

fn main() -> Result<()> {
    let args = CLIArgs::parse();
    let mut stones = day11::read_stones(&args.input_path).with_context(|| {
        format!(
            "Failed to read stones from given file({}).",
            args.input_path.display()
        )
    })?;

    let blink_count = 25;
    for _ in 0..blink_count {
        stones.blink();
    }
    println!(
        "After {} blink(s), the number of given stones changes to {}.",
        blink_count,
        stones.count()
    );

    Ok(())
}
