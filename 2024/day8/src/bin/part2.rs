use anyhow::{Context, Result};
use clap::Parser;
use day8::CLIArgs;

fn main() -> Result<()> {
    let args = CLIArgs::parse();
    let signal_map = day8::read_signal_map(&args.input_path).with_context(|| {
        format!(
            "Failed to read signal map from given file({}).",
            args.input_path.display()
        )
    })?;

    let antinode_locs = signal_map.harmonic_antinode_positions();
    println!(
        "There is(are) {} harmonic antinode(s) according to given signal map.",
        antinode_locs.len()
    );

    Ok(())
}
