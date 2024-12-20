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

    if map.fastest_steps_n().is_none() {
        eprintln!("Given map has no path if no cheat is allowed.");
        return Ok(());
    };

    let cheat_save_threshold = 100;
    let cheat_duration = 20;
    let saving_map = map.saving_cheats_n(cheat_duration);
    let cheat_candidates_n = saving_map
        .iter()
        .filter_map(|(saving_pico_sec_n, count)| {
            Some(count).filter(|_| *saving_pico_sec_n >= cheat_save_threshold)
        })
        .sum::<usize>();
    println!(
        "There is(are) {} cheat ways to save at least {} steps on given map.",
        cheat_candidates_n, cheat_save_threshold
    );

    Ok(())
}
