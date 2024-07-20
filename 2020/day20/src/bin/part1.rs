use anyhow::{Context, Result};
use clap::Parser;
use day20::{CLIArgs, SatelliteImage};

fn main() -> Result<()> {
    let args = CLIArgs::parse();
    let tiles = day20::read_tiles(&args.input_path).with_context(|| {
        format!(
            "Failed to read tiles from given input file({}).",
            args.input_path.display()
        )
    })?;

    let image = SatelliteImage::try_from(tiles)
        .context("Failed to construct satellite image from given tiles.")?;
    let (rows_n, cols_n) = image.tile_size();
    let corners_pos = [
        (0, 0),
        (0, cols_n - 1),
        (rows_n - 1, 0),
        (rows_n - 1, cols_n - 1),
    ];
    let corner_ids_prod = corners_pos
        .iter()
        .map(|(r, c)| image.tile(*r, *c).unwrap().id())
        .product::<usize>();
    println!("The product of corner tiles' ids is {}.", corner_ids_prod);

    Ok(())
}
