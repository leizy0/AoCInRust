use anyhow::{Context, Result};
use clap::Parser;
use day20::CLIArgs;

fn main() -> Result<()> {
    let args = CLIArgs::parse();
    let map = day20::read_map(&args.input_path).with_context(|| {
        format!(
            "Failed to read map from given file({}).",
            args.input_path.display()
        )
    })?;

    let Some(no_cheat_steps_n) = map.fastest_steps_n() else {
        eprintln!("Given map has no path if no cheat is allowed.");
        return Ok(());
    };

    let cheat_save_threshold = 100;
    let cheat_candidates_n = map.cheats_steps_n(
        no_cheat_steps_n
            .checked_sub(cheat_save_threshold)
            .unwrap_or(no_cheat_steps_n),
    );
    println!(
        "There is(are) {} cheat ways to save at least {} steps on given map.",
        cheat_candidates_n, cheat_save_threshold
    );

    Ok(())
}
