use anyhow::{Context, Result};
use clap::Parser;
use day15::{BoxGame, CLIArgs, PlainTile};

fn main() -> Result<()> {
    let args = CLIArgs::parse();
    let (map, move_dirs) = day15::read_game(&args.input_path).with_context(|| {
        format!(
            "Failed to read game(map and move directions) from given file({}).",
            args.input_path.display()
        )
    })?;

    let mut game = BoxGame::new(map);
    game.simulate(&move_dirs);
    let coord_sum = game
        .map()
        .position_iter(PlainTile::Box)
        .map(|p| p.r * 100 + p.c)
        .sum::<usize>();
    println!(
        "After move(s), he sum of boxes' GPS coordinates in map is {}.",
        coord_sum
    );

    Ok(())
}
