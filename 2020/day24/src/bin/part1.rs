use anyhow::{Context, Result};
use clap::Parser;
use day24::{CLIArgs, HexPlane};

fn main() -> Result<()> {
    let args = CLIArgs::parse();
    let poss = day24::read_hex_poss(&args.input_path).with_context(|| {
        format!(
            "Failed to read hexagonal positions from given input file({})",
            args.input_path.display()
        )
    })?;

    let mut floor = HexPlane::new();
    for pos in &poss {
        floor.flip_pos(pos);
    }
    println!(
        "After flipped, the count of tiles from {} given paths which flips to black is {}.",
        poss.len(),
        floor.black_n()
    );

    Ok(())
}
