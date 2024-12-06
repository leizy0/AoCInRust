use std::collections::HashSet;

use anyhow::{Context, Result};
use clap::Parser;
use day6::CLIArgs;

fn main() -> Result<()> {
    let args = CLIArgs::parse();
    let mut lab = day6::read_lab(&args.input_path).with_context(|| {
        format!(
            "Failed to read laboratory from given file({}).",
            args.input_path.display()
        )
    })?;

    let mut patrol_positions = lab.patrol_positions();
    patrol_positions.remove(lab.guard().pos());

    let mut loop_positions = HashSet::new();
    for pos in &patrol_positions {
        *lab.tile_mut(pos).unwrap() = true;
        if lab.is_loop_if_patrol() {
            loop_positions.insert(pos.clone());
        }

        *lab.tile_mut(pos).unwrap() = false;
    }

    println!(
        "There is(are) {} location(s) which can make guard loops in given laboratory.",
        loop_positions.len()
    );

    Ok(())
}
