use anyhow::{Context, Result};
use clap::Parser;
use day15::{CLIArgs, WideBoxGame, WideTile};

fn main() -> Result<()> {
    let args = CLIArgs::parse();
    let (map, move_dirs) = day15::read_game(&args.input_path).with_context(|| {
        format!(
            "Failed to read game(map and move directions) from given file({}).",
            args.input_path.display()
        )
    })?;

    let wide_map = map.widen();
    let mut game = WideBoxGame::new(wide_map);
    game.simulate(&move_dirs);
    let coord_sum = game
        .map()
        .position_iter(WideTile::WideBoxLeft)
        .map(|p| p.r * 100 + p.c)
        .sum::<usize>();
    println!(
        "After move(s), the sum of boxes' GPS coordinates in widened map is {}.",
        coord_sum
    );

    Ok(())
}
