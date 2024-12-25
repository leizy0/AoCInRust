use anyhow::{Context, Result};
use clap::Parser;
use day25::CLIArgs;

fn main() -> Result<()> {
    let args = CLIArgs::parse();
    let (keys, locks) = day25::read_key_lock(&args.input_path).with_context(|| {
        format!(
            "Failed to read keys and locks in given file({}).",
            args.input_path.display()
        )
    })?;

    let fit_pairs_n = keys
        .iter()
        .flat_map(|key| locks.iter().map(move |lock| (key, lock)))
        .filter(|(key, lock)| key.fit(lock))
        .count();
    println!(
        "There is(are) {} pair(s) in given keys and locks.",
        fit_pairs_n
    );

    Ok(())
}
