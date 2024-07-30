use std::collections::HashMap;

use anyhow::{Context, Result};
use clap::Parser;
use day24::CLIArgs;

fn main() -> Result<()> {
    let args = CLIArgs::parse();
    let poss = day24::read_hex_poss(&args.input_path).with_context(|| {
        format!(
            "Failed to read hexagonal positions from given input file({})",
            args.input_path.display()
        )
    })?;
    let mut flip_count_map = HashMap::new();
    for pos in &poss {
        *flip_count_map.entry(pos).or_insert(0usize) += 1;
    }

    let black_count = flip_count_map
        .iter()
        .filter(|(_, flip_n)| *flip_n % 2 == 1)
        .count();
    println!("After flipped, tiles at {} positions(from {} given paths), the count of black tiles is {}.", flip_count_map.len(), poss.len(), black_count);

    Ok(())
}
