use anyhow::{Context, Result};
use clap::Parser;
use day6::CLIArgs;

fn main() -> Result<()> {
    let args = CLIArgs::parse();
    let lab = day6::read_lab(&args.input_path).with_context(|| {
        format!(
            "Failed to read laboratory from given file({}).",
            args.input_path.display()
        )
    })?;

    let patrol_n = lab.patrol_positions();
    println!(
        "The guard will visit {} position(s) before leaving given laboratory.",
        patrol_n.len()
    );

    Ok(())
}
