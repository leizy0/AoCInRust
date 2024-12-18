use anyhow::{Context, Result};
use clap::Parser;
use day18::{CLIArgs, Map, Position};

fn main() -> Result<()> {
    let args = CLIArgs::parse();
    let corr_positions = day18::read_positions(&args.input_path).with_context(|| {
        format!(
            "Failed to read corrupted positions from given file({}).",
            args.input_path.display()
        )
    })?;
    let mut map = Map::new_square(args.map_size);
    let corrupt_size = args.corrupt_size.unwrap_or(corr_positions.len());
    map.corrupt(&corr_positions[..corrupt_size]);
    let start_pos = Position::new(0, 0);
    let end_pos = Position::new(args.map_size - 1, args.map_size - 1);
    if let Some(min_exit_steps_n) = map.min_steps_n(&start_pos, &end_pos) {
        println!(
            "It takes at least {} steps moving from {} to {} after corrupting given positions.",
            min_exit_steps_n, start_pos, end_pos
        );
    } else {
        eprintln!(
            "There's no path from {} to {} after corrupting given positions.",
            start_pos, end_pos
        );
    }

    Ok(())
}
