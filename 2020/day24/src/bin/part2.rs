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
    const FLIP_COUNT: usize = 100;
    for _ in 0..FLIP_COUNT {
        floor.flip();
    }
    println!(
        "After flipped {} times, the count of black tiles on the floor is {}.",
        FLIP_COUNT,
        floor.black_n()
    );

    Ok(())
}
