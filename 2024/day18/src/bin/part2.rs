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
    let mut clear_ind = 0;
    let mut blocked_ind = corr_positions.len();
    while clear_ind < blocked_ind {
        let center_ind = (clear_ind + blocked_ind) / 2;
        map.reset();
        map.corrupt(&corr_positions[..(center_ind + 1)]);
        if map.min_steps_n(&start_pos, &end_pos).is_some() {
            clear_ind = center_ind + 1;
        } else {
            blocked_ind = center_ind;
        }
    }

    if let Some(first_blocking_corr_pos) = corr_positions.get(blocked_ind) {
        println!(
            "The first corrupted position that makes no path from {} to {} exists is {}.",
            start_pos, end_pos, first_blocking_corr_pos
        );
    } else {
        eprintln!(
            "There's no corrupted position can break the path from {} to {}.",
            start_pos, end_pos
        );
    }

    Ok(())
}
