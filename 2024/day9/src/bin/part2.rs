use anyhow::{Context, Result};
use clap::Parser;
use day9::CLIArgs;

fn main() -> Result<()> {
    let args = CLIArgs::parse();
    let mut file_system = day9::read_file_system(&args.input_path).with_context(|| {
        format!(
            "Failed to read file system information from given file({}).",
            args.input_path.display()
        )
    })?;

    file_system.compact_per_file();
    let compact_checksum = file_system.checksum();
    println!(
        "The checksum of given file system after compaction with one file strategy is {}.",
        compact_checksum
    );

    Ok(())
}
