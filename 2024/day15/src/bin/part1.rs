use anyhow::{Context, Result};
use clap::Parser;
use day15::{CLIArgs, Tile};

fn main() -> Result<()> {
    let args = CLIArgs::parse();
    let (mut map, move_dirs) = day15::read_game(&args.input_path).with_context(|| {
        format!(
            "Failed to read game(map and move directions) from given file({}).",
            args.input_path.display()
        )
    })?;

    map.simulate(&move_dirs);
    let coord_sum = map
        .position_iter(Tile::Box)
        .map(|p| p.r * 100 + p.c)
        .sum::<usize>();
    println!("The sum of boxes' GPS coordinates in map is {}.", coord_sum);

    Ok(())
}
