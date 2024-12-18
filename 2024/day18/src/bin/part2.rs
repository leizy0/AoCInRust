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
    let start_pos = Position::new(0, 0);
    let end_pos = Position::new(args.map_size - 1, args.map_size - 1);
    let mut first_break_corrupt_pos = None;
    for corrupt_pos in &corr_positions {
        map.corrupt(&[corrupt_pos.clone()]);
        if map.min_steps_n(&start_pos, &end_pos).is_none() {
            first_break_corrupt_pos = Some(corrupt_pos.clone());
            break;
        }
    }

    if let Some(pos) = first_break_corrupt_pos {
        println!(
            "The first corrupted position that makes no path from {} to {} exists is {}.",
            start_pos, end_pos, pos
        );
    } else {
        eprintln!(
            "There's no corrupted position can break the path from {} to {}.",
            start_pos, end_pos
        );
    }

    Ok(())
}
