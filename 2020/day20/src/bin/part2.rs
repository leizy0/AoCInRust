use std::collections::HashSet;

use anyhow::{Context, Result};
use clap::Parser;
use day20::{ArrangedImageMask, Arrangement, CLIArgs, Error, Pixel, SatelliteImage};

fn main() -> Result<()> {
    let args = CLIArgs::parse();
    let tiles = day20::read_tiles(&args.tiles_path).with_context(|| {
        format!(
            "Failed to read tiles from given input file({}).",
            args.tiles_path.display()
        )
    })?;

    let image = SatelliteImage::try_from(tiles)
        .context("Failed to construct satellite image from given tiles.")?;

    if let Some(mask_path) = args.mask_path.as_ref() {
        let mask = day20::read_mask(mask_path).with_context(|| {
            format!(
                "Failed to read mask from given input file({}).",
                mask_path.display()
            )
        })?;
        let masked_pixels_pos = Arrangement::all_arrgs()
            .iter()
            .flat_map(|arrg| ArrangedImageMask::new(&mask, arrg).masked_pixels_pos(&image))
            .collect::<HashSet<_>>();
        let (pixel_rows_n, pixel_cols_n) = image.pixel_size();
        let image_ref = &image;
        let white_pixels_count = (0..pixel_rows_n)
            .flat_map(|r| (0..pixel_cols_n).map(move |c| image_ref.pixel(r, c).unwrap()))
            .filter(|p| **p == Pixel::White)
            .count();
        println!(
            "The water roughness, that is, the count of not masked white pixels, is {}.",
            white_pixels_count - masked_pixels_pos.len()
        );
    } else {
        return Err(Error::NoMaskPath).context("Can't compute roughness of water.");
    }

    Ok(())
}
